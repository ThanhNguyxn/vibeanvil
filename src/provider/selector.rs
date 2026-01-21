//! Automatic Provider Selection
//!
//! Selects the best provider based on task type, requirements, and availability.
//! Uses the capability matrix to match tasks with provider capabilities.



use super::capability::{Capability, CapabilityMatrix, ProviderProfile};

/// Task type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskType {
    /// Generate new code (implement feature)
    CodeGeneration,
    /// Review existing code
    CodeReview,
    /// Refactor/improve code
    Refactoring,
    /// Fix a bug
    BugFix,
    /// Write tests
    TestWriting,
    /// Write documentation
    Documentation,
    /// Architecture design
    ArchitectureDesign,
    /// Implementation planning
    Planning,
    /// Debug an issue
    Debugging,
    /// Multi-file changes
    MultiFileEdit,
    /// Shell/terminal task
    ShellTask,
    /// Simple question/explanation
    Explanation,
}

impl TaskType {
    /// Get required capabilities for this task type
    pub fn required_capabilities(&self) -> Vec<Capability> {
        match self {
            TaskType::CodeGeneration => vec![
                Capability::CodeGeneration,
                Capability::Streaming,
            ],
            TaskType::CodeReview => vec![
                Capability::CodeReview,
                Capability::CodeExplanation,
            ],
            TaskType::Refactoring => vec![
                Capability::Refactoring,
                Capability::CodeGeneration,
            ],
            TaskType::BugFix => vec![
                Capability::BugFixing,
                Capability::Debugging,
                Capability::CodeGeneration,
            ],
            TaskType::TestWriting => vec![
                Capability::TestGeneration,
                Capability::CodeGeneration,
            ],
            TaskType::Documentation => vec![
                Capability::Documentation,
            ],
            TaskType::ArchitectureDesign => vec![
                Capability::Architecture,
                Capability::Planning,
            ],
            TaskType::Planning => vec![
                Capability::Planning,
                Capability::Architecture,
            ],
            TaskType::Debugging => vec![
                Capability::Debugging,
                Capability::CodeExplanation,
            ],
            TaskType::MultiFileEdit => vec![
                Capability::MultiFile,
                Capability::CodeGeneration,
                Capability::Agentic,
            ],
            TaskType::ShellTask => vec![
                Capability::ShellCommands,
                Capability::Agentic,
            ],
            TaskType::Explanation => vec![
                Capability::CodeExplanation,
            ],
        }
    }
    
    /// Get preferred capabilities (nice to have)
    pub fn preferred_capabilities(&self) -> Vec<Capability> {
        match self {
            TaskType::CodeGeneration => vec![
                Capability::Agentic,
                Capability::MultiFile,
            ],
            TaskType::CodeReview => vec![
                Capability::Architecture,
            ],
            TaskType::Refactoring => vec![
                Capability::MultiFile,
                Capability::Agentic,
            ],
            TaskType::BugFix => vec![
                Capability::Agentic,
                Capability::ShellCommands,
            ],
            TaskType::TestWriting => vec![
                Capability::MultiFile,
            ],
            TaskType::Documentation => vec![
                Capability::CodeExplanation,
            ],
            TaskType::ArchitectureDesign => vec![
                Capability::MultiFile,
                Capability::Agentic,
            ],
            TaskType::Planning => vec![
                Capability::MultiFile,
            ],
            TaskType::Debugging => vec![
                Capability::ShellCommands,
                Capability::Agentic,
            ],
            TaskType::MultiFileEdit => vec![
                Capability::CodeSearch,
            ],
            TaskType::ShellTask => vec![
                Capability::Background,
            ],
            TaskType::Explanation => vec![
                Capability::Streaming,
            ],
        }
    }
    
