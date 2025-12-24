//! Beautiful UI helpers for CLI output

use colored::Colorize;

/// Print the banner/logo
pub fn print_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ██╗   ██╗██╗██████╗ ███████╗ █████╗ ███╗   ██╗██╗   ██╗██╗  ║
║   ██║   ██║██║██╔══██╗██╔════╝██╔══██╗████╗  ██║██║   ██║██║  ║
║   ██║   ██║██║██████╔╝█████╗  ███████║██╔██╗ ██║██║   ██║██║  ║
║   ╚██╗ ██╔╝██║██╔══██╗██╔══╝  ██╔══██║██║╚██╗██║╚██╗ ██╔╝██║  ║
║    ╚████╔╝ ██║██████╔╝███████╗██║  ██║██║ ╚████║ ╚████╔╝ ███╗ ║
║     ╚═══╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝  ╚═══╝  ╚══╝ ║
║                                                               ║
║   Contract-first vibe coding with evidence & brain pack  🧠   ║
╚═══════════════════════════════════════════════════════════════╝
"#;
    println!("{}", banner.cyan());
}

/// Print a success message with icon
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg.green());
}

/// Print an error message with icon
pub fn error(msg: &str) {
    println!("{} {}", "✗".red().bold(), msg.red());
}

/// Print a warning message with icon
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Print an info message with icon
pub fn info(msg: &str) {
    println!("{} {}", "→".blue().bold(), msg);
}

/// Print a step message
pub fn step(num: u32, total: u32, msg: &str) {
    println!("{} {}", format!("[{}/{}]", num, total).dimmed(), msg);
}

/// Print a header
pub fn header(title: &str) {
    let width = 50;
    let line = "═".repeat(width);
    println!("\n{}", line.cyan());
    println!("{}", format!(" {} ", title).cyan().bold());
    println!("{}\n", line.cyan());
}

/// Print a section divider
pub fn divider() {
    println!("{}", "─".repeat(50).dimmed());
}

/// Print key-value pair with nice formatting
pub fn kv(key: &str, value: &str) {
    println!("  {} {}", format!("{}:", key).dimmed(), value);
}

/// Print a boxed message
pub fn boxed(msg: &str, style: BoxStyle) {
    let width = msg.len() + 4;
    let border = match style {
        BoxStyle::Success => "─".repeat(width).green(),
        BoxStyle::Error => "─".repeat(width).red(),
        BoxStyle::Info => "─".repeat(width).cyan(),
        BoxStyle::Warning => "─".repeat(width).yellow(),
    };

    let top = format!("┌{}┐", border);
    let mid = format!("│  {}  │", msg);
    let bot = format!("└{}┘", border);

    match style {
        BoxStyle::Success => {
            println!("{}", top.green());
            println!("{}", mid.green());
            println!("{}", bot.green());
        }
        BoxStyle::Error => {
            println!("{}", top.red());
            println!("{}", mid.red());
            println!("{}", bot.red());
        }
        BoxStyle::Info => {
            println!("{}", top.cyan());
            println!("{}", mid.cyan());
            println!("{}", bot.cyan());
        }
        BoxStyle::Warning => {
            println!("{}", top.yellow());
            println!("{}", mid.yellow());
            println!("{}", bot.yellow());
        }
    }
}

#[derive(Clone, Copy)]
pub enum BoxStyle {
    Success,
    Error,
    Info,
    Warning,
}

/// Print state transition
pub fn state_transition(from: &str, to: &str) {
    println!(
        "\n  {} {} {} {}",
        from.yellow(),
        "→".dimmed(),
        to.green().bold(),
        "✓".green()
    );
}

/// Print a table row
pub fn table_row(label: &str, value: &str, width: usize) {
    let padding = width.saturating_sub(label.len());
    println!("  {}{}{}", label.white(), " ".repeat(padding), value.cyan());
}
