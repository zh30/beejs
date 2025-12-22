//! Istio Service Mesh integration
//! Provides configuration and management for Istio

use std::collections::HashMap;

/// Istio configuration manager
pub struct IstioConfigManager {
    /// Istio client
    client: kube::Client,

    /// Istio configuration
    config: IstioConfig,
}

impl IstioConfigManager {
    /// Create a new Istio config manager
    pub fn new(client: kube::Client, config: IstioConfig) -> Self {
        Self { client, config }
    }

    /// Configure Istio for Beejs services
    pub async fn configure(&self) -> Result<(), Error> {
        info!("Configuring Istio for Beejs services");

        // 1. Configure namespace labels for sidecar injection
        self.configure_namespace().await?;

        // 2. Create DestinationRules
        self.create_destination_rules().await?;

        // 3. Create VirtualServices
        self.create_virtual_services().await?;

        // 4. Create Gateway
        self.create_gateway().await?;

        // 5. Configure PeerAuthentication
        self.configure_peer_authentication().await?;

        // 6. Configure AuthorizationPolicy
        self.configure_authorization_policy().await?;

        info!("Istio configuration completed successfully");

        Ok(())
    }

    /// Configure namespace for sidecar injection
    async fn configure_namespace(&self) -> Result<(), Error> {
        let namespaces: Api<k8s::api::core::v1::Namespace> = Api::all(self.client.clone());

        // Check if namespace exists
        if let Err(e) = namespaces.get(&self.config.namespace).await {
            if matches!(e, kube::Error::Api(kube::api::Error { code: 404, .. })) {
                // Create namespace
                let namespace = k8s::api::core::v1::Namespace {
                    metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                        name: Some(self.config.namespace.clone()),
                        labels: Some(HashMap::from([
                            ("istio-injection".to_string(), "enabled".to_string()),
                        ])),
                        annotations: Some(HashMap::from([
                            ("istio.io/rev".to_string(), "default".to_string()),
                        ])),
                        ..Default::default()
                    },
                    spec: None,
                    status: None,
                };

                namespaces.create(&k8s::api::PostParams::default(), &namespace).await?;
                info!("Created namespace: {}", self.config.namespace);
            } else {
                return Err(Error::Kube(e));
            }
        } else {
            // Update namespace labels
            let patch = serde_json::json!({
                "metadata": {
                    "labels": {
                        "istio-injection": "enabled"
                    }
                }
            });

            let params = k8s::PatchParams::default();
            namespaces.patch(&self.config.namespace, &params, &k8s::Patch::Merge(&patch)).await?;
        }

