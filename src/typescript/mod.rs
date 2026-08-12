// TypeScript 支持模块
//
pub mod cache;
pub mod compiler;

pub use compiler::{
    CompilationOutput, ErrorSeverity, TypeScriptCompiler, TypeScriptCompilerConfig,
    TypeScriptError, TypeScriptModule, TypeScriptTarget,
};
/// 快速编译 TypeScript 源代码（带内容哈希缓存）
pub fn compile_typescript(source: &str, file_name: &str) -> Result<CompilationOutput, String> {
    if let Some(hit) = cache::get_cached(source, file_name) {
        return Ok(hit);
    }
    let config: _ = TypeScriptCompilerConfig::default();
    let mut compiler = TypeScriptCompiler::new(config);
    let output = compiler
        .compile_source(source, file_name)
        .map_err(|e: anyhow::Error| e.to_string())?;
    cache::put_cached(source, file_name, &output);
    Ok(output)
}
/// 快速编译 TypeScript 文件
pub fn compile_typescript_file(file_path: &std::path::Path) -> Result<CompilationOutput, String> {
    let source = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let file_name = file_path.to_string_lossy().to_string();
    compile_typescript(&source, &file_name)
}
