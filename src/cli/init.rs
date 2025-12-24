//! Init command handler

use anyhow::Result;

use crate::audit::{AuditLogger, generate_session_id};
use crate::workspace;

pub async fn run(force: bool) -> Result<()> {
    println!("🔨 VibeAnvil - Contract-first vibe coding");
    println!();

    if workspace::workspace_exists().await && !force {
        println!("⚠️  Workspace already exists. Use --force to reinitialize.");
        return Ok(());
    }

    workspace::init_workspace(force).await?;

    let session_id = generate_session_id();
    let logger = AuditLogger::new(&session_id);
    logger.log_command("init", vec![format!("force={}", force)]).await?;

    println!("✓ Initialized .vibeanvil workspace");
    println!();
    println!("Next steps:");
    println!("  1. vibeanvil intake --message \"Your project requirements\"");
    println!("  2. vibeanvil blueprint --auto");
    println!("  3. vibeanvil contract create");
    println!();

    Ok(())
}
