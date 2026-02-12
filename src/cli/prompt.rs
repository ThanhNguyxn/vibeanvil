//! Prompt command handler

use anyhow::Result;

use crate::cli::PromptKind;
use crate::prompt;

pub async fn run(kind: PromptKind) -> Result<()> {
    let template_name = match kind {
        PromptKind::Install => "install-vibeanvil",
        PromptKind::Architect => "architect",
        PromptKind::Developer => "developer",
        PromptKind::Qa => "qa",
        PromptKind::Plan => "plan",
        PromptKind::Review => "review",
        PromptKind::Commit => "commit",
        PromptKind::Debug => "debug",
        PromptKind::Xray => "xray",
        PromptKind::Vision => "vision",
        PromptKind::Security => "security",
        PromptKind::Migrate => "migrate",
        PromptKind::Refactor => "refactor",
    };

    let content = prompt::load_template(template_name)?;

    println!("{}", content);

    Ok(())
}
