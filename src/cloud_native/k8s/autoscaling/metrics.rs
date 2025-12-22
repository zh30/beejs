//! Metrics collection for HPA
//! Gathers resource usage metrics from Kubernetes

use kube::Api;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::super::crd::CustomMetric;

/// Metrics client for collecting pod and resource metrics
pub struct MetricsClient {
    /// Kubernetes client
    client: kube::Client,

    /// Metrics API
    metrics_api: Api<k8s::metrics::v1beta1::PodMetrics>,
}

impl MetricsClient {
    /// Create a new metrics client
    pub fn new(client: kube::Client, namespace: &str) -> Self {
        let metrics_api = Api::namespaced(client.clone(), namespace);
        Self {
            client,
            metrics_api,
        }
    }

    /// Collect pod metrics
    pub async fn collect_pod_metrics(&self, label_selector: &str) -> Result<PodMetricsSummary, Error> {
        let params = k8s::ListParams {
            label_selector: Some(label_selector.to_string()),
            ..Default::default()
        };

        let pods = self.metrics_api.list(&params).await?;

        let mut total_cpu_millicores = 0.0;
        let mut total_memory_bytes = 0.0;
        let mut total_cpu_request = 0.0;
        let mut total_memory_request = 0.0;

        for pod in pods {
            debug!("Processing pod: {}", pod.name_any());

            // Aggregate container metrics
            for container in pod.containers {
                // CPU usage (in millicores)
                if let Some(usage) = &container.usage.cpu {
                    if let Some(value) = parse_cpu_usage(usage) {
                        total_cpu_millicores += value;
                    }
                }

                // Memory usage (in bytes)
                if let Some(usage) = &container.usage.memory {
                    if let Some(value) = parse_memory_usage(usage) {
                        total_memory_bytes += value;
                    }
                }

                // CPU request (if available in annotations)
                if let Some(cpu_request) = container
                    .resources
                    .as_ref()
                    .and_then(|r| r.requests.get("cpu"))
                {
                    if let Some(value) = parse_cpu_usage(cpu_request) {
                        total_cpu_request += value;
                    }
                }

                // Memory request (if available in annotations)
                if let Some(mem_request) = container
                    .resources
                    .as_ref()
                    .and_then(|r| r.requests.get("memory"))
                {
                    if let Some(value) = parse_memory_usage(mem_request) {
                        total_memory_request += value;
                    }
                }
            }
        }

        Ok(PodMetricsSummary {
            total_pods: pods.items.len() as u32,
            total_cpu_millicores,
            total_memory_bytes,
            total_cpu_request,
            total_memory_request,
        })
    }

    /// Collect custom metrics
    pub async fn collect_custom_metrics(
        &self,
        metrics: &[CustomMetric],
    ) -> Result<HashMap<String, f64>, Error> {
        let mut result = HashMap::new();

        for metric in metrics {
            debug!("Collecting custom metric: {}", metric.name);

            let value = match metric.metric_type.as_str() {
                "Pod" => self.collect_pod_metric(&metric).await?,
                "Resource" => self.collect_resource_metric(&metric).await?,
                _ => {
                    warn!("Unsupported metric type: {}", metric.metric_type);
                    continue;
                }
            };

            result.insert(metric.name.clone(), value);
        }

        Ok(result)
    }

    /// Collect a single pod metric
    async fn collect_pod_metric(&self, metric: &CustomMetric) -> Result<f64, Error> {
        // TODO: Implement custom pod metric collection
        // This would involve querying external metrics API or custom metrics server

        Ok(0.0)
    }

    /// Collect a single resource metric
    async fn collect_resource_metric(&self, metric: &CustomMetric) -> Result<f64, Error> {
        // TODO: Implement custom resource metric collection
        // This would involve querying resource metrics API

        Ok(0.0)
    }
}

/// Pod metrics summary
#[derive(Debug, Clone)]
pub struct PodMetricsSummary {
    /// Total number of pods
    pub total_pods: u32,

