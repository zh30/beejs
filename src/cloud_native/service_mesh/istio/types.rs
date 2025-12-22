//! Local Istio CRD type definitions
//! These are simplified versions of Istio CRDs for use with kube-rs

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// DestinationRule defines policies for traffic to a service
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "networking.istio.io",
    version = "v1beta1",
    kind = "DestinationRule",
    namespaced
)]
pub struct DestinationRuleSpec {
    /// Host name of the service
    pub host: String,
    /// Traffic policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic_policy: Option<TrafficPolicy>,
    /// Subsets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsets: Option<Vec<Subset>>,
}

/// Traffic policy for a destination
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct TrafficPolicy {
    /// Load balancer settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_balancer: Option<LoadBalancerSettings>,
    /// Connection pool settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_pool: Option<ConnectionPoolSettings>,
    /// Outlier detection settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outlier_detection: Option<OutlierDetection>,
}

/// Load balancer settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct LoadBalancerSettings {
    /// Simple load balancer algorithm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simple: Option<String>,
}

/// Connection pool settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct ConnectionPoolSettings {
    /// TCP settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp: Option<TcpSettings>,
    /// HTTP settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpSettings>,
}

/// TCP connection settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct TcpSettings {
    /// Max connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<i32>,
    /// Connect timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_timeout: Option<String>,
}

/// HTTP connection settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpSettings {
    /// HTTP1 max pending requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h2_upgrade_policy: Option<String>,
    /// Max requests per connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_requests_per_connection: Option<i32>,
}

/// Outlier detection settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct OutlierDetection {
    /// Consecutive errors before ejection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consecutive_errors: Option<i32>,
    /// Interval between ejection sweeps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    /// Base ejection time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_ejection_time: Option<String>,
    /// Max ejection percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ejection_percent: Option<i32>,
}

/// Subset of a service
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Subset {
    /// Subset name
    pub name: String,
    /// Labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
}

/// VirtualService defines traffic routing rules
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "networking.istio.io",
    version = "v1beta1",
    kind = "VirtualService",
    namespaced
)]
pub struct VirtualServiceSpec {
    /// Hosts this rule applies to
    pub hosts: Vec<String>,
    /// Gateways to apply this rule to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateways: Option<Vec<String>>,
    /// HTTP routing rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<Vec<HttpRoute>>,
}

/// HTTP route
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpRoute {
    /// Match conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#match: Option<Vec<HttpMatchRequest>>,
    /// Route destinations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<Vec<HttpRouteDestination>>,
    /// Fault injection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault: Option<HttpFaultInjection>,
    /// Timeout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    /// Retries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<HttpRetry>,
}

/// HTTP match request
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpMatchRequest {
    /// URI match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<StringMatch>,
    /// Headers match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, StringMatch, std::collections::HashMap<String, StringMatch, String, StringMatch>>>,
}

/// String match
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct StringMatch {
    /// Exact match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact: Option<String>,
    /// Prefix match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Regex match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
}

/// HTTP route destination
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpRouteDestination {
    /// Destination
    pub destination: Destination,
    /// Weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}

/// Destination
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Destination {
    /// Host name
    pub host: String,
    /// Subset name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subset: Option<String>,
    /// Port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<PortSelector>,
}

/// Port selector
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct PortSelector {
    /// Port number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<u32>,
}

/// HTTP fault injection
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpFaultInjection {
    /// Delay injection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<Delay>,
    /// Abort injection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abort: Option<Abort>,
}

/// Delay injection
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Delay {
    /// Fixed delay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_delay: Option<String>,
    /// Percentage of requests to delay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<Percent>,
}

/// Abort injection
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Abort {
    /// HTTP status to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<i32>,
    /// Percentage of requests to abort
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<Percent>,
}

/// Percentage
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Percent {
    /// Value (0-100)
    pub value: f64,
}

/// HTTP retry policy
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct HttpRetry {
    /// Number of retries
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<i32>,
    /// Retry timeout per attempt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_try_timeout: Option<String>,
    /// Retry on conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_on: Option<String>,
}

/// Gateway defines an entry point into the mesh
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "networking.istio.io",
    version = "v1beta1",
    kind = "Gateway",
    namespaced
)]
pub struct GatewaySpec {
    /// Workload selector
    pub selector: HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
    /// Servers
    pub servers: Vec<Server>,
}

/// Server in a gateway
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Server {
    /// Port
    pub port: Port,
    /// Hosts
    pub hosts: Vec<String>,
    /// TLS settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsSettings>,
}

/// Port
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Port {
    /// Port number
    pub number: u32,
    /// Protocol
    pub protocol: String,
    /// Name
    pub name: String,
}

/// TLS settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct TlsSettings {
    /// TLS mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Credential name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_name: Option<String>,
}

/// PeerAuthentication defines mTLS settings
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "security.istio.io",
    version = "v1beta1",
    kind = "PeerAuthentication",
    namespaced
)]
pub struct PeerAuthenticationSpec {
    /// Workload selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<WorkloadSelector>,
    /// mTLS settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtls: Option<MutualTls>,
}

/// Workload selector
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct WorkloadSelector {
    /// Match labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_labels: Option<HashMap<String, String, std::collections::HashMap<String, String, String, String>>>,
}

/// Mutual TLS settings
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct MutualTls {
    /// mTLS mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// AuthorizationPolicy defines access control rules
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "security.istio.io",
    version = "v1beta1",
    kind = "AuthorizationPolicy",
    namespaced
)]
pub struct AuthorizationPolicySpec {
    /// Workload selector
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<WorkloadSelector>,
    /// Action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Rules
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<AuthorizationRule>>,
}

/// Authorization rule
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct AuthorizationRule {
    /// From sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Vec<Source>>,
    /// To destinations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<Operation>>,
}

/// Source of a request
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Source {
    /// Principals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principals: Option<Vec<String>>,
    /// Namespaces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespaces: Option<Vec<String>>,
}

/// Operation on a target
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Operation {
    /// Paths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
    /// Methods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
}
