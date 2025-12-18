//! CDN Provider Abstraction Layer
//! Supports multiple CDN providers with intelligent routing and configuration

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};

/// CDN Provider Type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CdnProviderType {
    Cloudflare,
    Vercel,
    AWSCloudFront,
    Fastly,
}

/// CDN Endpoint representing a deployment location
#[derive(Debug, Clone)]
pub struct CdnEndpoint {
    pub id: String,
    pub provider: CdnProviderType,
    pub region: String,
    pub endpoint_url: String,
    pub latency: f64, // milliseconds
    pub status: EndpointStatus,
    pub capacity: u64, // requests per second
    pub current_load: f64, // 0.0 to 1.0
}

/// Endpoint health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndpointStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

/// CDN Provider trait for abstraction
#[async_trait::async_trait]
pub trait CdnProvider: Send + Sync {
    /// Deploy code to CDN provider
    async fn deploy(&self, code: &[u8], region: &str) -> Result<DeploymentResult>;

    /// Get routing information for a region
    async fn route(&self, region: &str) -> Result<CdnEndpoint>;

    /// Invalidate cache across CDN
    async fn invalidate_cache(&self, paths: &[&str]) -> Result<()>;

    /// Get provider health status
    async fn health_check(&self) -> Result<ProviderHealth>;

    /// Update CDN configuration
    async fn update_config(&self, config: &HashMap<String, String>) -> Result<()>;
}

/// Deployment result
#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub deployment_id: String,
    pub endpoint_id: String,
    pub status: DeploymentStatus,
    pub deployment_url: Option<String>,
    pub estimated_propagation_time: u64, // seconds
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeploymentStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

/// Provider health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    pub provider: CdnProviderType,
    pub status: EndpointStatus,
    pub latency: f64,
    pub uptime: f64, // percentage
    pub last_check: std::time::SystemTime,
}

/// Smart Router for intelligent CDN selection
pub struct SmartRouter {
    providers: Arc<RwLock<Vec<Arc<dyn CdnProvider>>>>,
    routing_cache: Arc<RwLock<HashMap<String, CdnEndpoint>>>,
}

