mod analyze;
mod astq;
mod config;
mod git;
mod hook;
mod init;
mod meta;
mod reflect;
mod workflow;
mod wrapped;

// Re-export public functions
pub use analyze::analyze_metrics;
pub use astq::run_astq;
pub use config::handle_config;
pub use git::run_git;
pub use hook::handle_hook;
pub use init::init_project;
pub use meta::meta_mode;
pub use reflect::run_reflect;
pub use workflow::{
    abort_workflow, next_prompt, repeat_prompt, reset_workflow, restart_workflow, show_status,
    start_workflow,
};
pub use wrapped::run_wrapped_command;
