//! Beejs CLI Enhanced Module
//! Stage 56.0 - CLI 功能完善与 Bun 兼容性
//! Stage 91 Phase 4.1 - 开发者体验提升
//! Stage 91 Phase 4.3 - 快速启动模板系统

pub mod file_watcher;
pub mod repl;
pub mod package_json;
pub mod enhanced_cli;
pub mod commands;
pub mod script_executor;
pub mod wasm_commands;
pub mod module_resolver;

// Stage 91 Phase 4.1: 新增 CLI 增强模块
pub mod output_formatter;
pub mod init_command;
pub mod info_command;
pub mod doctor_command;

// Stage 91 Phase 4.2: 增强 REPL 模块
pub mod repl_completer;
pub mod repl_highlighter;
pub mod repl_enhanced;

// Stage 91 Phase 4.3: 快速启动模板系统
pub mod template_system;

pub use repl::Repl;
pub use commands::{CliApp, SubCommand, RunCommand, TestCommand, ReplCommand, BundleCommand, ProfileCommand, InitCommand as InitCommandArgs, InfoCommandArgs, DoctorCommandArgs, UpgradeCommand, ProjectTemplateArg};
pub use script_executor::{
    FileType, ModuleSystem, ExecutionContext, ExecutorConfig, ScriptExecutor,
    detect_file_type, shebang, args,
};

// Stage 91 Phase 4.1: 导出新命令
pub use output_formatter::OutputFormatter;
pub use init_command::{InitCommand, InitConfig, ProjectTemplate};
pub use info_command::{InfoCommand, SystemInfo};
pub use doctor_command::{DoctorCommand, CheckStatus, DiagnosticCheck};

// Stage 91 Phase 4.2: 导出增强 REPL
pub use repl_completer::{ReplCompleter, CompletionCandidate, CompletionKind, CompletionContext};
pub use repl_highlighter::{ReplHighlighter, HighlightTheme, HighlightedToken, TokenType};
pub use repl_enhanced::{EnhancedRepl, EnhancedReplConfig, EnhancedReplResult, EnhancedReplStats};

// Stage 91 Phase 4.3: 导出模板系统
pub use template_system::{
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    TemplateEngine, TemplateRegistry, TemplateInstantiator, TemplateInstantiationConfig,
    DirectoryGenerator, DirectoryStructure, FileEntry, DependencyInstaller, PackageManager,
    ProjectTemplate as TemplateDefinition,
};
