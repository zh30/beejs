//! Edge Computing Module
//! CDN integration, edge deployment, global distribution, and caching strategies

pub mod cdn_provider;
pub mod cloudflare_integration;
pub mod vercel_integration;
pub mod deployment_optimizer;
pub mod edge_runtime;
pub mod global_router;
pub mod cache_strategy;

pub use cache_strategy::*;
