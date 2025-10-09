mod analyze;
mod hook;
mod workflow;

// Re-export public functions
pub use analyze::analyze_metrics;
pub use hook::handle_hook;
pub use workflow::{next_prompt, reset_workflow, show_status, start_workflow};
