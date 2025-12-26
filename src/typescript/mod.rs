// TypeScript 支持模块
//
pub mod compiler;

pub use compiler::{
    TypeScriptCompiler, TypeScriptCompilerConfig, TypeScriptTarget, TypeScriptModule,
    CompilationOutput, TypeScriptError, ErrorSeverity,
};
/// 快速编译 TypeScript 源代码
pub fn compile_typescript(source: &str, file_name: &str) -> Result<CompilationOutput, String> {
    let config: _ = TypeScriptCompilerConfig::default();
    let mut compiler = TypeScriptCompiler::new(config);
    compiler.compile_source(source, file_name).map_err(|e| e.to_string())
}
/// 快速编译 TypeScript 文件
pub fn compile_typescript_file(file_path: &std::path::Path) -> Result<CompilationOutput, String> {
    let config: _ = TypeScriptCompilerConfig::default();
    let mut compiler = TypeScriptCompiler::new(config);
    compiler.compile_file(file_path).map_err(|e| e.to_string())
}