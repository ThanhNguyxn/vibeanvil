//! Iterate build mode - test/lint/fix loop

use anyhow::Result;
use std::process::Command;

use super::{BuildConfig, BuildResult};
use crate::evidence::EvidenceCollector;
use crate::guardrails::capsule::{generate_capsule_id, Capsule, CapsuleMeta};
use crate::guardrails::classifier::RiskClassifier;
use crate::guardrails::gate::ApprovalGate;
use crate::provider::{get_provider, Context};
use crate::audit::AuditLogger;
use crate::workspace;

/// Iteration result
#[derive(Debug)]
pub struct IterationResult {
    pub iteration: u32,
    pub test_passed: bool,
    pub lint_passed: bool,
    pub errors: Vec<String>,
    pub fixed: bool,
}

/// Iterate build - loop until tests/lint pass or max iterations
pub struct IterateBuild {
    config: BuildConfig,
    session_id: String,
    evidence: EvidenceCollector,
}

impl IterateBuild {
    /// Create new iterate build
    pub async fn new(config: BuildConfig, session_id: &str) -> Result<Self> {
        let evidence = EvidenceCollector::new(session_id).await?;
        Ok(Self {
            config,
            session_id: session_id.to_string(),
            evidence,
        })
    }

    /// Execute iterate loop
    #[allow(unused_assignments)]
    pub async fn execute(&self, initial_prompt: &str) -> Result<BuildResult> {
        let mut iterations = 0;
        let mut all_errors = vec![];
        let all_warnings = vec![];
        let mut last_output = String::new();

        let provider = get_provider(&self.config.provider)?;
        let context = Context {
            working_dir: std::env::current_dir()?,
            session_id: self.session_id.clone(),
            contract_hash: None,
        };

        // Initial apply
        println!("→ Iteration 1: Applying initial changes...");
        let response = provider.execute(initial_prompt, &context).await?;
        last_output = response.output.clone();
        iterations += 1;

        // Apply guardrails gate to initial changes
        if !self.gatekeep_changes(iterations).await? {
            return Ok(BuildResult {
                success: false,
                iterations,
                errors: vec!["Change denied by guardrails".to_string()],
                warnings: all_warnings,
                evidence_files: vec![],
                output: "Guardrails denied the changes".to_string(),
            });
        }

        loop {
            if iterations > self.config.max_iterations {
                println!("✗ Max iterations ({}) reached", self.config.max_iterations);
                break;
            }

            // Run tests (unless skipped)
            let test_result = if !self.config.skip_tests {
                self.run_tests().await?
            } else {
                TestResult {
                    passed: true,
                    output: String::new(),
                    errors: vec![],
                }
            };

            // Run lint (unless skipped)
            let lint_result = if !self.config.skip_lint {
                self.run_lint().await?
            } else {
                LintResult {
                    passed: true,
                    output: String::new(),
                    errors: vec![],
                }
            };

            // Check if all passed
            if test_result.passed && lint_result.passed {
                println!("✓ All checks passed after {} iteration(s)", iterations);
                return Ok(BuildResult {
                    success: true,
                    iterations,
                    errors: all_errors,
                    warnings: all_warnings,
                    evidence_files: vec![],
                    output: last_output,
                });
            }

            // Strict mode - fail on first error
            if self.config.strict && (!test_result.passed || !lint_result.passed) {
                all_errors.extend(test_result.errors.clone());
                all_errors.extend(lint_result.errors.clone());

                return Ok(BuildResult {
                    success: false,
                    iterations,
                    errors: all_errors,
                    warnings: all_warnings,
                    evidence_files: vec![],
                    output: format!(
                        "Strict mode: Failing on first error\nTests: {:?}\nLint: {:?}",
                        test_result.errors, lint_result.errors
                    ),
                });
            }

            iterations += 1;
            println!(
                "→ Iteration {}: Analyzing failures and applying fixes...",
                iterations
            );

            // Build fix prompt
            let fix_prompt = self.build_fix_prompt(&test_result, &lint_result);

            // Apply fix
            let response = provider.execute(&fix_prompt, &context).await?;
            last_output = response.output.clone();

            // Apply guardrails gate to fix changes
            if !self.gatekeep_changes(iterations).await? {
                return Ok(BuildResult {
                    success: false,
                    iterations,
                    errors: vec!["Fix denied by guardrails".to_string()],
                    warnings: all_warnings.clone(),
                    evidence_files: vec![],
                    output: "Guardrails denied the fix".to_string(),
                });
            }

            // Capture evidence
            self.evidence.capture_build_log(&last_output).await?;
        }

        Ok(BuildResult {
            success: false,
            iterations,
            errors: all_errors,
            warnings: all_warnings,
            evidence_files: vec![],
            output: last_output,
        })
    }

