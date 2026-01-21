//! Provider Capability Matrix
//!
//! Defines and compares capabilities across all providers.
//! Used for automatic provider selection based on task requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Can generate code
    CodeGeneration,
    /// Can review code
    CodeReview,
    /// Can explain code
    CodeExplanation,
    /// Can refactor code
    Refactoring,
    /// Can fix bugs
    BugFixing,
    /// Can write tests
    TestGeneration,
    /// Can write documentation
    Documentation,
    /// Can analyze architecture
    Architecture,
    /// Can plan implementation
    Planning,
    /// Can debug issues
    Debugging,
    /// Can handle shell/terminal
    ShellCommands,
    /// Can work with multiple files
    MultiFile,
    /// Can search codebase
    CodeSearch,
    /// Can access web/internet
    WebAccess,
    /// Can handle images/vision
    Vision,
    /// Can stream responses
    Streaming,
    /// Supports interactive mode
    Interactive,
    /// Has agentic capabilities
    Agentic,
    /// Can run in background
    Background,
    /// Supports MCP protocol
    McpSupport,
}

impl Capability {
    /// Get all capabilities
    pub fn all() -> Vec<Capability> {
        vec![
            Capability::CodeGeneration,
            Capability::CodeReview,
            Capability::CodeExplanation,
            Capability::Refactoring,
            Capability::BugFixing,
            Capability::TestGeneration,
            Capability::Documentation,
            Capability::Architecture,
            Capability::Planning,
            Capability::Debugging,
            Capability::ShellCommands,
            Capability::MultiFile,
            Capability::CodeSearch,
            Capability::WebAccess,
            Capability::Vision,
            Capability::Streaming,
            Capability::Interactive,
            Capability::Agentic,
            Capability::Background,
            Capability::McpSupport,
        ]
    }

    /// Get capability display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Capability::CodeGeneration => "Code Generation",
            Capability::CodeReview => "Code Review",
            Capability::CodeExplanation => "Code Explanation",
            Capability::Refactoring => "Refactoring",
            Capability::BugFixing => "Bug Fixing",
            Capability::TestGeneration => "Test Generation",
            Capability::Documentation => "Documentation",
            Capability::Architecture => "Architecture",
            Capability::Planning => "Planning",
            Capability::Debugging => "Debugging",
            Capability::ShellCommands => "Shell Commands",
            Capability::MultiFile => "Multi-File",
            Capability::CodeSearch => "Code Search",
            Capability::WebAccess => "Web Access",
            Capability::Vision => "Vision",
            Capability::Streaming => "Streaming",
            Capability::Interactive => "Interactive",
            Capability::Agentic => "Agentic",
            Capability::Background => "Background",
            Capability::McpSupport => "MCP Support",
        }
    }

    /// Get capability short code for matrix display
    pub fn short_code(&self) -> &'static str {
        match self {
            Capability::CodeGeneration => "GEN",
            Capability::CodeReview => "REV",
            Capability::CodeExplanation => "EXP",
            Capability::Refactoring => "REF",
            Capability::BugFixing => "FIX",
            Capability::TestGeneration => "TST",
            Capability::Documentation => "DOC",
            Capability::Architecture => "ARC",
            Capability::Planning => "PLN",
            Capability::Debugging => "DBG",
            Capability::ShellCommands => "SHL",
            Capability::MultiFile => "MUL",
            Capability::CodeSearch => "SRC",
            Capability::WebAccess => "WEB",
            Capability::Vision => "VIS",
            Capability::Streaming => "STR",
            Capability::Interactive => "INT",
            Capability::Agentic => "AGT",
            Capability::Background => "BKG",
            Capability::McpSupport => "MCP",
        }
    }
}

/// Provider capability score (0-10)
pub type CapabilityScore = u8;

