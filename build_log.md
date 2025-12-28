    Checking vibeanvil v0.4.1 (D:\Code\vibeanvil)
error[E0716]: temporary value dropped while borrowed
  --> src\cli\plan.rs:38:16
   |
38 |     let root = workspace::workspace_path().parent().unwrap_or(std::path::Path::new("."));
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^                                              - temporary value is freed at the end of this statement
   |                |
   |                creates a temporary value which is freed while still in use
39 |     
40 |     if let Err(e) = repo_map.scan(root) {
   |                                   ---- borrow later used here
   |
   = note: consider using a `let` binding to create a longer lived value

warning: unused variable: `context`
  --> src\provider\mod.rs:47:57
   |
47 |     async fn generate_commit_message(&self, diff: &str, context: &Context) -> Result<String> {
   |                                                         ^^^^^^^ help: if this is intentional, prefix it with an underscore: `_context`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0716`.
warning: `vibeanvil` (bin "vibeanvil") generated 1 warning
error: could not compile `vibeanvil` (bin "vibeanvil") due to 1 previous error; 1 warning emitted
