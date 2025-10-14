mod analyze;
mod astq;
mod git;
mod hook;
mod reflect;
mod workflow;
mod wrapped;

// Re-export public functions
pub use analyze::analyze_metrics;
pub use astq::run_astq;
pub use git::run_git;
pub use hook::handle_hook;
pub use reflect::run_reflect;
pub use workflow::{next_prompt, repeat_prompt, reset_workflow, show_status, start_workflow};
pub use wrapped::run_wrapped_command;