        Ok(())
    }

    /// Create DestinationRules
    async fn create_destination_rules(&self) -> Result<(), Error> {
        let destination_rules: Api<k8s::istio::networking::v1beta1::DestinationRule> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        for service in &self.config.services {
            let destination_rule = k8s::istio::networking::v1beta1::DestinationRule {
                metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(format!("{}-dr", service.name)),
                    namespace: Some(self.config.namespace.clone()),
                    labels: Some(HashMap::from([
                        ("beejs.io/service".to_string(), service.name.clone()),
                    ])),
                    ..Default::default()
                },
                spec: Some(k8s::istio::networking::v1beta1::DestinationRuleSpec {
                    host: service.name.clone(),
                    traffic_policy: Some(k8s::istio::networking::v1beta1::TrafficPolicy {
                        load_balancer: Some(k8s::istio::networking::v1beta1::LoadBalancerSettings {
                            simple: Some(k8s::istio::networking::v1beta1::LoadBalancerSimple::LeastRequest),
                            consistent_hash: None,
                            simple_lb_algorithm: None,
                            locality_weighted_lb_config: None,
                            warmup_duration: None,
                        }),
                        connection_pool: Some(k8s::istio::networking::v1beta1::ConnectionPoolSettings {
                            tcp: Some(k8s::istio::networking::v1beta1::ConnectionPoolSettingsTCPSettings {
                                max_connections: Some(self.config.traffic_policy.connection_pool.max_connections),
                                connect_timeout: Some(k8s::apimachinery::pkg::apis::meta::v1::Duration::from(
                                    std::time::Duration::from_secs(10)
                                )),
                                tcp_keepalive: None,
                                tcp_max_connections: None,
                                handshake_timeout: None,
                                delayed_close_timeout: None,
                                pass_through_mode: None,
                                use_client_protocol: None,
                            }),
                            http: Some(k8s::istio::networking::v1beta1::ConnectionPoolSettingsHTTPSettings {
                                http1_max_pending_requests: Some(self.config.traffic_policy.connection_pool.max_pending_requests),
                                http2_max_requests: Some(1000),
                                max_requests_per_connection: Some(100),
                                max_retries: Some(3),
                                consecutive_gateway_failure: None,
                                interval: None,
                                base_ejection_time: Some(k8s::apimachinery::pkg::apis::meta::v1::Duration::from(
                                    std::time::Duration::from_secs(30)
                                )),
                                max_ejection_percent: Some(50),
                                min_health_percent: Some(50),
                                split_external_local_origin_errors: None,
                                consecutive_local_origin_failure: None,
                                h2_upgrade_policy: None,
                                use_client_protocol: None,
                                allow_upgrade: None,
                                headers_to_upstream_request_headers: None,
                                headers_to_downstream_request_headers: None,
                                headers_to_upstream_response_headers: None,
                                auto_sni: None,
                                autossl: None,
                            }),
                        }),
                        outlier_detection: Some(k8s::istio::networking::v1beta1::OutlierDetection {
                            consecutive_gateway_errors: Some(self.config.traffic_policy.outlier_detection.consecutive_errors),
                            consecutive_5xx_errors: Some(self.config.traffic_policy.outlier_detection.consecutive_errors),
                            interval: Some(k8s::apimachinery::pkg::apis::meta::v1::Duration::from(
                                self.config.traffic_policy.outlier_detection.interval
                            )),
                            base_ejection_time: Some(k8s::apimachinery::pkg::apis::meta::v1::Duration::from(
                                self.config.traffic_policy.outlier_detection.base_ejection_time
                            )),
                            max_ejection_percent: Some(50),
                            min_health_percent: Some(50),
                            split_external_local_origin_errors: None,
                            consecutive_local_origin_failure: None,
                            successful_circuit_breaker: None,
                            enhanced_circuit_breaker: None,
                            consecutive_circuit_breaker_error: None,
                        }),
                        tls: None,
                        port_level_settings: None,
                        connection_balance: None,
                        default_port_level_settings: None,
                    }),
                    subsets: Some(vec![
                        k8s::istio::networking::v1beta1::Subset {
                            name: "v1".to_string(),
                            labels: Some(HashMap::from([
                                ("version".to_string(), "v1".to_string()),
                            ])),
                            traffic_policy: None,
                        },
                        k8s::istio::networking::v1beta1::Subset {
                            name: "v2".to_string(),
                            labels: Some(HashMap::from([
                                ("version".to_string(), "v2".to_string()),
                            ])),
                            traffic_policy: None,
                        },
                    ]),
                    export_to: None,
                    workload_selector: None,
                }),
                status: None,
            };

            let params = k8s::api::PostParams::default();
            destination_rules.create(&params, &destination_rule).await?;

            info!("Created DestinationRule: {}-dr", service.name);
        }

        Ok(())
    }

    /// Create VirtualServices
    async fn create_virtual_services(&self) -> Result<(), Error> {
        let virtual_services: Api<k8s::istio::networking::v1beta1::VirtualService> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        for service in &self.config.services {
            let virtual_service = k8s::istio::networking::v1beta1::VirtualService {
                metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(format!("{}-vs", service.name)),
                    namespace: Some(self.config.namespace.clone()),
                    labels: Some(HashMap::from([
                        ("beejs.io/service".to_string(), service.name.clone()),
                    ])),
                    ..Default::default()
                },
                spec: Some(k8s::istio::networking::v1beta1::VirtualServiceSpec {
                    hosts: Some(vec![service.name.clone()]),
                    gateways: Some(vec![format!("{}-gateway", service.name)]),
                    http: Some(vec![
                        k8s::istio::networking::v1beta1::HTTPRoute {
                            name: Some(format!("{}-route", service.name)),
                            match: None,
                            route: Some(vec![
                                k8s::istio::networking::v1beta1::HTTPRouteDestination {
                                    destination: Some(k8s::istio::networking::v1beta1::Destination {
                                        host: service.name.clone(),
                                        subset: Some("v1".to_string()),
                                        port: Some(k8s::istio::networking::v1beta1::PortSelector {
                                            number: Some(service.port),
                                        }),
                                    }),
                                    weight: Some(100),
                                    headers: None,
                                    fault: None,
                                    mirror: None,
                                    mirror_percent: None,
                                    timeout: None,
                                    retries: None,
                                    cors_policy: None,
                                    delegate: None,
                                    rewrite: None,
                                    websocket_upgrade: None,
                                    timeout_percent: None,
                                    meta: None,
                                },
                            ]),
                            websocket_upgrade: None,
                            timeout: None,
                            fault: None,
                            retry_policy: None,
                            mirror: None,
                            mirror_percent: None,
                            cors_policy: None,
                            append_headers: None,
                            remove_response_headers: None,
                            append_response_headers: None,
                            remove_request_headers: None,
                            append_request_headers: None,
                            direct_response: None,
                            delegate: None,
                            redirect: None,
                            match_outer: None,
                            query_params: None,
                            without_headers: None,
                            headers: None,
                        },
                    ]),
                    tls: None,
                    tcp: None,
                    export_to: None,
                }),
                status: None,
            };

            let params = k8s::api::PostParams::default();
            virtual_services.create(&params, &virtual_service).await?;

            info!("Created VirtualService: {}-vs", service.name);
        }

        Ok(())
    }

    /// Create Gateway
    async fn create_gateway(&self) -> Result<(), Error> {
        let gateways: Api<k8s::istio::networking::v1beta1::Gateway> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        for service in &self.config.services {
            let gateway = k8s::istio::networking::v1beta1::Gateway {
                metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(format!("{}-gateway", service.name)),
                    namespace: Some(self.config.namespace.clone()),
                    labels: Some(HashMap::from([
                        ("beejs.io/service".to_string(), service.name.clone()),
                    ])),
                    ..Default::default()
                },
                spec: Some(k8s::istio::networking::v1beta1::GatewaySpec {
                    selector: Some(HashMap::from([
                        ("istio".to_string(), "ingressgateway".to_string()),
                    ])),
                    servers: Some(vec![
                        k8s::istio::networking::v1beta1::Server {
                            port: Some(k8s::istio::networking::v1beta1::Port {
                                number: service.port,
                                name: service.name.clone(),
                                protocol: "HTTP".to_string(),
                            }),
                            hosts: Some(vec!["*".to_string()]),
                            tls: None,
                            default_endpoint: None,
                            bind: None,
                            servers: None,
                        },
                    ]),
                    default_config: None,
                }),
                status: None,
            };

            let params = k8s::api::PostParams::default();
            gateways.create(&params, &gateway).await?;

            info!("Created Gateway: {}-gateway", service.name);
        }

        Ok(())
    }

    /// Configure PeerAuthentication
    async fn configure_peer_authentication(&self) -> Result<(), Error> {
        if !self.config.mtls_enabled {
            return Ok(());
        }

        let peer_authentications: Api<k8s::istio::security::v1beta1::PeerAuthentication> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        let peer_authentication = k8s::istio::security::v1beta1::PeerAuthentication {
            metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("default".to_string()),
                namespace: Some(self.config.namespace.clone()),
                ..Default::default()
            },
            spec: Some(k8s::istio::security::v1beta1::PeerAuthenticationSpec {
                selector: None,
                mtls: Some(k8s::istio::security::v1beta1::PeerAuthenticationMutualTLS {
                    mode: k8s::istio::security::v1beta1::PeerAuthenticationMutualTLSMode::Strict,
                }),
                port_level_mtls: None,
            }),
            status: None,
        };

        let params = k8s::api::PostParams::default();
        peer_authentications.create(&params, &peer_authentication).await?;

        info!("Configured PeerAuthentication with STRICT mTLS");

        Ok(())
    }

    /// Configure AuthorizationPolicy
    async fn configure_authorization_policy(&self) -> Result<(), Error> {
        let authorization_policies: Api<k8s::istio::security::v1beta1::AuthorizationPolicy> =
            Api::namespaced(self.client.clone(), &self.config.namespace);

        for service in &self.config.services {
            let auth_policy = k8s::istio::security::v1beta1::AuthorizationPolicy {
                metadata: k8s::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(format!("{}-authz", service.name)),
                    namespace: Some(self.config.namespace.clone()),
                    labels: Some(HashMap::from([
                        ("beejs.io/service".to_string(), service.name.clone()),
                    ])),
                    ..Default::default()
                },
                spec: Some(k8s::istio::security::v1beta1::AuthorizationPolicySpec {
                    selector: Some(k8s::istio::networking::v1beta1::WorkloadSelector {
                        match_labels: Some(HashMap::from([
                            ("app".to_string(), service.name.clone()),
                        ])),
                        match_expressions: None,
                    }),
                    rules: Some(vec![
                        k8s::istio::security::v1beta1::Rule {
                            from: Some(vec![
                                k8s::istio::security::v1beta1::RuleFrom {
                                    source: Some(k8s::istio::security::v1beta1::Source {
                                        principals: None,
                                        requestPrincipals: None,
                                        ipBlocks: None,
                                        namespaces: None,
                                        notPrincipals: None,
                                        notIpBlocks: None,
                                        notNamespaces: None,
                                    }),
                                },
                            ]),
                            to: Some(vec![
                                k8s::istio::security::v1beta1::RuleTo {
                                    operation: Some(k8s::istio::security::v1beta1::Operation {
                                        methods: Some(vec!["GET".to_string(), "POST".to_string()]),
                                        hosts: None,
                                        paths: None,
                                        ports: None,
                                        notMethods: None,
                                        notHosts: None,
                                        notPaths: None,
                                        notPorts: None,
                                    }),
                                },
                            ]),
                            when: None,
                        },
                    ]),
                    action: Some(k8s::istio::security::v1beta1::AuthorizationPolicy_Action::Allow),
                    override: None,
                    provider: None,
                }),
                status: None,
            };

            let params = k8s::api::PostParams::default();
            authorization_policies.create(&params, &auth_policy).await?;

            info!("Created AuthorizationPolicy: {}-authz", service.name);
        }

        Ok(())
    }
}