    /// Infer task type from description/prompt
    pub fn infer(description: &str) -> Self {
        let lower = description.to_lowercase();
        
        // Check for explicit patterns
        if lower.contains("review") || lower.contains("check") || lower.contains("audit") {
            return TaskType::CodeReview;
        }
        if lower.contains("refactor") || lower.contains("improve") || lower.contains("optimize") {
            return TaskType::Refactoring;
        }
        if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
            return TaskType::BugFix;
        }
        if lower.contains("test") || lower.contains("spec") {
            return TaskType::TestWriting;
        }
        if lower.contains("doc") || lower.contains("readme") || lower.contains("comment") {
            return TaskType::Documentation;
        }
        if lower.contains("architect") || lower.contains("design") || lower.contains("structure") {
            return TaskType::ArchitectureDesign;
        }
        if lower.contains("plan") || lower.contains("roadmap") || lower.contains("breakdown") {
            return TaskType::Planning;
        }
        if lower.contains("debug") || lower.contains("trace") || lower.contains("investigate") {
            return TaskType::Debugging;
        }
        if lower.contains("multiple file") || lower.contains("across") || lower.contains("codebase") {
            return TaskType::MultiFileEdit;
        }
        if lower.contains("run") || lower.contains("execute") || lower.contains("command") || lower.contains("shell") {
            return TaskType::ShellTask;
        }
        if lower.contains("explain") || lower.contains("what is") || lower.contains("how does") {
            return TaskType::Explanation;
        }
        
        // Default to code generation
        TaskType::CodeGeneration
    }
}

/// Selection criteria
#[derive(Debug, Clone)]
pub struct SelectionCriteria {
    /// Task type
    pub task_type: TaskType,
    /// Prefer lower cost
    pub prefer_low_cost: bool,
    /// Prefer lower latency
    pub prefer_low_latency: bool,
    /// Require agentic capabilities
    pub require_agentic: bool,
    /// Require local/offline
    pub require_local: bool,
    /// Exclude specific providers
    pub exclude: Vec<String>,
    /// Minimum capability threshold (1-10)
    pub min_capability: u8,
}

impl Default for SelectionCriteria {
    fn default() -> Self {
        Self {
            task_type: TaskType::CodeGeneration,
            prefer_low_cost: false,
            prefer_low_latency: false,
            require_agentic: false,
            require_local: false,
            exclude: Vec::new(),
            min_capability: 6,
        }
    }
}

impl SelectionCriteria {
    pub fn for_task(task_type: TaskType) -> Self {
        Self {
            task_type,
            ..Default::default()
        }
    }
    
    pub fn agentic(mut self) -> Self {
        self.require_agentic = true;
        self
    }
    
    pub fn local(mut self) -> Self {
        self.require_local = true;
        self
    }
    
    pub fn low_cost(mut self) -> Self {
        self.prefer_low_cost = true;
        self
    }
    
    pub fn fast(mut self) -> Self {
        self.prefer_low_latency = true;
        self
    }
    
    pub fn exclude_provider(mut self, name: &str) -> Self {
        self.exclude.push(name.to_string());
        self
    }
}

/// Provider selector
pub struct ProviderSelector {
    matrix: CapabilityMatrix,
    available: Vec<String>,
}

impl ProviderSelector {
    /// Create new selector with default matrix
    pub fn new() -> Self {
        Self {
            matrix: CapabilityMatrix::build_default(),
            available: Vec::new(),
        }
    }
    
    /// Create with custom matrix
    pub fn with_matrix(matrix: CapabilityMatrix) -> Self {
        Self {
            matrix,
            available: Vec::new(),
        }
    }
    
    /// Set available providers (empty = all available)
    pub fn set_available(&mut self, providers: Vec<String>) {
        self.available = providers;
    }
    
    /// Check if provider is available
    fn is_available(&self, name: &str) -> bool {
        self.available.is_empty() || self.available.contains(&name.to_string())
    }
    