/// Provider capability profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    /// Provider name
    pub name: String,
    /// Provider tier (1=premium, 2=standard, 3=basic, 4=local)
    pub tier: u8,
    /// Capability scores (0=no, 1-5=limited, 6-10=good/excellent)
    pub capabilities: HashMap<Capability, CapabilityScore>,
    /// Estimated cost per 1K tokens (input + output)
    pub cost_per_1k: f32,
    /// Typical response latency in ms
    pub latency_ms: u32,
    /// Maximum context window size
    pub context_window: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl ProviderProfile {
    /// Create new provider profile
    pub fn new(name: &str, tier: u8) -> Self {
        Self {
            name: name.to_string(),
            tier,
            capabilities: HashMap::new(),
            cost_per_1k: 0.0,
            latency_ms: 1000,
            context_window: 8000,
            tags: Vec::new(),
        }
    }

    /// Add capability with score
    pub fn with_capability(mut self, cap: Capability, score: u8) -> Self {
        self.capabilities.insert(cap, score.min(10));
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost: f32) -> Self {
        self.cost_per_1k = cost;
        self
    }

    /// Set latency
    pub fn with_latency(mut self, ms: u32) -> Self {
        self.latency_ms = ms;
        self
    }

    /// Set context window
    pub fn with_context(mut self, tokens: u32) -> Self {
        self.context_window = tokens;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if provider has capability (score >= threshold)
    pub fn has_capability(&self, cap: Capability, threshold: u8) -> bool {
        self.capabilities.get(&cap).copied().unwrap_or(0) >= threshold
    }

    /// Get capability score
    pub fn capability_score(&self, cap: Capability) -> u8 {
        self.capabilities.get(&cap).copied().unwrap_or(0)
    }

    /// Calculate overall score for a set of required capabilities
    pub fn match_score(&self, required: &[Capability]) -> u32 {
        let mut score: u32 = 0;
        for cap in required {
            score += self.capability_score(*cap) as u32;
        }
        score
    }
}

/// Provider capability matrix containing all provider profiles
#[derive(Debug, Clone)]
pub struct CapabilityMatrix {
    providers: HashMap<String, ProviderProfile>,
}

impl CapabilityMatrix {
    /// Create empty matrix
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Build the default capability matrix with all providers
    pub fn build_default() -> Self {
        let mut matrix = Self::new();

        // === Tier 1: Premium AI Coding Agents ===

        // Claude Code - Best for agentic tasks
        matrix.add(
            ProviderProfile::new("claude_code", 1)
                .with_capability(Capability::CodeGeneration, 10)
                .with_capability(Capability::CodeReview, 10)
                .with_capability(Capability::CodeExplanation, 10)
                .with_capability(Capability::Refactoring, 10)
                .with_capability(Capability::BugFixing, 10)
                .with_capability(Capability::TestGeneration, 9)
                .with_capability(Capability::Documentation, 9)
                .with_capability(Capability::Architecture, 10)
                .with_capability(Capability::Planning, 10)
                .with_capability(Capability::Debugging, 10)
                .with_capability(Capability::ShellCommands, 10)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::CodeSearch, 10)
                .with_capability(Capability::WebAccess, 8)
                .with_capability(Capability::Vision, 10)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 10)
                .with_capability(Capability::Background, 10)
                .with_capability(Capability::McpSupport, 10)
                .with_cost(0.015)
                .with_latency(800)
                .with_context(200000)
                .with_tag("agentic")
                .with_tag("premium"),
        );

        // GitHub Copilot Agent
        matrix.add(
            ProviderProfile::new("github_copilot_agent", 1)
                .with_capability(Capability::CodeGeneration, 10)
                .with_capability(Capability::CodeReview, 9)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::BugFixing, 10)
                .with_capability(Capability::TestGeneration, 9)
                .with_capability(Capability::Documentation, 8)
                .with_capability(Capability::Architecture, 9)
                .with_capability(Capability::Planning, 9)
                .with_capability(Capability::Debugging, 10)
                .with_capability(Capability::ShellCommands, 9)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::CodeSearch, 10)
                .with_capability(Capability::Vision, 8)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 10)
                .with_capability(Capability::Background, 10)
                .with_capability(Capability::McpSupport, 10)
                .with_cost(0.01)
                .with_latency(600)
                .with_context(128000)
                .with_tag("agentic")
                .with_tag("github"),
        );

        // Cursor Agent
        matrix.add(
            ProviderProfile::new("cursor_agent", 1)
                .with_capability(Capability::CodeGeneration, 10)
                .with_capability(Capability::CodeReview, 9)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Refactoring, 10)
                .with_capability(Capability::BugFixing, 10)
                .with_capability(Capability::TestGeneration, 9)
                .with_capability(Capability::Documentation, 8)
                .with_capability(Capability::Architecture, 9)
                .with_capability(Capability::Planning, 9)
                .with_capability(Capability::Debugging, 10)
                .with_capability(Capability::ShellCommands, 9)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::CodeSearch, 10)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 10)
                .with_cost(0.02)
                .with_latency(700)
                .with_context(128000)
                .with_tag("agentic")
                .with_tag("ide"),
        );

        // Windsurf Cascade
        matrix.add(
            ProviderProfile::new("windsurf", 1)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 9)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::BugFixing, 9)
                .with_capability(Capability::TestGeneration, 8)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::CodeSearch, 9)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 9)
                .with_cost(0.015)
                .with_latency(800)
                .with_context(128000)
                .with_tag("agentic")
                .with_tag("ide"),
        );

        // Augment
        matrix.add(
            ProviderProfile::new("augment", 1)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 9)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::CodeSearch, 10)
                .with_capability(Capability::Architecture, 9)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 9)
                .with_cost(0.015)
                .with_latency(900)
                .with_context(100000)
                .with_tag("agentic")
                .with_tag("enterprise"),
        );

        // Amp (Sourcegraph)
        matrix.add(
            ProviderProfile::new("amp", 1)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeSearch, 10)
                .with_capability(Capability::MultiFile, 10)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Architecture, 9)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_capability(Capability::Agentic, 9)
                .with_cost(0.015)
                .with_latency(800)
                .with_context(100000)
                .with_tag("agentic")
                .with_tag("search"),
        );

        // === Tier 2: Standard AI Coding Tools ===

        // Aider
        matrix.add(
            ProviderProfile::new("aider", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::BugFixing, 8)
                .with_capability(Capability::MultiFile, 9)
                .with_capability(Capability::ShellCommands, 8)
                .with_capability(Capability::Streaming, 9)
                .with_capability(Capability::Interactive, 9)
                .with_capability(Capability::Agentic, 8)
                .with_cost(0.01)
                .with_latency(1000)
                .with_context(128000)
                .with_tag("cli")
                .with_tag("open-source"),
        );

        // Cline
        matrix.add(
            ProviderProfile::new("cline", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::MultiFile, 9)
                .with_capability(Capability::ShellCommands, 9)
                .with_capability(Capability::Streaming, 9)
                .with_capability(Capability::Interactive, 9)
                .with_capability(Capability::Agentic, 9)
                .with_cost(0.01)
                .with_latency(900)
                .with_context(100000)
                .with_tag("vscode")
                .with_tag("open-source"),
        );

        // Roo Code
        matrix.add(
            ProviderProfile::new("roo_code", 2)
                .with_capability(Capability::CodeGeneration, 8)
                .with_capability(Capability::CodeReview, 8)
                .with_capability(Capability::MultiFile, 8)
                .with_capability(Capability::Streaming, 9)
                .with_capability(Capability::Interactive, 9)
                .with_cost(0.01)
                .with_latency(1000)
                .with_context(100000)
                .with_tag("vscode"),
        );

        // Zed AI
        matrix.add(
            ProviderProfile::new("zed_ai", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 8)
                .with_capability(Capability::Refactoring, 8)
                .with_capability(Capability::MultiFile, 9)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 9)
                .with_cost(0.01)
                .with_latency(700)
                .with_context(100000)
                .with_tag("ide")
                .with_tag("fast"),
        );

        // Codex CLI
        matrix.add(
            ProviderProfile::new("codex_cli", 2)
                .with_capability(Capability::CodeGeneration, 8)
                .with_capability(Capability::ShellCommands, 9)
                .with_capability(Capability::MultiFile, 8)
                .with_capability(Capability::Agentic, 8)
                .with_cost(0.008)
                .with_latency(1000)
                .with_context(100000)
                .with_tag("cli")
                .with_tag("open-source"),
        );

        // Claude API Direct
        matrix.add(
            ProviderProfile::new("claude_api", 2)
                .with_capability(Capability::CodeGeneration, 10)
                .with_capability(Capability::CodeReview, 10)
                .with_capability(Capability::CodeExplanation, 10)
                .with_capability(Capability::Documentation, 10)
                .with_capability(Capability::Architecture, 10)
                .with_capability(Capability::Planning, 10)
                .with_capability(Capability::Vision, 10)
                .with_capability(Capability::Streaming, 10)
                .with_cost(0.015)
                .with_latency(800)
                .with_context(200000)
                .with_tag("api")
                .with_tag("anthropic"),
        );

        // OpenAI API
        matrix.add(
            ProviderProfile::new("openai_api", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 9)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Documentation, 9)
                .with_capability(Capability::Architecture, 9)
                .with_capability(Capability::Vision, 10)
                .with_capability(Capability::Streaming, 10)
                .with_cost(0.01)
                .with_latency(700)
                .with_context(128000)
                .with_tag("api")
                .with_tag("openai"),
        );

        // Gemini API
        matrix.add(
            ProviderProfile::new("gemini_api", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 8)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Documentation, 9)
                .with_capability(Capability::Vision, 10)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::WebAccess, 9)
                .with_cost(0.005)
                .with_latency(600)
                .with_context(1000000)
                .with_tag("api")
                .with_tag("google"),
        );

        // Copilot Chat
        matrix.add(
            ProviderProfile::new("copilot_chat", 2)
                .with_capability(Capability::CodeGeneration, 9)
                .with_capability(Capability::CodeReview, 8)
                .with_capability(Capability::CodeExplanation, 9)
                .with_capability(Capability::Streaming, 10)
                .with_capability(Capability::Interactive, 10)
                .with_cost(0.0)
                .with_latency(500)
                .with_context(32000)
                .with_tag("vscode")
                .with_tag("github"),
        );

        // === Tier 3: Basic/Specialized Tools ===

        // Goose
        matrix.add(
            ProviderProfile::new("goose", 3)
                .with_capability(Capability::CodeGeneration, 7)
                .with_capability(Capability::ShellCommands, 8)
                .with_capability(Capability::Agentic, 7)
                .with_cost(0.01)
                .with_latency(1200)
                .with_context(50000)
                .with_tag("cli"),
        );

        // Amazon Q
        matrix.add(
            ProviderProfile::new("amazon_q", 3)
                .with_capability(Capability::CodeGeneration, 8)
                .with_capability(Capability::CodeReview, 7)
                .with_capability(Capability::Documentation, 8)
                .with_capability(Capability::Streaming, 8)
                .with_cost(0.0)
                .with_latency(1000)
                .with_context(64000)
                .with_tag("aws")
                .with_tag("enterprise"),
        );

        // Tabnine
        matrix.add(
            ProviderProfile::new("tabnine", 3)
                .with_capability(Capability::CodeGeneration, 7)
                .with_capability(Capability::Streaming, 9)
                .with_cost(0.0)
                .with_latency(200)
                .with_context(8000)
                .with_tag("autocomplete")
                .with_tag("fast"),
        );

        // Sourcery
        matrix.add(
            ProviderProfile::new("sourcery", 3)
                .with_capability(Capability::CodeReview, 8)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::BugFixing, 7)
                .with_cost(0.0)
                .with_latency(500)
                .with_context(16000)
                .with_tag("review")
                .with_tag("python"),
        );

        // Grit.io
        matrix.add(
            ProviderProfile::new("grit", 3)
                .with_capability(Capability::Refactoring, 9)
                .with_capability(Capability::MultiFile, 9)
                .with_cost(0.01)
                .with_latency(1000)
                .with_context(50000)
                .with_tag("migration")
                .with_tag("refactoring"),
        );

        // === Tier 4: Local/Offline Models ===

        // Ollama
        matrix.add(
            ProviderProfile::new("ollama", 4)
                .with_capability(Capability::CodeGeneration, 7)
                .with_capability(Capability::CodeExplanation, 7)
                .with_capability(Capability::Streaming, 9)
                .with_cost(0.0)
                .with_latency(2000)
                .with_context(32000)
                .with_tag("local")
                .with_tag("offline"),
        );

        // LMStudio
        matrix.add(
            ProviderProfile::new("lmstudio", 4)
                .with_capability(Capability::CodeGeneration, 7)
                .with_capability(Capability::CodeExplanation, 7)
                .with_capability(Capability::Streaming, 9)
                .with_cost(0.0)
                .with_latency(2000)
                .with_context(32000)
                .with_tag("local")
                .with_tag("offline"),
        );

        // Continue.dev
        matrix.add(
            ProviderProfile::new("continue_dev", 4)
                .with_capability(Capability::CodeGeneration, 8)
                .with_capability(Capability::CodeExplanation, 8)
                .with_capability(Capability::Streaming, 9)
                .with_capability(Capability::Interactive, 9)
                .with_cost(0.0)
                .with_latency(1500)
                .with_context(50000)
                .with_tag("vscode")
                .with_tag("open-source"),
        );

        // Human (manual review)
        matrix.add(
            ProviderProfile::new("human", 4)
                .with_capability(Capability::CodeReview, 10)
                .with_capability(Capability::Architecture, 10)
                .with_capability(Capability::Planning, 10)
                .with_capability(Capability::Debugging, 10)
                .with_cost(0.0)
                .with_latency(86400000)
                .with_context(999999)
                .with_tag("manual")
                .with_tag("review"),
        );

        matrix
    }

    /// Add provider profile
    pub fn add(&mut self, profile: ProviderProfile) {
        self.providers.insert(profile.name.clone(), profile);
    }

    /// Get provider profile
    pub fn get(&self, name: &str) -> Option<&ProviderProfile> {
        self.providers.get(name)
    }

    /// List all providers
    pub fn list(&self) -> Vec<&ProviderProfile> {
        let mut providers: Vec<_> = self.providers.values().collect();
        providers.sort_by(|a, b| a.tier.cmp(&b.tier).then_with(|| a.name.cmp(&b.name)));
        providers
    }

    /// Find providers with specific capability above threshold
    pub fn with_capability(&self, cap: Capability, threshold: u8) -> Vec<&ProviderProfile> {
        self.providers
            .values()
            .filter(|p| p.has_capability(cap, threshold))
            .collect()
    }

    /// Find best providers for a set of required capabilities
    pub fn find_best(&self, required: &[Capability], limit: usize) -> Vec<&ProviderProfile> {
        let mut scored: Vec<_> = self
            .providers
            .values()
            .map(|p| (p, p.match_score(required)))
            .filter(|(_, score)| *score > 0)
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(limit).map(|(p, _)| p).collect()
    }

    /// Find providers by tier
    pub fn by_tier(&self, tier: u8) -> Vec<&ProviderProfile> {
        self.providers.values().filter(|p| p.tier == tier).collect()
    }

    /// Find providers with specific tag
    pub fn with_tag(&self, tag: &str) -> Vec<&ProviderProfile> {
        self.providers
            .values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Generate capability matrix table
    pub fn to_table(&self) -> String {
        let mut output = String::new();
        let providers = self.list();

        // Header
        output.push_str("| Provider | Tier | ");
        let caps = Capability::all();
        for cap in &caps {
            output.push_str(&format!("{} | ", cap.short_code()));
        }
        output.push_str("\n");

        // Separator
        output.push_str("|---------|------|");
        for _ in &caps {
            output.push_str("----|");
        }
        output.push_str("\n");

        // Rows
        for provider in providers {
            output.push_str(&format!("| {} | {} | ", provider.name, provider.tier));
            for cap in &caps {
                let score = provider.capability_score(*cap);
                let symbol = if score == 0 {
                    "·"
                } else if score <= 5 {
                    "○"
                } else if score <= 8 {
                    "◐"
                } else {
                    "●"
                };
                output.push_str(&format!("{} | ", symbol));
            }
            output.push_str("\n");
        }

        // Legend
        output.push_str("\n");
        output.push_str(
            "Legend: ● = Excellent (9-10), ◐ = Good (6-8), ○ = Limited (1-5), · = None\n",
        );
        output.push_str("\nCapability Codes:\n");
        for cap in &caps {
            output.push_str(&format!(
                "  {} = {}\n",
                cap.short_code(),
                cap.display_name()
            ));
        }

        output
    }
}

