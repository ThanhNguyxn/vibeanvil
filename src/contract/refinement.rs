//! Refinement Phase Module
//! Helps clarify ambiguous requirements before planning.
//! Inspired by Speckit's refinement phase.

use serde::{Deserialize, Serialize};

/// A refinement question to clarify requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementQuestion {
    /// Question ID
    pub id: String,
    /// The question text
    pub question: String,
    /// Category (scope, tech, ux, security)
    pub category: String,
    /// Whether it's answered
    pub answered: bool,
    /// The answer (if provided)
    pub answer: Option<String>,
}

/// Common refinement questions by category
pub const COMMON_QUESTIONS: &[(&str, &str, &str)] = &[
    // Scope questions
    (
        "scope-1",
        "scope",
        "What are the must-have features for MVP?",
    ),
    ("scope-2", "scope", "What features can be deferred to v2?"),
    ("scope-3", "scope", "Are there any hard deadlines?"),
    // Technology questions
    (
        "tech-1",
        "tech",
        "What's the preferred programming language?",
    ),
    (
        "tech-2",
        "tech",
        "Are there existing systems to integrate with?",
    ),
    (
        "tech-3",
        "tech",
        "Any specific libraries or frameworks required?",
    ),
    // UX questions
    ("ux-1", "ux", "Who are the primary users?"),
    ("ux-2", "ux", "What devices/platforms must be supported?"),
    ("ux-3", "ux", "Are there accessibility requirements?"),
    // Security questions
    ("sec-1", "security", "What data needs to be protected?"),
    (
        "sec-2",
        "security",
        "Are there compliance requirements (GDPR, HIPAA)?",
    ),
    ("sec-3", "security", "How should authentication work?"),
];

/// Generate refinement questions for a requirement
pub fn generate_questions(requirement: &str) -> Vec<RefinementQuestion> {
    let mut questions = Vec::new();

    // Add relevant questions based on keywords in requirement
    let req_lower = requirement.to_lowercase();

    // Always add scope questions
    for (id, cat, q) in COMMON_QUESTIONS.iter().filter(|(_, c, _)| *c == "scope") {
        questions.push(RefinementQuestion {
            id: id.to_string(),
            question: q.to_string(),
            category: cat.to_string(),
            answered: false,
            answer: None,
        });
    }

    // Add tech questions if tech terms mentioned
    if req_lower.contains("api") || req_lower.contains("database") || req_lower.contains("server") {
        for (id, cat, q) in COMMON_QUESTIONS.iter().filter(|(_, c, _)| *c == "tech") {
            questions.push(RefinementQuestion {
                id: id.to_string(),
                question: q.to_string(),
                category: cat.to_string(),
                answered: false,
                answer: None,
            });
        }
    }

    // Add UX questions if UI/web mentioned
    if req_lower.contains("ui")
        || req_lower.contains("web")
        || req_lower.contains("app")
        || req_lower.contains("user")
    {
        for (id, cat, q) in COMMON_QUESTIONS.iter().filter(|(_, c, _)| *c == "ux") {
            questions.push(RefinementQuestion {
                id: id.to_string(),
                question: q.to_string(),
                category: cat.to_string(),
                answered: false,
                answer: None,
            });
        }
    }

    // Add security questions if auth/data mentioned
    if req_lower.contains("auth")
        || req_lower.contains("login")
        || req_lower.contains("password")
        || req_lower.contains("data")
    {
        for (id, cat, q) in COMMON_QUESTIONS.iter().filter(|(_, c, _)| *c == "security") {
            questions.push(RefinementQuestion {
                id: id.to_string(),
                question: q.to_string(),
                category: cat.to_string(),
                answered: false,
                answer: None,
            });
        }
    }

    questions
}

/// Format questions as markdown for display
pub fn format_questions_md(questions: &[RefinementQuestion]) -> String {
    let mut md = String::from("# Refinement Questions\n\n");
    md.push_str("Please answer these questions to clarify requirements:\n\n");

    let mut current_cat = String::new();
    for q in questions {
        if q.category != current_cat {
            current_cat = q.category.clone();
            md.push_str(&format!("\n## {}\n\n", capitalize(&current_cat)));
        }

        let status = if q.answered { "[x]" } else { "[ ]" };
        md.push_str(&format!("- {} {}\n", status, q.question));
        if let Some(ref answer) = q.answer {
            md.push_str(&format!("  > {}\n", answer));
        }
    }

    md
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_questions() {
        let questions = generate_questions("Build a web app with user authentication");
        assert!(!questions.is_empty());
        // Should have scope + ux + security questions
        assert!(questions.iter().any(|q| q.category == "scope"));
        assert!(questions.iter().any(|q| q.category == "ux"));
        assert!(questions.iter().any(|q| q.category == "security"));
    }

    #[test]
    fn test_format_questions_md() {
        let questions = generate_questions("CLI tool");
        let md = format_questions_md(&questions);
        assert!(md.contains("# Refinement Questions"));
        assert!(md.contains("## Scope"));
    }
}
