//! WASM 模块加载器
//!
//! 负责高效加载、验证和实例化 WebAssembly 模块

use wasmtime::{Engine, Module, Instance, Store, Linker, Config};
use wasmtime_wasi::{WasiCtx, add_to_linker};
use anyhow::{Result, Context, anyhow};
use std::sync::Arc;
use std::time::Instant;

/// WebAssembly 模块结构体
///
/// 封装已加载的 WASM 模块及其元数据
pub struct WasmModule {
    /// 模块实例
    instance: Instance,
    /// 模块实例 ID
    id: String,
    /// 加载时间
    load_time: std::time::Duration,
    /// 模块大小（字节）
    size: usize,
}

impl WasmModule {
    /// 获取实例引用
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// 获取模块 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取加载时间
    pub fn load_time(&self) -> std::time::Duration {
        self.load_time
    }

    /// 获取模块大小
    pub fn size(&self) -> usize {
        self.size
    }
}

/// WASM 模块加载器
///
/// 提供高性能的 WASM 模块加载、验证和实例化功能
pub struct WasmModuleLoader {
    /// Wasmtime 引擎
    engine: Arc<Engine>,
    /// 加载器配置
    config: LoaderConfig,
}

#[derive(Debug, Clone)]
struct LoaderConfig {
    /// 最大模块大小（字节）
    max_module_size: usize,
    /// 是否启用验证
    enable_validation: bool,
    /// 是否启用并行加载
    enable_parallel: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            max_module_size: 100 * 1024 * 1024, // 100MB
            enable_validation: true,
            enable_parallel: true,
        }
    }
}

impl WasmModuleLoader {
    /// 创建新的模块加载器
    ///
    /// # 返回值
    /// * `Result<WasmModuleLoader>` - 成功返回加载器，失败返回错误
    pub fn new() -> Result<Self> {
        let config = Config::default();
        let engine = Arc::new(Engine::new(&config)
            .context("Failed to create Wasmtime engine for module loader")?);

        Ok(WasmModuleLoader {
            engine,
            config: LoaderConfig::default(),
        }
    }

    /// 创建自定义配置的加载器
    ///
    /// # 参数
    /// * `max_module_size` - 最大模块大小
    /// * `enable_validation` - 是否启用验证
    /// * `enable_parallel` - 是否启用并行加载
    ///
    /// # 返回值
    /// * `Result<WasmModuleLoader>` - 加载器实例
    pub fn new_with_config(
        max_module_size: usize,
        enable_validation: bool,
        enable_parallel: bool,
    ) -> Result<Self> {
        let config = Config::default();
        let engine = Arc::new(Engine::new(&config)
            .context("Failed to create Wasmtime engine")?);

        Ok(WasmModuleLoader {
            engine,
            config: LoaderConfig {
                max_module_size,
                enable_validation,
                enable_parallel,
            },
        })
    }

    /// 加载 WebAssembly 模块
    ///
    /// # 参数
    /// * `wasm_bytes` - WASM 字节码
    ///
    /// # 返回值
    /// * `Result<WasmModule>` - 成功返回模块实例，失败返回错误
    ///
    /// # 示例
    /// ```
    /// let loader = WasmModuleLoader::new()?;
    /// let wasm_bytes = read_wasm_file("module.wasm")?;
    /// let module = loader.load_module(&wasm_bytes)?;
    /// ```
    pub fn load_module(&self, wasm_bytes: &[u8]) -> Result<WasmModule> {
        let start = Instant::now();

        // 检查模块大小
        if wasm_bytes.len() > self.config.max_module_size {
            return Err(anyhow!(
                "Module size {} exceeds maximum allowed size {}",
                wasm_bytes.len(),
                self.config.max_module_size
            ));
        }

        // 验证模块
        if self.config.enable_validation {
            Module::validate(&self.engine, wasm_bytes)
                .context("WASM module validation failed")?;
        }

        // 编译模块
        let module = Module::new(&self.engine, wasm_bytes)
            .context("Failed to compile WASM module")?;

        // 创建 WASI 上下文
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .build();

        // 创建存储
        let mut store = Store::new(&self.engine, wasi);

        // 创建链接器
        let mut linker = Linker::new(&self.engine);
        add_to_linker(&mut linker, |s: &mut WasiCtx| s)
            .context("Failed to add WASI to linker")?;

        // 实例化模块
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        // 生成模块 ID（基于内容哈希）
        let id = self.generate_module_id(wasm_bytes)?;

        let load_time = start.elapsed();
        let size = wasm_bytes.len();

        Ok(WasmModule {
            instance,
            id,
            load_time,
            size,
        })
    }

    /// 从文件加载模块
    ///
    /// # 参数
    /// * `file_path` - 文件路径
    ///
    /// # 返回值
    /// * `Result<WasmModule>` - 模块实例
    pub fn load_module_from_file(&self, file_path: &str) -> Result<WasmModule> {
        let wasm_bytes = std::fs::read(file_path)
            .context(format!("Failed to read WASM file: {}", file_path))?;

        self.load_module(&wasm_bytes)
    }

    /// 预热模块缓存
    ///
    /// # 参数
    /// * `wasm_bytes_list` - WASM 模块字节码列表
    ///
    /// # 返回值
    /// * `Result<Vec<WasmModule>>` - 预热的模块列表
    pub fn prewarm_modules(&self, wasm_bytes_list: Vec<Vec<u8>>) -> Result<Vec<WasmModule>> {
        let mut modules = Vec::with_capacity(wasm_bytes_list.len());

        for wasm_bytes in wasm_bytes_list {
            let module = self.load_module(&wasm_bytes)?;
            modules.push(module);
        }

        Ok(modules)
    }

    /// 获取引擎引用
    pub fn engine(&self) -> &Arc<Engine> {
        &self.engine
    }

    /// 生成模块 ID
    ///
    /// # 参数
    /// * `wasm_bytes` - WASM 字节码
    ///
    /// # 返回值
    /// * `Result<String>` - 模块 ID
    fn generate_module_id(&self, wasm_bytes: &[u8]) -> Result<String> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        hasher.update(wasm_bytes);
        let hash = hasher.finalize();
        Ok(hash.to_hex().to_string())
    }

