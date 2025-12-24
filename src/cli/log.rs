//! Log command handler

use anyhow::Result;

use crate::audit::read_audit_log;

pub async fn run(lines: usize, json: bool) -> Result<()> {
    let entries = read_audit_log(Some(lines)).await?;

    if entries.is_empty() {
        println!("No audit log entries found.");
        return Ok(());
    }

    if json {
        for entry in &entries {
            println!("{}", serde_json::to_string(entry)?);
        }
    } else {
        println!("📜 Audit Log (last {} entries)", entries.len());
        println!();
        
        for entry in &entries {
            let status = if entry.success { "✓" } else { "✗" };
            let time = entry.timestamp.format("%Y-%m-%d %H:%M:%S");
            
            println!("{} [{}] {}", status, time, entry.command);
            
            if !entry.args.is_empty() {
                println!("    Args: {}", entry.args.join(", "));
            }
            
            if let (Some(prev), Some(next)) = (&entry.prev_state, &entry.next_state) {
                println!("    State: {} → {}", prev, next);
            }
            
            if let Some(error) = &entry.error {
                println!("    Error: {}", error);
            }
        }
    }

    Ok(())
}
