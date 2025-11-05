mod analyze;
mod analyze_impl;
pub mod archive;
mod astq;
mod config;
mod external_bin;
mod fork;
mod git;
mod hook;
mod hooks_setup;
mod init;
mod meta;
mod pm;
mod reflect;
mod status;
mod workflow;
mod wrapped;

// Re-export public functions
pub use analyze::analyze_metrics;
pub use archive::archive;
pub use astq::run_astq;
pub use config::handle_config;
pub use fork::handle_fork;
pub use hook::handle_hook;
pub use hooks_setup::auto_install_hooks;
pub use init::init_project;
pub use meta::meta_mode;
pub use pm::run_pm;
pub use reflect::run_reflect;
pub use status::show_status;
pub use workflow::{
    abort_workflow, list_guides, list_workflows, next_prompt, prev_prompt, repeat_prompt,
    reset_workflow, restart_workflow, start_workflow,
};
pub use wrapped::run_wrapped_command;