    /// 获取加载器统计信息
    ///
    /// # 返回值
    /// * `LoaderStats` - 统计信息
    pub fn get_stats(&self) -> LoaderStats {
        LoaderStats {
            max_module_size: self.config.max_module_size,
            enable_validation: self.config.enable_validation,
            enable_parallel: self.config.enable_parallel,
            engine_info: format!("Wasmtime Engine v38.0"),
        }
    }
}

/// 模块加载器统计信息
#[derive(Debug, Clone)]
pub struct LoaderStats {
    /// 最大模块大小
    pub max_module_size: usize,
    /// 是否启用验证
    pub enable_validation: bool,
    /// 是否启用并行加载
    pub enable_parallel: bool,
    /// 引擎信息
    pub engine_info: String,
}

impl std::fmt::Display for LoaderStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "WasmModuleLoader Stats:\n\
             - Max Module Size: {} MB\n\
             - Validation: {}\n\
             - Parallel Loading: {}\n\
             - Engine: {}",
            self.max_module_size / (1024 * 1024),
            if self.enable_validation { "Enabled" } else { "Disabled" },
            if self.enable_parallel { "Enabled" } else { "Disabled" },
            self.engine_info
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_creation() {
        let loader = WasmModuleLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_loader_with_config() {
        let loader = WasmModuleLoader::new_with_config(
            50 * 1024 * 1024, // 50MB
            true,
            true,
        );
        assert!(loader.is_ok());
    }

    #[test]
    fn test_module_loading() {
        let loader = WasmModuleLoader::new().unwrap();

        // 创建简单的 WASM 模块
        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], [ValType::I32]);
        let types_id = module.section(&types);

        let mut functions = FunctionSection::new();
        functions.function(types_id, 0);
        let functions_id = module.section(&functions);

        let mut exports = ExportSection::new();
        exports.function("test", functions_id, 0);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new([]);
        func.instruction(&Instruction::I32Const(42));
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();

        let result = loader.load_module(&wasm_bytes);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert!(!module.id().is_empty());
        assert!(module.size() > 0);
    }

    #[test]
    fn test_module_loading_performance() {
        let loader = WasmModuleLoader::new().unwrap();

        // 创建测试模块
        use wasm_encoder::*;
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.function([], []);
        let types_id = module.section(&types);

        let mut functions = FunctionSection::new();
        functions.function(types_id, 0);
        let functions_id = module.section(&functions);

        let mut exports = ExportSection::new();
        exports.function("test", functions_id, 0);
        module.section(&exports);

        let mut codes = CodeSection::new();
        let mut func = Function::new([]);
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        let wasm_bytes = module.finish();

        let start = Instant::now();
        let result = loader.load_module(&wasm_bytes);
        let load_time = start.elapsed();

        assert!(result.is_ok());
        assert!(load_time < std::time::Duration::from_millis(5));
    }

    #[test]
    fn test_prewarm_modules() {
        let loader = WasmModuleLoader::new().unwrap();

        // 创建两个测试模块
        use wasm_encoder::*;
        let mut module1 = Module::new();
        let mut types1 = TypeSection::new();
        types1.function([], [ValType::I32]);
        module1.section(&types1);
        let wasm_bytes1 = module1.finish();

        let mut module2 = Module::new();
        let mut types2 = TypeSection::new();
        types2.function([], [ValType::I32]);
        module2.section(&types2);
        let wasm_bytes2 = module2.finish();

        let modules = loader.prewarm_modules(vec![wasm_bytes1, wasm_bytes2]);
        assert!(modules.is_ok());

        let modules = modules.unwrap();
        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_stats() {
        let loader = WasmModuleLoader::new().unwrap();
        let stats = loader.get_stats();
        assert!(stats.max_module_size > 0);
        assert!(stats.enable_validation);
    }
}
