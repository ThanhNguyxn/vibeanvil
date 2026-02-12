//! Build command handler

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use crate::audit::{generate_session_id, AuditLogger};
use crate::build::iterate::IterateBuild;
use crate::build::{AutoBuild, BuildConfig, BuildMode, ManualBuild};
use crate::cli::progress::BuildProgress;
use crate::cli::{BuildArgs, ManualBuildAction};
use crate::prompt;
use crate::state::State;
use crate::workspace;

pub async fn run(args: BuildArgs) -> Result<()> {
    // Handle resume flag
    if args.resume {
        return handle_resume().await;
    }

    let state_data = workspace::load_state().await?;

    if !state_data.current_state.is_at_least(State::PlanCreated) {
        anyhow::bail!("Plan not created. Run 'vibeanvil plan' first.");
    }

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);

    // Build config from args
    let config = BuildConfig {
        mode: match args.mode {
            crate::cli::BuildMode::Manual => BuildMode::Manual,
            crate::cli::BuildMode::Auto => BuildMode::Auto,
            crate::cli::BuildMode::Iterate => BuildMode::Iterate,
        },
        provider: args.provider.clone(),
        max_iterations: args.max,
        strict: args.strict,
        timeout_secs: args.timeout,
        skip_tests: args.no_test,
        skip_lint: args.no_lint,
        capture_evidence: args.evidence,
    };

    // Handle watch mode (only for iterate)
    if args.watch {
        if !matches!(config.mode, BuildMode::Iterate) {
            anyhow::bail!("Watch mode is only available for 'iterate' build mode. Use: vibeanvil build iterate --watch");
        }
        return run_watch_mode(config, &session_id, &logger).await;
    }

    match config.mode {
        BuildMode::Manual => {
            run_manual_build(&args, &session_id, &logger).await?;
        }
        BuildMode::Auto => {
            run_auto_build(config, &session_id, &logger).await?;
        }
        BuildMode::Iterate => {
            run_iterate_build(config, &session_id, &logger).await?;
        }
    }

    Ok(())
}

/// Handle --resume flag
async fn handle_resume() -> Result<()> {
    println!("{}", "🔄 Checking for resumable build...".cyan());

    if let Some(progress) = BuildProgress::load().await? {
        println!();
        println!("{}", "Found previous build session:".white().bold());
        println!("  {} {}", "Session:".dimmed(), progress.session_id);
        println!("  {} {}", "Mode:".dimmed(), progress.mode);
        println!("  {} {}", "Step:".dimmed(), progress.current_step);
        println!(
            "  {} {}/{}",
            "Iteration:".dimmed(),
            progress.current_iteration,
            progress.max_iterations
        );
        println!(
            "  {} {}",
            "Started:".dimmed(),
            progress.started_at.format("%Y-%m-%d %H:%M:%S")
        );

        if let Some(ref error) = progress.last_error {
            println!("  {} {}", "Last Error:".red(), error);
        }

        println!();
        println!(
            "{}",
            "To continue from this point, run the build command again without --resume.".dimmed()
        );
        println!(
            "{}",
            "The session ID will be reused automatically.".dimmed()
        );
    } else {
        println!();
        println!("{}", "No resumable build found.".yellow());
        println!(
            "{}",
            "Start a new build with: vibeanvil build iterate".dimmed()
        );
    }

    Ok(())
}

/// Run in watch mode - auto-rebuild on file changes
async fn run_watch_mode(config: BuildConfig, session_id: &str, logger: &AuditLogger) -> Result<()> {
    use crate::cli::watch::FileWatcher;

    println!("{}", "🔄 Starting watch mode...".cyan().bold());
    println!();

    // Initial build
    run_iterate_build(config.clone(), session_id, logger).await?;

    // Start watching
    let watcher = FileWatcher::new();

    // Clone what we need for the closure
    let config_clone = config.clone();
    let session_id_clone = session_id.to_string();

    // Note: This is a blocking call that runs the event loop
    // In a real implementation, you'd want to handle this with proper async
    watcher.watch(move || {
        // Create a new runtime for each rebuild
        // This is a workaround for the sync callback requirement
        let rt = tokio::runtime::Runtime::new()?;
        let logger = AuditLogger::new(&session_id_clone);

        rt.block_on(async {
            // Reset state for rebuild
            let mut state = workspace::load_state().await?;
            if state.current_state == State::BuildDone {
                // Allow re-running build after completion
                state.current_state = State::PlanCreated;
                workspace::save_state(&state).await?;
            }

            run_iterate_build(config_clone.clone(), &session_id_clone, &logger).await
        })
    })?;

    Ok(())
}