/// Istio configuration
#[derive(Debug, Clone)]
pub struct IstioConfig {
    /// Namespace for Istio resources
    pub namespace: String,

    /// Enable mTLS
    pub mtls_enabled: bool,

    /// Services to configure
    pub services: Vec<IstioService>,

    /// Traffic policy configuration
    pub traffic_policy: TrafficPolicyConfig,
}

/// Istio service configuration
#[derive(Debug, Clone)]
pub struct IstioService {
    /// Service name
    pub name: String,

    /// Service port
    pub port: u32,

    /// Service protocol
    pub protocol: String,
}

/// Traffic policy configuration
#[derive(Debug, Clone)]
pub struct TrafficPolicyConfig {
    /// Load balancer algorithm
    pub load_balancer: LoadBalancerAlgorithm,

    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,

    /// Outlier detection settings
    pub outlier_detection: OutlierDetectionConfig,
}

/// Load balancer algorithm
#[derive(Debug, Clone)]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    LeastRequest,
    Random,
    ConsistentHash,
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: u32,

    /// Maximum pending requests
    pub max_pending_requests: u32,
}

/// Outlier detection configuration
#[derive(Debug, Clone)]
pub struct OutlierDetectionConfig {
    /// Consecutive errors before ejection
    pub consecutive_errors: u32,

    /// Interval between ejection sweeps
    pub interval: std::time::Duration,

    /// Base ejection time
    pub base_ejection_time: std::time::Duration,
}

/// Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_istio_config_creation() {
        let config = IstioConfig {
            namespace: "beejs-system".to_string(),
            mtls_enabled: true,
            services: vec![
                IstioService {
                    name: "beejs-api".to_string(),
                    port: 8080,
                    protocol: "HTTP".to_string(),
                },
            ],
            traffic_policy: TrafficPolicyConfig {
                load_balancer: LoadBalancerAlgorithm::LeastRequest,
                connection_pool: ConnectionPoolConfig {
                    max_connections: 100,
                    max_pending_requests: 10,
                },
                outlier_detection: OutlierDetectionConfig {
                    consecutive_errors: 5,
                    interval: std::time::Duration::from_secs(10),
                    base_ejection_time: std::time::Duration::from_secs(30),
                },
            },
        };

        assert_eq!(config.namespace, "beejs-system");
        assert!(config.mtls_enabled);
        assert_eq!(config.services.len(), 1);
        assert_eq!(config.services[0].name, "beejs-api");
        assert_eq!(config.services[0].port, 8080);
    }
}