impl Default for CapabilityMatrix {
    fn default() -> Self {
        Self::build_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_all() {
        let caps = Capability::all();
        assert_eq!(caps.len(), 20);
    }

    #[test]
    fn test_provider_profile() {
        let profile = ProviderProfile::new("test", 1)
            .with_capability(Capability::CodeGeneration, 9)
            .with_capability(Capability::CodeReview, 7)
            .with_cost(0.01)
            .with_latency(500);

        assert_eq!(profile.name, "test");
        assert_eq!(profile.tier, 1);
        assert!(profile.has_capability(Capability::CodeGeneration, 8));
        assert!(!profile.has_capability(Capability::CodeReview, 8));
        assert_eq!(profile.cost_per_1k, 0.01);
    }

    #[test]
    fn test_capability_matrix_default() {
        let matrix = CapabilityMatrix::build_default();
        assert!(matrix.providers.len() >= 20);
    }

    #[test]
    fn test_find_by_capability() {
        let matrix = CapabilityMatrix::build_default();
        let agentic = matrix.with_capability(Capability::Agentic, 9);
        assert!(!agentic.is_empty());

        for p in &agentic {
            assert!(p.has_capability(Capability::Agentic, 9));
        }
    }

    #[test]
    fn test_find_best() {
        let matrix = CapabilityMatrix::build_default();
        let required = vec![
            Capability::CodeGeneration,
            Capability::Agentic,
            Capability::MultiFile,
        ];

        let best = matrix.find_best(&required, 3);
        assert_eq!(best.len(), 3);
    }

    #[test]
    fn test_by_tier() {
        let matrix = CapabilityMatrix::build_default();
        let tier1 = matrix.by_tier(1);
        assert!(!tier1.is_empty());

        for p in tier1 {
            assert_eq!(p.tier, 1);
        }
    }

    #[test]
    fn test_with_tag() {
        let matrix = CapabilityMatrix::build_default();
        let agentic = matrix.with_tag("agentic");
        assert!(!agentic.is_empty());
    }

    #[test]
    fn test_to_table() {
        let matrix = CapabilityMatrix::build_default();
        let table = matrix.to_table();
        assert!(table.contains("| Provider |"));
        assert!(table.contains("claude_code"));
        assert!(table.contains("Legend:"));
    }

    #[test]
    fn test_match_score() {
        let profile = ProviderProfile::new("test", 1)
            .with_capability(Capability::CodeGeneration, 10)
            .with_capability(Capability::Agentic, 8);

        let score = profile.match_score(&[
            Capability::CodeGeneration,
            Capability::Agentic,
            Capability::Vision,
        ]);

        assert_eq!(score, 18); // 10 + 8 + 0
    }
}
