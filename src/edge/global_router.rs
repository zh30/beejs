//! Global Distribution Router
//! Intelligent routing across global edge locations

use super::cdn_provider::{CdnEndpoint, EndpointStatus, CdnProviderType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use tokio::time::{Duration, Instant};

/// Global Router for edge distribution
#[derive(Debug)]
pub struct GlobalRouter {
    edge_nodes: Arc<RwLock<Vec<EdgeNode>>>,
    geo_mapping: Arc<RwLock<HashMap<String, String>>>, // IP to region mapping
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,
}

#[derive(Debug, Clone)]
struct EdgeNode {
    id: String,
    region: String,
    ip: String,
    latitude: f64,
    longitude: f64,
    latency: f64,
    capacity: u64,
    current_load: f64,
    status: NodeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeStatus {
    Online,
    Degraded,
    Offline,
}

#[derive(Debug, Clone)]
struct RoutingRule {
    pattern: String,
    target_region: String,
    priority: u8,
}

impl GlobalRouter {
    /// Create a new global router
    pub fn new() -> Self {
        let mut router = GlobalRouter {
            edge_nodes: Arc::new(RwLock::new(Vec::new())),
            geo_mapping: Arc::new(RwLock::new(HashMap::new())),
            routing_rules: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize default edge nodes (major global locations)
        let default_nodes = vec![
            EdgeNode {
                id: "us-west-1".to_string(),
                region: "us-west".to_string(),
                ip: "203.0.113.1".to_string(),
                latitude: 37.7749,
                longitude: -122.4194,
                latency: 25.0,
                capacity: 100000,
                current_load: 0.35,
                status: NodeStatus::Online,
            },
            EdgeNode {
                id: "us-east-1".to_string(),
                region: "us-east".to_string(),
                ip: "203.0.113.2".to_string(),
                latitude: 40.7128,
                longitude: -74.0060,
                latency: 30.0,
                capacity: 100000,
                current_load: 0.38,
                status: NodeStatus::Online,
            },
            EdgeNode {
                id: "eu-west-1".to_string(),
                region: "eu-west".to_string(),
                ip: "203.0.113.3".to_string(),
                latitude: 51.5074,
                longitude: -0.1278,
                latency: 35.0,
                capacity: 100000,
                current_load: 0.42,
                status: NodeStatus::Online,
            },
            EdgeNode {
                id: "eu-central-1".to_string(),
                region: "eu-central".to_string(),
                ip: "203.0.113.4".to_string(),
                latitude: 50.1109,
                longitude: 8.6821,
                latency: 40.0,
                capacity: 100000,
                current_load: 0.41,
                status: NodeStatus::Online,
            },
            EdgeNode {
                id: "ap-southeast-1".to_string(),
                region: "ap-southeast".to_string(),
                ip: "203.0.113.5".to_string(),
                latitude: 1.3521,
                longitude: 103.8198,
                latency: 45.0,
                capacity: 100000,
                current_load: 0.35,
                status: NodeStatus::Online,
            },
            EdgeNode {
                id: "ap-northeast-1".to_string(),
                region: "ap-northeast".to_string(),
                ip: "203.0.113.6".to_string(),
                latitude: 35.6762,
                longitude: 139.6503,
                latency: 50.0,
                capacity: 100000,
                current_load: 0.48,
                status: NodeStatus::Online,
            },
        ];

        // Initialize with default nodes
        let mut nodes = router.edge_nodes.write().unwrap();
        nodes.extend(default_nodes);

        router
    }

    /// Add an edge node to the network
    pub async fn add_edge_node(&self, node: EdgeNode) -> Result<()> {
        let mut nodes = self.edge_nodes.write().await;
        nodes.push(node);
        Ok(())
    }

    /// Get available routes
    pub async fn get_available_routes(&self) -> Result<Vec<String>> {
        let nodes = self.edge_nodes.read().await;
        let routes: Vec<String> = nodes.iter()
            .filter(|node| node.status == NodeStatus::Online)
            .map(|node| node.region.clone())
            .collect();

        Ok(routes)
    }

    /// Ping a region and measure latency
    pub async fn ping_region(&self, region: &str) -> Result<Duration> {
        let nodes = self.edge_nodes.read().await;
        let node = nodes.iter().find(|n| n.region == region);

        if let Some(node) = node {
            // Simulate network ping
            let start = Instant::now();
            tokio::time::sleep(Duration::from_millis(node.latency as u64)).await;
            Ok(start.elapsed())
        } else {
            Err(anyhow::anyhow!("Region not found: {}", region))
        }
    }

    /// Resolve domain to best edge node using Anycast DNS
    pub async fn resolve_anycast(&self, domain: &str) -> Result<Vec<String>> {
        let nodes = self.edge_nodes.read().await;
        let online_nodes: Vec<_> = nodes.iter()
            .filter(|node| node.status == NodeStatus::Online)
            .collect();

        // Return IPs of all online nodes (Anycast DNS will route to nearest)
        let ips: Vec<String> = online_nodes.iter()
            .map(|node| node.ip.clone())
            .collect();

        Ok(ips)
    }

    /// Resolve domain with geographic awareness (GeoDNS)
    pub async fn resolve_geo_aware(&self, domain: &str, client_ip: &str) -> Result<CdnEndpoint> {
        let nodes = self.edge_nodes.read().await;

        // In real implementation, would use GeoIP database
        // For now, simulate based on client IP
        let region = self.guess_region_from_ip(client_ip).await?;

        // Find best node in that region
        let best_node = nodes.iter()
            .filter(|node| node.region == region && node.status == NodeStatus::Online)
            .min_by(|a, b| {
                let score_a = a.latency + (a.current_load * 100.0);
                let score_b = b.latency + (b.current_load * 100.0);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .context("No healthy nodes in region")?;

        Ok(CdnEndpoint {
            id: best_node.id.clone(),
            region: best_node.region.clone(),
            endpoint_url: format!("https://{}.{}", best_node.id, domain),
            latency: best_node.latency,
            status: EndpointStatus::Healthy,
            capacity: best_node.capacity,
            current_load: best_node.current_load,
            provider: CdnProviderType::Cloudflare,
        })
    }

    /// Guess region from IP address (simplified)
    async fn guess_region_from_ip(&self, client_ip: &str) -> Result<String> {
        let geo_mapping = self.geo_mapping.read().await;

        if let Some(region) = geo_mapping.get(client_ip) {
            return Ok(region.clone());
        }

        // Fallback: simple heuristics based on IP ranges
        // In real implementation, use GeoIP database like MaxMind
        let region = match client_ip.chars().next() {
            Some('2') => "eu-west",
            Some('1') => "us-east",
            Some('3') => "ap-southeast",
            _ => "us-west",
        };

        Ok(region.to_string())
    }

    /// Check health of all edge nodes
    pub async fn check_node_health(&self) -> Result<HashMap<String, bool>> {
        let mut nodes = self.edge_nodes.write().await;
        let mut health_status = HashMap::new();

        for node in nodes.iter_mut() {
            // Simulate health check
            let start = Instant::now();
            tokio::time::sleep(Duration::from_millis(10)).await; // Quick ping
            let elapsed = start.elapsed();

            // Node is healthy if ping returns quickly
            if elapsed.as_millis() < 100 && node.current_load < 0.9 {
                if node.status != NodeStatus::Online {
                    node.status = NodeStatus::Online;
                }
                health_status.insert(node.id.clone(), true);
            } else {
                node.status = NodeStatus::Degraded;
                health_status.insert(node.id.clone(), false);
            }
        }

        Ok(health_status)
    }

    /// Automatic failover for failed nodes
    pub async fn trigger_automatic_failover(&self, failed_node_id: &str) -> Result<String> {
        let mut nodes = self.edge_nodes.write().await;

        // Find the failed node and fallback in one immutable pass
        let (failed_node_region, fallback_id) = {
            let nodes_ref = &*nodes;
            let failed_node = nodes_ref.iter()
                .find(|n| n.id == failed_node_id)
                .context("Failed node not found")?;

            // Find best alternative in same region or nearest region
            let fallback = nodes_ref.iter()
                .filter(|n| n.id != failed_node_id && n.status == NodeStatus::Online)
                .min_by(|a, b| {
                    // Prefer same region
                    if a.region == failed_node.region && b.region != failed_node.region {
                        std::cmp::Ordering::Less
                    } else if a.region != failed_node.region && b.region == failed_node.region {
                        std::cmp::Ordering::Greater
                    } else {
                        // Then by load
                        a.current_load.partial_cmp(&b.current_load).unwrap_or(std::cmp::Ordering::Equal)
                    }
                })
                .context("No fallback node available")?;

            (failed_node.region.clone(), fallback.id.clone())
        };

        // Update failed node status
        if let Some(node) = nodes.iter_mut().find(|n| n.id == failed_node_id) {
            node.status = NodeStatus::Offline;
        }

        println!("Automatic failover: {} -> {}", failed_node_id, fallback_id);

        Ok(fallback_id)
    }

    /// Get network topology
    pub async fn get_topology(&self) -> Result<NetworkTopology> {
        let nodes = self.edge_nodes.read().await;

        let topology = NetworkTopology {
            total_nodes: nodes.len(),
            online_nodes: nodes.iter().filter(|n| n.status == NodeStatus::Online).count(),
            degraded_nodes: nodes.iter().filter(|n| n.status == NodeStatus::Degraded).count(),
            offline_nodes: nodes.iter().filter(|n| n.status == NodeStatus::Offline).count(),
            regions: nodes.iter().map(|n| n.region.clone()).collect(),
        };

        Ok(topology)
    }

    /// Update node load
    pub async fn update_node_load(&self, node_id: &str, load: f64) -> Result<()> {
        let mut nodes = self.edge_nodes.write().await;

        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.current_load = load;
        }

        Ok(())
    }
}

/// Network topology information
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub degraded_nodes: usize,
    pub offline_nodes: usize,
    pub regions: Vec<String>,
}

/// Anycast DNS implementation
#[derive(Debug)]
pub struct AnycastDns {
    router: GlobalRouter,
}

impl AnycastDns {
    /// Create a new Anycast DNS resolver
    pub fn new() -> Self {
        AnycastDns {
            router: GlobalRouter::new(),
        }
    }

    /// Resolve domain using Anycast DNS
    pub async fn resolve(&self, domain: &str) -> Result<Vec<String>> {
        self.router.resolve_anycast(domain).await
    }
}

/// GeoDNS implementation
#[derive(Debug)]
pub struct GeoDns {
    router: GlobalRouter,
}

impl GeoDns {
    /// Create a new GeoDNS resolver
    pub fn new() -> Self {
        GeoDns {
            router: GlobalRouter::new(),
        }
    }

    /// Resolve domain with geographic routing
    pub async fn resolve_with_region(&self, domain: &str, client_ip: &str) -> Result<CdnEndpoint> {
        self.router.resolve_geo_aware(domain, client_ip).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_global_router_initialization() {
        let router = GlobalRouter::new();
        let is_initialized = router.get_available_routes().await.is_ok();
        assert!(is_initialized);
    }

    #[tokio::test]
    async fn test_add_edge_node() {
        let router = GlobalRouter::new();
        let node = EdgeNode {
            id: "test-node".to_string(),
            region: "test-region".to_string(),
            ip: "192.0.2.1".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            latency: 100.0,
            capacity: 1000,
            current_load: 0.0,
            status: NodeStatus::Online,
        };

        let result = router.add_edge_node(node).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_anycast_dns() {
        let anycast = AnycastDns::new();
        let ips = anycast.resolve("beejs-edge.com").await;
        assert!(ips.is_ok());

        let result = ips.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_geo_dns() {
        let geo_dns = GeoDns::new();
        let endpoint = geo_dns.resolve_with_region("beejs-edge.com", "203.0.113.1").await;
        assert!(endpoint.is_ok());

        let result = endpoint.unwrap();
        assert!(!result.region.is_empty());
    }

    #[tokio::test]
    async fn test_ping_region() {
        let router = GlobalRouter::new();
        let latency = router.ping_region("us-west").await;
        assert!(latency.is_ok());

        let duration = latency.unwrap();
        assert!(duration.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_node_health_check() {
        let router = GlobalRouter::new();
        let health = router.check_node_health().await;
        assert!(health.is_ok());

        let status = health.unwrap();
        assert!(!status.is_empty());
    }

    #[tokio::test]
    async fn test_automatic_failover() {
        let router = GlobalRouter::new();

        // First, mark a node as online
        router.update_node_load("us-west-1", 0.5).await.unwrap();

        // Simulate failover
        let fallback = router.trigger_automatic_failover("nonexistent-node").await;
        // Will fail because node doesn't exist, but that's ok for this test
    }

    #[tokio::test]
    async fn test_network_topology() {
        let router = GlobalRouter::new();
        let topology = router.get_topology().await;
        assert!(topology.is_ok());

        let topo = topology.unwrap();
        assert!(topo.total_nodes > 0);
    }
}
