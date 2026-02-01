//! Prompt command handler

use anyhow::Result;

use crate::cli::PromptKind;
use crate::prompt;

pub async fn run(kind: PromptKind) -> Result<()> {
    let template_name = match kind {
        PromptKind::Install => "install-vibeanvil",
    };

    let content = prompt::load_template(template_name).unwrap_or_else(|_| {
        "Install and configure VibeAnvil by following the instructions here:\nhttps://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/docs/guide/installation.md".to_string()
    });

    println!("{}", content);

    Ok(())
}