impl SmartRouter {
    /// Create a new smart router
    pub fn new() -> Self {
        SmartRouter {
            providers: Arc::new(RwLock::new(Vec::new())),
            routing_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a CDN provider
    pub async fn register_provider(&self, provider: Arc<dyn CdnProvider>) {
        let mut providers = self.providers.write().await;
        providers.push(provider);
    }

    /// Select the best CDN endpoint for a region
    pub async fn select_best_route(&self, region: &str) -> Result<CdnEndpoint> {
        let cache_key = format!("route:{}", region);

        // Check cache first
        {
            let cache = self.routing_cache.read().await;
            if let Some(endpoint) = cache.get(&cache_key) {
                if endpoint.status == EndpointStatus::Healthy {
                    return Ok(endpoint.clone());
                }
            }
        }

        // Query all providers
        let providers = self.providers.read().await;
        let mut candidates = Vec::new();

        for provider in providers.iter() {
            match provider.route(region).await {
                Ok(endpoint) => {
                    if endpoint.status == EndpointStatus::Healthy {
                        candidates.push(endpoint);
                    }
                }
                Err(_) => continue,
            }
        }

        // Select best candidate based on latency and load
        let best = candidates
            .into_iter()
            .min_by(|a, b| {
                let score_a = a.latency + (a.current_load * 100.0);
                let score_b = b.latency + (b.current_load * 100.0);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .context("No healthy endpoints available")?;

        // Update cache
        {
            let mut cache = self.routing_cache.write().await;
            cache.insert(cache_key, best.clone());
        }

        Ok(best)
    }

    /// Clear routing cache
    pub async fn clear_cache(&self) {
        let mut cache = self.routing_cache.write().await;
        cache.clear();
    }
}

/// CDN Configuration Optimizer
#[derive(Debug)]
pub struct CdnOptimizer {
    optimization_history: Arc<RwLock<Vec<OptimizationRecord>>>,
}

#[derive(Debug, Clone)]
struct OptimizationRecord {
    timestamp: std::time::SystemTime,
    config: HashMap<String, String>,
    performance_delta: f64,
}

impl CdnOptimizer {
    /// Create a new CDN optimizer
    pub fn new() -> Self {
        CdnOptimizer {
            optimization_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Optimize CDN configuration based on historical data
    pub async fn optimize(&self, mut config: HashMap<String, String>) -> Result<HashMap<String, String>> {
        let history = self.optimization_history.read().await;

        // Apply optimization rules based on historical performance
        if let Some(tier) = config.get("tier") {
            if tier == "enterprise" {
                // Enterprise tier optimizations
                config.insert("cache_level".to_string(), "aggressive".to_string());
                config.insert("compression".to_string(), "brotli".to_string());
                config.insert("http_version".to_string(), "3".to_string());
            }
        }

        // Enable HTTP/3 and 0-RTT for better performance
        config.insert("enable_http3".to_string(), "true".to_string());
        config.insert("enable_0rtt".to_string(), "true".to_string());

        // Record optimization
        drop(history);
        let mut history = self.optimization_history.write().await;
        history.push(OptimizationRecord {
            timestamp: std::time::SystemTime::now(),
            config: config.clone(),
            performance_delta: 0.0, // Would be calculated from actual metrics
        });

        Ok(config)
    }

    /// Get optimization recommendations
    pub async fn get_recommendations(&self) -> Result<Vec<String>> {
        let history = self.optimization_history.read().await;
        let mut recommendations = Vec::new();

        if history.len() > 10 {
            recommendations.push("Consider enabling edge compute for dynamic content".to_string());
            recommendations.push("Implement image optimization at edge".to_string());
            recommendations.push("Enable brotli compression for text assets".to_string());
        }

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        provider_type: CdnProviderType,
    }

    #[async_trait::async_trait]
    impl CdnProvider for MockProvider {
        async fn deploy(&self, _code: &[u8], _region: &str) -> Result<DeploymentResult> {
            Ok(DeploymentResult {
                deployment_id: "mock-deployment-123".to_string(),
                endpoint_id: "mock-endpoint".to_string(),
                status: DeploymentStatus::Complete,
                deployment_url: Some("https://mock.example.com".to_string()),
                estimated_propagation_time: 30,
            })
        }

        async fn route(&self, _region: &str) -> Result<CdnEndpoint> {
            Ok(CdnEndpoint {
                id: "mock-endpoint".to_string(),
                provider: self.provider_type,
                region: "us-west".to_string(),
                endpoint_url: "https://mock.example.com".to_string(),
                latency: 45.0,
                status: EndpointStatus::Healthy,
                capacity: 10000,
                current_load: 0.3,
            })
        }

        async fn invalidate_cache(&self, _paths: &[&str]) -> Result<()> {
            Ok(())
        }

        async fn health_check(&self) -> Result<ProviderHealth> {
            Ok(ProviderHealth {
                provider: self.provider_type,
                status: EndpointStatus::Healthy,
                latency: 45.0,
                uptime: 99.99,
                last_check: std::time::SystemTime::now(),
            })
        }

        async fn update_config(&self, _config: &HashMap<String, String>) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_smart_router_registration() {
        let router = SmartRouter::new();
        let provider = Arc::new(MockProvider { provider_type: CdnProviderType::Cloudflare });
        router.register_provider(provider).await;

        let providers = router.providers.read().await;
        assert_eq!(providers.len(), 1);
    }

    #[tokio::test]
    async fn test_smart_router_selection() {
        let router = SmartRouter::new();
        let cloudflare = Arc::new(MockProvider { provider_type: CdnProviderType::Cloudflare });
        let vercel = Arc::new(MockProvider { provider_type: CdnProviderType::Vercel });

        router.register_provider(cloudflare).await;
        router.register_provider(vercel).await;

        let route = router.select_best_route("us-west").await;
        assert!(route.is_ok());
    }

    #[tokio::test]
    async fn test_cdn_optimizer() {
        let optimizer = CdnOptimizer::new();
        let mut config = HashMap::new();
        config.insert("tier".to_string(), "enterprise".to_string());

        let optimized = optimizer.optimize(config).await;
        assert!(optimized.is_ok());
        let result = optimized.unwrap();

        assert_eq!(result.get("cache_level"), Some(&"aggressive".to_string()));
        assert_eq!(result.get("enable_http3"), Some(&"true".to_string()));
    }
}
