//! CLI command definitions and subcommand handlers

use clap::{Parser, Subcommand, ValueEnum};

pub mod analyze;
pub mod blueprint;
pub mod brain;
pub mod build;
pub mod clarify;
pub mod constitution;
pub mod contract;
pub mod doctor;
pub mod harvest;
pub mod implement;
pub mod init;
pub mod intake;
pub mod log;
pub mod mcp;
pub mod mode;
pub mod plan;
pub mod progress;
pub mod prompt;
pub mod providers;
pub mod repomap;
pub mod review;
pub mod run;
pub mod ship;
pub mod snapshot;
pub mod status;
pub mod style;
pub mod tasks;
pub mod ui;
pub mod undo;
pub mod update;
pub mod watch;
pub mod wizard;

pub use mcp::McpAction;

/// VibeAnvil - Contract-first vibe coding with evidence, audit, and repo-brain harvesting
#[derive(Parser)]
#[command(name = "vibeanvil")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new vibeanvil workspace
    Init {
        /// Force re-initialization even if workspace exists
        #[arg(short, long)]
        force: bool,
    },

    /// Capture requirements/intake for the project
    Intake {
        /// The requirement or intake message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Generate or view the blueprint
    Blueprint {
        /// Auto-generate blueprint from intake
        #[arg(short, long)]
        auto: bool,
    },

    /// Manage the contract (create, validate, lock)
    Contract {
        /// Contract action to perform
        #[arg(value_enum)]
        action: ContractAction,
    },

    /// Create an implementation plan
    Plan {
        /// Provider to use for plan generation
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
    },

    /// Execute a build
    Build(BuildArgs),

    /// Review the current build
    Review {
        /// Review action
        #[arg(value_enum)]
        action: ReviewAction,
    },

    /// Create a snapshot of current state
    Snapshot {
        /// Snapshot message/description
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Mark the project as shipped
    Ship {
        /// Version tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Ship message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Harvest repos for the brain pack (dynamic, user-driven)
    Harvest(HarvestArgs),

    /// Manage the brain pack
    Brain(BrainArgs),

    /// Show current workflow status
    Status {
        /// Show verbose status with history
        #[arg(short, long)]
        verbose: bool,

        /// Output as JSON (machine-readable)
        #[arg(long)]
        json: bool,
    },

    /// View audit log
    Log {
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "20")]
        lines: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check for updates
    Update,

    /// Download and install the latest version
    Upgrade,

    /// Check system and workspace health
    Doctor,

    /// Interactive wizard menu
    Wizard,

    /// Print install or usage prompts
    Prompt {
        /// Which prompt to print
        #[arg(value_enum)]
        kind: PromptKind,
    },

    /// List available AI providers and capability matrix
    Providers {
        /// Subcommand: list, matrix, recommend, compare
        #[arg(value_name = "SUBCOMMAND")]
        subcommand: Option<String>,
        /// Additional arguments (task description or provider names)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Undo the last AI-made change (reverts last commit)
    Undo {
        /// Show what would be undone without actually undoing
        #[arg(long)]
        dry_run: bool,
    },

    // ============ NEW WORKFLOW COMMANDS (Spec-Kit & Aider inspired) ============
    /// Set project principles and governance guidelines
    Constitution {
        /// Guidelines to incorporate (interactive if not provided)
        #[arg(short, long)]
        guidelines: Option<String>,
        /// View current constitution only
        #[arg(long)]
        view: bool,
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
    },

    /// Clarify requirements with interactive Q&A
    Clarify {
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
    },

    /// Generate actionable tasks from implementation plan
    Tasks {
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
        /// Regenerate tasks even if they exist
        #[arg(long)]
        regenerate: bool,
        /// Mark a task as done
        #[arg(long)]
        done: Option<String>,
    },

    /// Analyze artifacts for consistency and coverage
    Analyze {
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
    },

    /// Execute tasks to implement the plan
    Implement {
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
        /// Specific task ID to implement
        #[arg(long)]
        task: Option<String>,
        /// Implement all remaining tasks
        #[arg(long)]
        all: bool,
        /// Show what would be done without doing it
        #[arg(long)]
        dry_run: bool,
    },

    /// Run a command and optionally share output with AI
    Run {
        /// Command to run
        command: String,
        /// Capture output as evidence
        #[arg(long)]
        capture: bool,
        /// Share output with AI for analysis
        #[arg(long)]
        share: bool,
    },

    /// Run tests with optional auto-fix
    Test {
        /// Custom test command
        #[arg(long)]
        cmd: Option<String>,
        /// Auto-fix failing tests
        #[arg(long)]
        fix: bool,
    },

    /// Run linter with optional auto-fix
    Lint {
        /// Custom lint command
        #[arg(long)]
        cmd: Option<String>,
        /// Auto-fix lint errors
        #[arg(long)]
        fix: bool,
    },

    /// Generate a repository map for AI context
    Map {
        /// Maximum tokens for context output
        #[arg(long)]
        max_tokens: Option<usize>,
    },

    /// Chat with AI in different modes (ask/code/architect/help)
    Chat {
        /// Chat mode
        #[arg(value_enum, default_value = "code")]
        mode: ChatModeArg,
        /// Message to send
        message: String,
        /// Provider to use
        #[arg(short, long, default_value = "claude-code")]
        provider: String,
    },

    /// MCP (Model Context Protocol) server for AI tool integration
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
}

/// Chat mode argument
#[derive(Clone, ValueEnum)]
pub enum ChatModeArg {
    /// Ask questions without making changes
    Ask,
    /// Make code changes
    Code,
    /// High-level architecture proposals
    Architect,
    /// Get help with VibeAnvil
    Help,
}

#[derive(Clone, ValueEnum)]
pub enum ContractAction {
    /// Create a new contract
    Create,
    /// Validate the current contract
    Validate,
    /// Lock the contract (no more changes)
    Lock,
    /// Show contract status
    Status,
}

#[derive(Clone, ValueEnum)]
pub enum ReviewAction {
    /// Start review process
    Start,
    /// Pass the review
    Pass,
    /// Fail the review with reason
    Fail,
    /// Show review status
    Status,
}

#[derive(Clone, ValueEnum)]
pub enum PromptKind {
    /// Installer prompt for LLM paste-in setup
    #[value(name = "install")]
    Install,
}

#[derive(clap::Args)]
pub struct BuildArgs {
    /// Build mode
    #[arg(value_enum, default_value = "manual")]
    pub mode: BuildMode,

    /// Provider to use (for auto/iterate modes)
    #[arg(short, long, default_value = "claude-code")]
    pub provider: String,

    /// Maximum iterations (for iterate mode)
    #[arg(long, default_value = "5")]
    pub max: u32,

    /// Strict mode - fail on first error
    #[arg(long)]
    pub strict: bool,

    /// Timeout in seconds per iteration
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Skip test execution
    #[arg(long)]
    pub no_test: bool,

    /// Skip lint execution
    #[arg(long)]
    pub no_lint: bool,

    /// Capture evidence
    #[arg(long)]
    pub evidence: bool,

    /// Watch mode - auto-rebuild on file changes (iterate mode only)
    #[arg(long)]
    pub watch: bool,

    /// Resume from last saved progress
    #[arg(long)]
    pub resume: bool,

    /// Build action (for manual mode)
    #[arg(value_enum)]
    pub action: Option<ManualBuildAction>,
}

#[derive(Clone, ValueEnum, Default)]
pub enum BuildMode {
    /// Manual build with explicit steps
    #[default]
    Manual,
    /// Automatic single-shot build
    Auto,
    /// Iterative test/lint/fix loop
    Iterate,
}

#[derive(Clone, ValueEnum)]
pub enum ManualBuildAction {
    /// Start the manual build
    Start,
    /// Capture evidence during build
    Evidence,
    /// Complete the manual build
    Complete,
}

/// Harvest command arguments
#[derive(clap::Args)]
pub struct HarvestArgs {
    #[command(subcommand)]
    pub command: Option<HarvestCommands>,

    /// Use a named preset from brainpacks/presets.yaml
    #[arg(long)]
    pub preset: Option<String>,

    /// Search query terms (repeatable)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub query: Vec<String>,

    /// Topic filters (repeatable)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub topic: Vec<String>,

    /// Language filter
    #[arg(short, long)]
    pub language: Option<String>,

    /// Maximum repos to harvest
    #[arg(long, default_value = "20")]
    pub max_repos: usize,

    /// Minimum stars filter
    #[arg(long, default_value = "10")]
    pub min_stars: u32,

    /// Only repos updated within N days
    #[arg(long, default_value = "365")]
    pub updated_within_days: u32,

    /// Download method
    #[arg(long, value_enum, default_value = "tarball")]
    pub download: DownloadMethod,

    /// Cache directory
    #[arg(long)]
    pub cache_dir: Option<String>,

    /// Ignore glob patterns (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    pub ignore_glob: Vec<String>,

    /// Allow glob patterns (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    pub allow_glob: Vec<String>,
}

#[derive(Subcommand)]
pub enum HarvestCommands {
    /// List available harvest presets
    Presets,
}

#[derive(Clone, ValueEnum, Default)]
pub enum DownloadMethod {
    #[default]
    Tarball,
    Git,
}

/// Brain command arguments
#[derive(clap::Args)]
pub struct BrainArgs {
    /// Brain command
    #[command(subcommand)]
    pub command: BrainCommands,
}

#[derive(Subcommand)]
pub enum BrainCommands {
    /// Ensure brain pack is initialized with core knowledge
    Ensure {
        /// Force refresh core even if fingerprint matches
        #[arg(long)]
        refresh_core: bool,

        /// Show detailed parsing errors (line numbers)
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show brain pack statistics
    Stats {
        /// Output as JSON (machine-readable)
        #[arg(long)]
        json: bool,
    },

    /// Search the brain pack
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,

        /// Filter by record type (e.g., function, class, doc)
        #[arg(short = 't', long)]
        record_type: Option<String>,

        /// Filter by language (e.g., rust, python, typescript)
        #[arg(short = 'l', long)]
        language: Option<String>,
    },

    /// Export the brain pack
    Export {
        /// Export format
        #[arg(value_enum, default_value = "jsonl")]
        format: ExportFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Include anonymized source IDs (default: false for privacy)
        #[arg(long)]
        include_source_ids: bool,

        /// Limit entries for markdown export (default: 50, use 0 for no limit)
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Compact the brain pack (dedup JSONL, optimize SQLite)
    Compact,

    /// Pack the current codebase into a single AI-friendly file
    Pack {
        /// Output file path (default: context.xml)
        #[arg(short, long, default_value = "context.xml")]
        output: String,

        /// Output format (xml or markdown)
        #[arg(short, long, default_value = "xml")]
        format: String,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    /// JSON Lines format
    Jsonl,
    /// Markdown format
    Md,
}
