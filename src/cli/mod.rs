//! CLI command definitions and subcommand handlers

use clap::{Parser, Subcommand, ValueEnum};

pub mod blueprint;
pub mod brain;
pub mod build;
pub mod contract;
pub mod doctor;
pub mod harvest;
pub mod init;
pub mod intake;
pub mod log;
pub mod plan;
pub mod review;
pub mod ship;
pub mod snapshot;
pub mod status;
pub mod ui;
pub mod update;
pub mod wizard;

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
    Ensure,

    /// Show brain pack statistics
    Stats,

    /// Search the brain pack
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },

    /// Export the brain pack
    Export {
        /// Export format
        #[arg(value_enum, default_value = "jsonl")]
        format: ExportFormat,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Include anonymized source IDs (default: true for debugging)
        #[arg(long, default_value = "true")]
        include_source_ids: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ExportFormat {
    /// JSON Lines format
    Jsonl,
    /// Markdown format
    Md,
}