    /// Select best provider for criteria
    pub fn select(&self, criteria: &SelectionCriteria) -> Option<&ProviderProfile> {
        let required = criteria.task_type.required_capabilities();
        let preferred = criteria.task_type.preferred_capabilities();
        
        // Get candidates
        let candidates: Vec<_> = self.matrix
            .list()
            .into_iter()
            .filter(|p| {
                // Check availability
                if !self.is_available(&p.name) {
                    return false;
                }
                
                // Check exclusions
                if criteria.exclude.contains(&p.name) {
                    return false;
                }
                
                // Check agentic requirement
                if criteria.require_agentic && !p.has_capability(Capability::Agentic, 7) {
                    return false;
                }
                
                // Check local requirement
                if criteria.require_local && !p.tags.contains(&"local".to_string()) {
                    return false;
                }
                
                // Check minimum capability for required caps
                for cap in &required {
                    if p.capability_score(*cap) < criteria.min_capability {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        // Score candidates
        let scored: Vec<_> = candidates.iter().map(|p| {
            let mut score: i32 = 0;
            
            // Base score from required capabilities
            for cap in &required {
                score += p.capability_score(*cap) as i32 * 10;
            }
            
            // Bonus from preferred capabilities
            for cap in &preferred {
                score += p.capability_score(*cap) as i32 * 3;
            }
            
            // Tier bonus (lower tier = better)
            score += (5 - p.tier as i32) * 20;
            
            // Cost penalty
            if criteria.prefer_low_cost && p.cost_per_1k > 0.0 {
                score -= (p.cost_per_1k * 100.0) as i32;
            }
            
            // Latency penalty  
            if criteria.prefer_low_latency {
                score -= (p.latency_ms / 100) as i32;
            }
            
            (*p, score)
        }).collect();
        
        // Find best
        scored.into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(p, _)| p)
    }
    
    /// Select multiple providers (for fallback chain)
    pub fn select_chain(&self, criteria: &SelectionCriteria, count: usize) -> Vec<&ProviderProfile> {
        let required = criteria.task_type.required_capabilities();
        
        // Get all matching providers
        let candidates: Vec<_> = self.matrix
            .list()
            .into_iter()
            .filter(|p| {
                if !self.is_available(&p.name) {
                    return false;
                }
                if criteria.exclude.contains(&p.name) {
                    return false;
                }
                if criteria.require_agentic && !p.has_capability(Capability::Agentic, 7) {
                    return false;
                }
                if criteria.require_local && !p.tags.contains(&"local".to_string()) {
                    return false;
                }
                true
            })
            .collect();
        
        // Score and sort
        let mut scored: Vec<_> = candidates.iter().map(|p| {
            let mut score: i32 = p.match_score(&required) as i32;
            score += (5 - p.tier as i32) * 10;
            (*p, score)
        }).collect();
        
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        
        scored.into_iter()
            .take(count)
            .map(|(p, _)| p)
            .collect()
    }
    
    /// Quick select for task description
    pub fn for_task(&self, description: &str) -> Option<&ProviderProfile> {
        let task_type = TaskType::infer(description);
        let criteria = SelectionCriteria::for_task(task_type);
        self.select(&criteria)
    }
    
    /// Get recommendations with explanations
    pub fn recommend(&self, description: &str, count: usize) -> Vec<ProviderRecommendation> {
        let task_type = TaskType::infer(description);
        let criteria = SelectionCriteria::for_task(task_type);
        let chain = self.select_chain(&criteria, count);
        
        chain.into_iter().enumerate().map(|(i, p)| {
            let reason = if i == 0 {
                format!(
                    "Best match for {:?} task with {}/10 capability score",
                    task_type,
                    p.match_score(&criteria.task_type.required_capabilities()) / criteria.task_type.required_capabilities().len() as u32
                )
            } else {
                format!("Alternative option (Tier {})", p.tier)
            };
            
            ProviderRecommendation {
                name: p.name.clone(),
                tier: p.tier,
                reason,
                capabilities: criteria.task_type.required_capabilities()
                    .iter()
                    .map(|c| (c.display_name().to_string(), p.capability_score(*c)))
                    .collect(),
            }
        }).collect()
    }
}

impl Default for ProviderSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider recommendation with explanation
#[derive(Debug, Clone)]
pub struct ProviderRecommendation {
    /// Provider name
    pub name: String,
    /// Provider tier
    pub tier: u8,
    /// Reason for recommendation
    pub reason: String,
    /// Relevant capability scores
    pub capabilities: Vec<(String, u8)>,
}

impl ProviderRecommendation {
    pub fn display(&self) -> String {
        let mut output = format!("🎯 {} (Tier {})\n", self.name, self.tier);
        output.push_str(&format!("   {}\n", self.reason));
        output.push_str("   Capabilities: ");
        for (cap, score) in &self.capabilities {
            output.push_str(&format!("{}: {}/10, ", cap, score));
        }
        output.pop(); // Remove trailing comma
        output.pop();
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_infer() {
        assert_eq!(TaskType::infer("review this code"), TaskType::CodeReview);
        assert_eq!(TaskType::infer("fix the bug in login"), TaskType::BugFix);
        assert_eq!(TaskType::infer("refactor the auth module"), TaskType::Refactoring);
        assert_eq!(TaskType::infer("write tests for api"), TaskType::TestWriting);
        assert_eq!(TaskType::infer("add documentation"), TaskType::Documentation);
        assert_eq!(TaskType::infer("implement new feature"), TaskType::CodeGeneration);
    }

    #[test]
    fn test_task_required_capabilities() {
        let caps = TaskType::CodeReview.required_capabilities();
        assert!(caps.contains(&Capability::CodeReview));
        
        let caps = TaskType::BugFix.required_capabilities();
        assert!(caps.contains(&Capability::BugFixing));
        assert!(caps.contains(&Capability::Debugging));
    }

    #[test]
    fn test_selector_select() {
        let selector = ProviderSelector::new();
        let criteria = SelectionCriteria::for_task(TaskType::CodeGeneration);
        
        let best = selector.select(&criteria);
        assert!(best.is_some());
        
        let provider = best.unwrap();
        assert!(provider.has_capability(Capability::CodeGeneration, 6));
    }

    #[test]
    fn test_selector_agentic() {
        let selector = ProviderSelector::new();
        let criteria = SelectionCriteria::for_task(TaskType::MultiFileEdit)
            .agentic();
        
        let best = selector.select(&criteria);
        assert!(best.is_some());
        assert!(best.unwrap().has_capability(Capability::Agentic, 7));
    }

    #[test]
    fn test_selector_local() {
        let selector = ProviderSelector::new();
        let criteria = SelectionCriteria::for_task(TaskType::CodeGeneration)
            .local();
        
        let best = selector.select(&criteria);
        assert!(best.is_some());
        assert!(best.unwrap().tags.contains(&"local".to_string()));
    }

    #[test]
    fn test_selector_exclude() {
        let selector = ProviderSelector::new();
        let criteria = SelectionCriteria::for_task(TaskType::CodeGeneration)
            .exclude_provider("claude_code");
        
        let best = selector.select(&criteria);
        assert!(best.is_some());
        assert_ne!(best.unwrap().name, "claude_code");
    }

    #[test]
    fn test_selector_chain() {
        let selector = ProviderSelector::new();
        let criteria = SelectionCriteria::for_task(TaskType::CodeGeneration);
        
        let chain = selector.select_chain(&criteria, 3);
        assert_eq!(chain.len(), 3);
    }

    #[test]
    fn test_selector_for_task() {
        let selector = ProviderSelector::new();
        
        let provider = selector.for_task("review my code changes");
        assert!(provider.is_some());
        
        let provider = selector.for_task("fix the authentication bug");
        assert!(provider.is_some());
    }

    #[test]
    fn test_recommend() {
        let selector = ProviderSelector::new();
        let recs = selector.recommend("implement a REST API", 3);
        
        assert_eq!(recs.len(), 3);
        assert!(!recs[0].reason.is_empty());
    }

    #[test]
    fn test_recommendation_display() {
        let rec = ProviderRecommendation {
            name: "test_provider".to_string(),
            tier: 1,
            reason: "Best for testing".to_string(),
            capabilities: vec![
                ("Code Generation".to_string(), 9),
                ("Testing".to_string(), 8),
            ],
        };
        
        let display = rec.display();
        assert!(display.contains("test_provider"));
        assert!(display.contains("Tier 1"));
        assert!(display.contains("9/10"));
    }
}