    /// Total CPU usage in millicores
    pub total_cpu_millicores: f64,

    /// Total memory usage in bytes
    pub total_memory_bytes: f64,

    /// Total CPU request in millicores
    pub total_cpu_request: f64,

    /// Total memory request in bytes
    pub total_memory_request: f64,
}

impl PodMetricsSummary {
    /// Calculate CPU usage percentage
    pub fn cpu_usage_percent(&self) -> f64 {
        if self.total_cpu_request > 0.0 {
            (self.total_cpu_millicores / self.total_cpu_request) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.total_memory_request > 0.0 {
            (self.total_memory_bytes / self.total_memory_request) * 100.0
        } else {
            0.0
        }
    }

    /// Convert CPU millicores to cores
    pub fn cpu_cores(&self) -> f64 {
        self.total_cpu_millicores / 1000.0
    }

    /// Convert memory bytes to GB
    pub fn memory_gb(&self) -> f64 {
        self.total_memory_bytes / (1024.0 * 1024.0 * 1024.0)
    }
}

/// Parse CPU usage string (e.g., "100m", "1")
fn parse_cpu_usage(usage: &str) -> Option<f64> {
    if usage.ends_with('m') {
        // Millicores (e.g., "100m")
        let value = usage.trim_end_matches('m');
        value.parse::<f64>().ok()
    } else {
        // Cores (e.g., "1")
        let value = usage.parse::<f64>().ok()?;
        Some(value * 1000.0) // Convert to millicores
    }
}

/// Parse memory usage string (e.g., "1Gi", "100Mi")
fn parse_memory_usage(usage: &str) -> Option<f64> {
    if usage.ends_with("Ki") {
        let value = usage.trim_end_matches("Ki").parse::<f64>().ok()?;
        Some(value * 1024.0)
    } else if usage.ends_with("Mi") {
        let value = usage.trim_end_matches("Mi").parse::<f64>().ok()?;
        Some(value * 1024.0 * 1024.0)
    } else if usage.ends_with("Gi") {
        let value = usage.trim_end_matches("Gi").parse::<f64>().ok()?;
        Some(value * 1024.0 * 1024.0 * 1024.0)
    } else if usage.ends_with("Ti") {
        let value = usage.trim_end_matches("Ti").parse::<f64>().ok()?;
        Some(value * 1024.0 * 1024.0 * 1024.0 * 1024.0)
    } else {
        // Assume bytes
        usage.parse::<f64>().ok()
    }
}

/// Error type for metrics collection
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Other error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_usage() {
        assert_eq!(parse_cpu_usage("100m"), Some(100.0));
        assert_eq!(parse_cpu_usage("1"), Some(1000.0));
        assert_eq!(parse_cpu_usage("0.5"), Some(500.0));
    }

    #[test]
    fn test_parse_memory_usage() {
        assert_eq!(parse_memory_usage("1024Ki"), Some(1024.0 * 1024.0));
        assert_eq!(parse_memory_usage("100Mi"), Some(100.0 * 1024.0 * 1024.0));
        assert_eq!(parse_memory_usage("1Gi"), Some(1024.0 * 1024.0 * 1024.0));
    }

    #[test]
    fn test_pod_metrics_summary() {
        let summary = PodMetricsSummary {
            total_pods: 5,
            total_cpu_millicores: 500.0,
            total_memory_bytes: 1024.0 * 1024.0 * 1024.0, // 1GB
            total_cpu_request: 1000.0,
            total_memory_request: 2.0 * 1024.0 * 1024.0 * 1024.0, // 2GB
        };

        assert_eq!(summary.total_pods, 5);
        assert_eq!(summary.cpu_usage_percent(), 50.0); // 500/1000 * 100
        assert_eq!(summary.memory_usage_percent(), 50.0); // 1GB/2GB * 100
        assert_eq!(summary.cpu_cores(), 0.5); // 500m / 1000
        assert_eq!(summary.memory_gb(), 1.0);
    }
}