async fn run_manual_build(args: &BuildArgs, session_id: &str, logger: &AuditLogger) -> Result<()> {
    let action = args.action.clone().unwrap_or(ManualBuildAction::Start);

    match action {
        ManualBuildAction::Start => {
            // Update state to build in progress
            let mut state = workspace::load_state().await?;
            if state.current_state == State::PlanCreated || state.current_state == State::BuildDone
            {
                state.transition_to(State::BuildInProgress, "build start", session_id)?;
                workspace::save_state(&state).await?;
                logger
                    .log_state_transition(
                        "build start",
                        state.current_state,
                        State::BuildInProgress,
                    )
                    .await?;
            }

            let mut build = ManualBuild::new(session_id).await?;
            build.start().await?;
        }
        ManualBuildAction::Evidence => {
            let build = ManualBuild::new(session_id).await?;
            build.capture_evidence().await?;
        }
        ManualBuildAction::Complete => {
            let build = ManualBuild::new(session_id).await?;
            let result = build.complete().await?;

            // Update state to build done
            let mut state = workspace::load_state().await?;
            state.transition_to(State::BuildDone, "build complete", session_id)?;
            workspace::save_state(&state).await?;
            logger
                .log_state_transition("build complete", State::BuildInProgress, State::BuildDone)
                .await?;

            println!("✓ Build completed");
            if result.success {
                println!("Next: vibeanvil review start");
            }
        }
    }

    Ok(())
}

async fn run_auto_build(config: BuildConfig, session_id: &str, logger: &AuditLogger) -> Result<()> {
    // Update state to build in progress
    let mut state = workspace::load_state().await?;
    if state.current_state == State::PlanCreated {
        state.transition_to(State::BuildInProgress, "build auto", session_id)?;
        workspace::save_state(&state).await?;
        logger
            .log_state_transition(
                "build auto start",
                State::PlanCreated,
                State::BuildInProgress,
            )
            .await?;
    }

    println!("🔧 Running auto build with {} provider...", config.provider);

    let build = AutoBuild::new(config, session_id);

    // Read plan and contract for context
    let plan_path = workspace::workspace_path().join("plan.md");
    let plan = tokio::fs::read_to_string(&plan_path)
        .await
        .unwrap_or_default();
    let contract = load_contract().await;
    let repo_context = build_repo_context();

    let prompt = build_developer_prompt(&plan, &contract, &repo_context);
    let result = build.execute(&prompt).await?;

    // Update state to build done
    let mut state = workspace::load_state().await?;
    state.transition_to(State::BuildDone, "build auto complete", session_id)?;
    workspace::save_state(&state).await?;
    logger
        .log_state_transition(
            "build auto complete",
            State::BuildInProgress,
            State::BuildDone,
        )
        .await?;

    if result.success {
        println!("✓ Auto build completed successfully");
    } else {
        println!("✗ Auto build completed with errors:");
        for error in &result.errors {
            println!("  - {}", error);
        }
    }

    println!();
    println!("Next: vibeanvil review start");

    Ok(())
}

async fn run_iterate_build(
    config: BuildConfig,
    session_id: &str,
    logger: &AuditLogger,
) -> Result<()> {
    // Update state to build in progress
    let mut state = workspace::load_state().await?;
    if state.current_state == State::PlanCreated {
        state.transition_to(State::BuildInProgress, "build iterate", session_id)?;
        workspace::save_state(&state).await?;
        logger
            .log_state_transition(
                "build iterate start",
                State::PlanCreated,
                State::BuildInProgress,
            )
            .await?;
    }

    println!(
        "🔄 Running iterate build (max {} iterations)...",
        config.max_iterations
    );

    let build = IterateBuild::new(config.clone(), session_id).await?;

    // Read plan and contract for context
    let plan_path = workspace::workspace_path().join("plan.md");
    let plan = tokio::fs::read_to_string(&plan_path)
        .await
        .unwrap_or_default();
    let contract = load_contract().await;
    let repo_context = build_repo_context();

    let prompt = build_developer_prompt(&plan, &contract, &repo_context);
    let result = build.execute(&prompt).await?;

    // Update state to build done
    let mut state = workspace::load_state().await?;
    state.transition_to(State::BuildDone, "build iterate complete", session_id)?;
    workspace::save_state(&state).await?;
    logger
        .log_state_transition(
            "build iterate complete",
            State::BuildInProgress,
            State::BuildDone,
        )
        .await?;

    if result.success {
        println!(
            "✓ Iterate build completed in {} iteration(s)",
            result.iterations
        );
    } else {
        println!(
            "✗ Iterate build failed after {} iteration(s):",
            result.iterations
        );
        for error in &result.errors {
            println!("  - {}", error);
        }
    }

    println!();
    println!("Next: vibeanvil review start");

    Ok(())
}

async fn load_contract() -> String {
    let contract_path = workspace::contracts_path().join("contract.json");
    tokio::fs::read_to_string(&contract_path)
        .await
        .unwrap_or_default()
}

fn build_repo_context() -> String {
    let mut repo_map = crate::brain::map::RepositoryMap::new();
    let workspace_path = workspace::workspace_path();
    let root = workspace_path.parent().unwrap_or(Path::new("."));
    if repo_map.scan(root).is_ok() {
        repo_map.to_markdown()
    } else {
        String::new()
    }
}

fn build_developer_prompt(plan: &str, contract: &str, context: &str) -> String {
    let mut vars = HashMap::new();
    vars.insert("task", plan);
    vars.insert("contract", contract);
    vars.insert("context", context);

    match prompt::load_template("developer") {
        Ok(template) => prompt::render(&template, &vars),
        Err(_) => format!("Implement the following plan:\n\n{}", plan),
    }
}
