//! Beejs CLI Enhanced Module
//! Stage 56.0 - CLI 功能完善与 Bun 兼容性

pub mod file_watcher;
pub mod repl;
pub mod package_json;
pub mod enhanced_cli;
pub mod commands;
pub mod script_executor;

pub use repl::Repl;
pub use commands::{CliApp, SubCommand, RunCommand, TestCommand, ReplCommand, BundleCommand};
pub use script_executor::{
    FileType, ModuleSystem, ExecutionContext, ExecutorConfig, ScriptExecutor,
    detect_file_type, shebang, args,
};
