//! Edge Computing Module
//! CDN integration, edge deployment, global distribution, and caching strategies

pub mod cdn_provider;
pub mod cloudflare_integration;
pub mod vercel_integration;
pub mod deployment_optimizer;
pub mod edge_runtime;
pub mod global_router;
pub mod cache_strategy;
pub mod node_manager;
pub mod local_cache;
pub mod offline_engine;

pub use cache_strategy::*;
pub use node_manager::*;
pub use local_cache::{*, CacheStats as LocalCacheStats};
pub use offline_engine::{*, ExecutionResult as OfflineExecutionResult};
