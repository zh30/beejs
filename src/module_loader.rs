use std::path::PathBuf;
use anyhow::{Result, anyhow};
use rquickjs::{Context, Ctx, Value};

/// Module loader for Node.js-style require() support
pub struct ModuleLoader {
    base_dir: PathBuf,
}

impl ModuleLoader {
    /// Create a new module loader with the given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
        }
    }

    /// Set up the module system in the given QuickJS context
    pub fn setup_module_system(&self, ctx: &Context) -> Result<()> {
        ctx.with(|ctx| {
            self.setup_require_function(ctx)
        })
    }

    fn setup_require_function(&self, ctx: Ctx) -> Result<()> {
        let base_dir = self.base_dir.clone();
        let ctx_clone = ctx.clone();

        // Create require function
        let require_fn = rquickjs::function::Function::new(
            ctx.clone(),
            move |_ctx: Ctx, module_path: String| -> Value {
                // Inline module loading to avoid context management issues
                let result = load_module_simple(&ctx_clone, &module_path, &base_dir);
                result.expect("Failed to load module")
            }
        );

        // Set up global require function
        ctx.globals().set("require", require_fn)?;

        Ok(())
    }
}

/// Load a module by path (simplified version - no module.exports)
fn load_module_simple<'a>(
    ctx: &Ctx<'a>,
    module_path: &str,
    base_dir: &PathBuf,
) -> Result<Value<'a>> {
    // Resolve module path
    let resolved_path = resolve_module_path(module_path, base_dir)?;

    // Read module file
    let code = std::fs::read_to_string(&resolved_path)
        .map_err(|e| anyhow!("Failed to read module file {:?}: {}", resolved_path, e))?;

    // Create a simple module wrapper that sets module.exports
    let wrapper = format!(
        r#"
        const module = {{ exports: {{}} }};
        const exports = module.exports;
        {code};
        module.exports;
        "#
    );

    // Execute module code with module context
    let result: Result<Option<Value>, _> = ctx.eval(&*wrapper);

    match result {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(anyhow!("Module returned undefined")),
        Err(e) => {
            Err(anyhow!("Module execution error: {}", e))
        }
    }
}

/// Resolve module path to absolute file path
fn resolve_module_path(module_path: &str, base_dir: &PathBuf) -> Result<PathBuf> {
    // Handle relative paths
    if module_path.starts_with("./") || module_path.starts_with("../") {
        let mut full_path = base_dir.clone();
        full_path.push(module_path);

        // Add .js extension if not present
        if full_path.extension().is_none() {
            full_path.set_extension("js");
        }

        // Normalize the path
        let normalized = full_path.canonicalize()
            .unwrap_or(full_path);

        Ok(normalized)
    } else {
        // Built-in modules not supported yet
        Err(anyhow!("Built-in modules not supported: {}", module_path))
    }
}