    /// Run tests and capture results
    async fn run_tests(&self) -> Result<TestResult> {
        // Try common test commands
        let test_commands = vec![
            ("cargo", vec!["test"]),
            ("npm", vec!["test"]),
            ("pytest", vec![]),
            ("go", vec!["test", "./..."]),
        ];

        for (cmd, args) in test_commands {
            if which::which(cmd).is_ok() {
                let output = Command::new(cmd).args(&args).output();

                if let Ok(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let full_output = format!("{}\n{}", stdout, stderr);

                    self.evidence.capture_test_log(&full_output).await?;

                    return Ok(TestResult {
                        passed: output.status.success(),
                        output: full_output.clone(),
                        errors: if output.status.success() {
                            vec![]
                        } else {
                            vec![full_output]
                        },
                    });
                }
            }
        }

        Ok(TestResult {
            passed: true,
            output: "No test framework detected".to_string(),
            errors: vec![],
        })
    }

    /// Run lint and capture results
    async fn run_lint(&self) -> Result<LintResult> {
        // Try common lint commands
        let lint_commands = vec![
            ("cargo", vec!["clippy", "--", "-D", "warnings"]),
            ("npm", vec!["run", "lint"]),
            ("eslint", vec!["."]),
            ("pylint", vec!["."]),
        ];

        for (cmd, args) in lint_commands {
            if which::which(cmd).is_ok() {
                let output = Command::new(cmd).args(&args).output();

                if let Ok(output) = output {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let full_output = format!("{}\n{}", stdout, stderr);

                    self.evidence.capture_lint_log(&full_output).await?;

                    return Ok(LintResult {
                        passed: output.status.success(),
                        output: full_output.clone(),
                        errors: if output.status.success() {
                            vec![]
                        } else {
                            vec![full_output]
                        },
                    });
                }
            }
        }

        Ok(LintResult {
            passed: true,
            output: "No lint tool detected".to_string(),
            errors: vec![],
        })
    }

    /// Build a prompt to fix errors
    fn build_fix_prompt(&self, test_result: &TestResult, lint_result: &LintResult) -> String {
        let mut prompt = String::from("Fix the following errors:\n\n");

        if !test_result.passed {
            prompt.push_str("## Test Failures:\n");
            prompt.push_str(&test_result.output);
            prompt.push_str("\n\n");
        }

        if !lint_result.passed {
            prompt.push_str("## Lint Errors:\n");
            prompt.push_str(&lint_result.output);
            prompt.push_str("\n\n");
        }

        prompt.push_str("Please fix these issues and ensure tests and lint pass.");
        prompt
    }

    /// Apply guardrails gate to changes made by provider
    /// Returns true if changes are approved, false if denied
    async fn gatekeep_changes(&self, iteration: u32) -> Result<bool> {
        // Load state to check if guardrails enabled
        let state = workspace::load_state().await?;
        if !state.guardrails.enabled {
            return Ok(true); // Guardrails disabled, auto-approve
        }

        // Get current git diff (unstaged changes)
        let diff_output = Command::new("git")
            .args(["diff", "--no-color"])
            .output()?;

        let diff = String::from_utf8_lossy(&diff_output.stdout).to_string();
        if diff.trim().is_empty() {
            // No changes to gate
            return Ok(true);
        }

        // Classify the diff
        let classifier = RiskClassifier::new();
        let classification = classifier.classify(&diff);

        // Create capsule
        let capsule_id = generate_capsule_id();
        let meta = CapsuleMeta::new(
            capsule_id,
            classification.risk,
            classification.reasons,
            classification.touched_files,
            classification.public_surface_changes,
        )
        .with_why(format!("Auto-generated by iteration {}", iteration))
        .with_provider(&self.config.provider);

        let mut capsule = Capsule::new(meta, diff, self.session_id.clone());

        // Save capsule for audit
        capsule.save().await?;

        // Run through approval gate
        let logger = AuditLogger::new(&self.session_id);
        let gate = ApprovalGate::new(&state.guardrails, &logger);
        let approved = gate.process(&mut capsule).await?;

        if approved {
            // Update capsule with approval
            capsule.save().await?;
            gate.log_applied(&capsule).await?;
        }

        Ok(approved)
    }
}

#[derive(Debug)]
struct TestResult {
    passed: bool,
    output: String,
    errors: Vec<String>,
}

#[derive(Debug)]
struct LintResult {
    passed: bool,
    output: String,
    errors: Vec<String>,
}
