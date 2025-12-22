// Stage 29.2: 分布式负载均衡模块
// 提供一致性哈希、智能路由、流量熔断等功能

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use tracing::{debug, info, warn};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::atomic::AtomicU64;
use std::time::SystemTime;
use std::hash::{Hash, Hasher, DefaultHasher};

// ============================================================================
// 一致性哈希 (Consistent Hashing)
// ============================================================================
/// 哈希环配置
#[derive(Debug, Clone)]
pub struct HashRingConfig {
    /// 每个节点的虚拟节点数量
    pub virtual_nodes: usize,
    /// 哈希函数类型
    pub hash_function: String,
}
impl Default for HashRingConfig {
    fn default() -> Self {
        Self {
            virtual_nodes: 150,
            hash_function: "xxhash".to_string(),
        }
    }
}
/// 一致性哈希环
#[derive(Debug)]
pub struct ConsistentHashRing {
    config: HashRingConfig,
    /// 哈希环: hash_value -> node_id
    ring: RwLock<BTreeMap<u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String, u64, String>>,
    /// 节点及其虚拟节点数
    nodes: RwLock<HashMap<String, usize>>,
}
impl ConsistentHashRing {
    /// 创建新的哈希环
    pub fn new(config: HashRingConfig) -> Self {
        Self {
            config,
            ring: RwLock::new(BTreeMap::new()),
            nodes: RwLock::new(HashMap::new()),
        }
    }
    /// 计算哈希值 (xxhash 风格)
    fn hash(&self, key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    /// 添加节点到哈希环
    pub fn add_node(&mut self, node_id: &str, weight: usize) {
        let virtual_count: _ = (self.config.virtual_nodes * weight) / 100;
        let mut ring = self.ring.write().unwrap();
        let mut nodes = self.nodes.write().unwrap();
        // 添加虚拟节点
        for i in 0..virtual_count {
            let virtual_key: _ = format!("{}#{}, node_id", i, node_id);
            let hash: _ = self.hash(&virtual_key);
            ring.insert(hash, node_id.to_string());
        }
        nodes.insert(node_id.to_string(), virtual_count);
        debug!("Added node {} with {} virtual nodes", node_id, virtual_count);
    }
    /// 从哈希环移除节点
    pub fn remove_node(&mut self, node_id: &str) {
        let mut ring = self.ring.write().unwrap();
        let mut nodes = self.nodes.write().unwrap();
        if let Some(&virtual_count) = nodes.get(node_id) {
            // 移除虚拟节点
            for i in 0..virtual_count {
                let virtual_key: _ = format!("{}#{}, node_id", i, node_id);
                let hash: _ = self.hash(&virtual_key);
                ring.remove(&hash);
            }
            nodes.remove(node_id);
            debug!("Removed node {} with {} virtual nodes", node_id, virtual_count);
        }
    }
    /// 获取键对应的节点
    pub fn get_node(&self, key: &str) -> Option<String> {
        let ring: _ = self.ring.read().unwrap();
        if ring.is_empty() {
            return None;
        }
        let hash: _ = self.hash(key);
        // 找到大于等于 hash 的第一个节点
        if let Some((_, node_id)) = ring.range(hash..).next() {
            return Some(node_id.clone());
        }
        // 环形，回到开头
        ring.iter().next().map(|(_, node_id)| node_id.clone())
    }
    /// 获取键的副本节点列表
    pub fn get_replicas(&self, key: &str, count: usize) -> Vec<String> {
        let ring: _ = self.ring.read().unwrap();
        if ring.is_empty() {
            return vec![];
        }
        let hash: _ = self.hash(key);
        let mut replicas = Vec::new();
        let mut seen = std::collections::HashSet::new();
        // 从 hash 位置开始顺时针查找
        for (_, node_id) in ring.range(hash..).chain(ring.iter()) {
            if !seen.contains(node_id) {
                seen.insert(node_id.clone());
                replicas.push(node_id.clone());
                if replicas.len() >= count {
                    break;
                }
            }
        }
        replicas
    }
    /// 获取实际节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.read().unwrap().len()
    }
    /// 获取虚拟节点数量
    pub fn virtual_node_count(&self) -> usize {
        self.ring.read().unwrap().len()
    }
}
// ============================================================================
// 智能路由 (Intelligent Routing)
// ============================================================================
/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutingStrategy {
    /// 最低负载优先
    LeastLoaded,
    /// 最低延迟优先
    LowestLatency,
    /// 加权综合评分
    Weighted,
    /// 轮询
    RoundRobin,
    /// 会话粘滞
    Sticky,
    /// 随机
    Random,
}
impl Default for RoutingStrategy {
    fn default() -> Self {
        Self::Weighted
    }
}
/// 路由器配置
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub strategy: RoutingStrategy,
    pub health_weight: f64,
    pub load_weight: f64,
    pub latency_weight: f64,
}
impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Weighted,
            health_weight: 0.3,
            load_weight: 0.4,
            latency_weight: 0.3,
        }
    }
}
/// 节点度量信息
#[derive(Debug, Clone)]
struct NodeMetrics {
    health: f64,
    load: f64,
    latency: Duration,
    last_update: Instant,
}
impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            health: 1.0,
            load: 0.0,
            latency: Duration::from_millis(10),
            last_update: Instant::now(),
        }
    }
}
/// 智能路由器
#[derive(Debug)]
pub struct IntelligentRouter {
    config: RouterConfig,
    nodes: RwLock<Vec<String>>,
    metrics: RwLock<HashMap<String, NodeMetrics>>,
    round_robin_index: AtomicUsize,
    sticky_map: RwLock<HashMap<String, String>>,
}
impl IntelligentRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            nodes: RwLock::new(Vec::new()),
            metrics: RwLock::new(HashMap::new()),
            round_robin_index: AtomicUsize::new(0),
            sticky_map: RwLock::new(HashMap::new()),
        }
    }
    pub fn add_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().unwrap();
        let mut metrics = self.metrics.write().unwrap();
        if !nodes.contains(&node_id.to_string()) {
            nodes.push(node_id.to_string());
            metrics.insert(node_id.to_string(), NodeMetrics::default());
        }
    }
    pub fn remove_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().unwrap();
        let mut metrics = self.metrics.write().unwrap();
        nodes.retain(|n| n != node_id);
        metrics.remove(node_id);
    }
    pub fn update_node_health(&self, node_id: &str, health: f64) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(node_id) {
            m.health = health.clamp(0.0, 1.0);
            m.last_update = Instant::now();
        }
    }
    pub fn update_node_load(&self, node_id: &str, load: f64) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(node_id) {
            m.load = load.clamp(0.0, 1.0);
            m.last_update = Instant::now();
        }
    }
    pub fn update_node_latency(&self, node_id: &str, latency: Duration) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(m) = metrics.get_mut(node_id) {
            m.latency = latency;
            m.last_update = Instant::now();
        }
    }
    /// 路由请求到最佳节点
    pub fn route(&self, key: &str) -> Option<String> {
        match self.config.strategy {
            RoutingStrategy::LeastLoaded => self.route_least_loaded(),
            RoutingStrategy::LowestLatency => self.route_lowest_latency(),
            RoutingStrategy::Weighted => self.route_weighted(),
            RoutingStrategy::RoundRobin => self.route_round_robin(),
            RoutingStrategy::Sticky => self.route_sticky(key),
            RoutingStrategy::Random => self.route_random(),
        }
    }
    fn route_least_loaded(&self) -> Option<String> {
        let nodes: _ = self.nodes.read().unwrap();
        let metrics: _ = self.metrics.read().unwrap();
        nodes.iter()
            .filter_map(|n| metrics.get(n).map(|m| (n, m.load)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(n, _)| n.clone())
    }
    fn route_lowest_latency(&self) -> Option<String> {
        let nodes: _ = self.nodes.read().unwrap();
        let metrics: _ = self.metrics.read().unwrap();
        nodes.iter()
            .filter_map(|n| metrics.get(n).map(|m| (n, m.latency)))
            .min_by_key(|(_, latency)| *latency)
            .map(|(n, _)| n.clone())
    }
    fn route_weighted(&self) -> Option<String> {
        let nodes: _ = self.nodes.read().unwrap();
        let metrics: _ = self.metrics.read().unwrap();
        nodes.iter()
            .filter_map(|n| {
                metrics.get(n).map(|m| {
                    // 计算综合评分 (越高越好)
                    let health_score: _ = m.health * self.config.health_weight;
                    let load_score: _ = (1.0 - m.load) * self.config.load_weight;
                    let latency_score: _ = (1.0 - (m.latency.as_millis() as f64 / 1000.0).min(1.0))
                        * self.config.latency_weight;
                    let total_score: _ = health_score + load_score + latency_score;
                    (n, total_score)
                })
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(n, _)| n.clone())
    }
    fn route_round_robin(&self) -> Option<String> {
        let nodes: _ = self.nodes.read().unwrap();
        if nodes.is_empty() {
            return None;
        }
        let index: _ = self.round_robin_index.fetch_add(1, Ordering::SeqCst) % nodes.len();
        Some(nodes[index].clone())
    }
    fn route_sticky(&self, key: &str) -> Option<String> {
        // 检查是否已有映射
        {
            let sticky_map: _ = self.sticky_map.read().unwrap();
            if let Some(node) = sticky_map.get(key) {
                return Some(node.clone());
            }
        }
        // 选择一个节点并记录
        let selected: _ = self.route_round_robin()?;
        let mut sticky_map = self.sticky_map.write().unwrap();
        sticky_map.insert(key.to_string(), selected.clone());
        Some(selected)
    }
    fn route_random(&self) -> Option<String> {
        let nodes: _ = self.nodes.read().unwrap();
        if nodes.is_empty() {
            return None;
        }
        let seed: _ = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        let index: _ = seed % nodes.len();
        Some(nodes[index].clone())
    }
}
// ============================================================================
// 熔断器 (Circuit Breaker)
// ============================================================================
/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// 关闭 - 正常工作
    Closed,
    /// 打开 - 拒绝请求
    Open,
    /// 半开 - 允许有限请求测试
    HalfOpen,
}
/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 触发熔断的失败次数阈值
    pub failure_threshold: u32,
    /// 从半开恢复到关闭需要的成功次数
    pub success_threshold: u32,
    /// 熔断超时时间
    pub timeout: Duration,
    /// 半开状态允许的最大请求数
    pub half_open_max_calls: u32,
}
impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}
/// 熔断器统计信息
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub total_requests: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub failure_rate: f64,
    pub state: CircuitState,
    pub last_failure_time: Option<Instant>,
}
/// 熔断器
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    total_requests: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    last_state_change: RwLock<Instant>,
    half_open_calls: AtomicU64,
}
impl CircuitBreaker {
    pub fn new(name: &str, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            last_state_change: RwLock::new(Instant::now()),
            half_open_calls: AtomicU64::new(0),
        }
    }
    /// 检查是否允许请求
    pub fn allow_request(&self) -> bool {
        self.check_state_transition();
        let state: _ = *self.state.read().unwrap();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => {
                let calls: _ = self.half_open_calls.fetch_add(1, Ordering::SeqCst);
                calls < self.config.half_open_max_calls as u64
            }
        }
    }
    /// 记录成功请求
    pub fn record_success(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.success_count.fetch_add(1, Ordering::SeqCst);
        let mut state = self.state.write().unwrap();
        if *state == CircuitState::HalfOpen {
            let success_count: _ = self.success_count.load(Ordering::SeqCst);
            if success_count >= self.config.success_threshold as u64 {
                *state = CircuitState::Closed;
                self.reset_counters();
                *self.last_state_change.write().unwrap() = Instant::now();
                info!("Circuit breaker {} closed after recovery", self.name);
            }
        }
    }
    /// 记录失败请求
    pub fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        let failure_count: _ = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.write().unwrap() = Some(Instant::now());
        let mut state = self.state.write().unwrap();
        match *state {
            CircuitState::Closed => {
                if failure_count >= self.config.failure_threshold as u64 {
                    *state = CircuitState::Open;
                    *self.last_state_change.write().unwrap() = Instant::now();
                    warn!("Circuit breaker {} opened after {} failures", self.name, failure_count);
                }
            }
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                self.half_open_calls.store(0, Ordering::SeqCst);
                *self.last_state_change.write().unwrap() = Instant::now();
                warn!("Circuit breaker {} re-opened from half-open state", self.name);
            }
            CircuitState::Open => {}
        }
    }
    /// 检查状态转换
    fn check_state_transition(&self) {
        let mut state = self.state.write().unwrap();
        if *state == CircuitState::Open {
            let last_change: _ = *self.last_state_change.read().unwrap();
            if last_change.elapsed() >= self.config.timeout {
                *state = CircuitState::HalfOpen;
                self.half_open_calls.store(0, Ordering::SeqCst);
                self.success_count.store(0, Ordering::SeqCst);
                *self.last_state_change.write().unwrap() = Instant::now();
                info!("Circuit breaker {} transitioned to half-open", self.name);
            }
        }
    }
    fn reset_counters(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        self.half_open_calls.store(0, Ordering::SeqCst);
    }
    pub fn is_closed(&self) -> bool {
        self.check_state_transition();
        *self.state.read().unwrap() == CircuitState::Closed
    }
    pub fn is_open(&self) -> bool {
        self.check_state_transition();
        *self.state.read().unwrap() == CircuitState::Open
    }
    pub fn is_half_open(&self) -> bool {
        self.check_state_transition();
        *self.state.read().unwrap() == CircuitState::HalfOpen
    }
    pub fn get_statistics(&self) -> CircuitBreakerStats {
        let total: _ = self.total_requests.load(Ordering::SeqCst);
        let failures: _ = self.failure_count.load(Ordering::SeqCst);
        let successes: _ = self.success_count.load(Ordering::SeqCst);
        CircuitBreakerStats {
            total_requests: total,
            success_count: successes,
            failure_count: failures,
            failure_rate: if total > 0 { failures as f64 / total as f64 } else { 0.0 },
            state: *self.state.read().unwrap(),
            last_failure_time: *self.last_failure_time.read().unwrap(),
        }
    }
}
/// 熔断器注册表
#[derive(Debug)]
pub struct CircuitBreakerRegistry {
    config: CircuitBreakerConfig,
    breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
}
impl CircuitBreakerRegistry {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            breakers: RwLock::new(HashMap::new()),
        }
    }
    pub fn get_or_create(&self, service_name: &str) -> Arc<CircuitBreaker> {
        // 先尝试读取
        {
            let breakers: _ = self.breakers.read().unwrap();
            if let Some(breaker) = breakers.get(service_name) {
                return breaker.clone();
            }
        }
        // 不存在则创建
        let mut breakers = self.breakers.write().unwrap();
        breakers.entry(service_name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(CircuitBreaker::new(service_name, self.config.clone()))))
            .clone()
    }
    pub fn breaker_count(&self) -> usize {
        self.breakers.read().unwrap().len()
    }
}
// ============================================================================
// 负载均衡器 (Load Balancer)
// ============================================================================
/// 负载均衡器配置
#[derive(Debug, Clone)]
pub struct LoadBalancerConfig {
    pub strategy: RoutingStrategy,
    pub enable_circuit_breaker: bool,
    pub enable_health_check: bool,
    pub virtual_nodes: usize,
}
impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Weighted,
            enable_circuit_breaker: true,
            enable_health_check: true,
            virtual_nodes: 100,
        }
    }
}
/// 后端节点信息
#[derive(Debug, Clone)]
pub struct Backend {
    pub id: String,
    pub address: String,
    pub weight: usize,
    pub healthy: bool,
}
/// 请求
#[derive(Debug, Clone)]
pub struct Request {
    pub id: String,
    pub key: String,
    pub payload: Vec<u8>,
}
impl Request {
    pub fn new(id: &str, key: &str) -> Self {
        Self {
            id: id.to_string(),
            key: key.to_string(),
            payload: vec![],
        }
    }
}
/// 负载均衡器统计
#[derive(Debug, Clone)]
pub struct LoadBalancerStats {
    pub total_backends: usize,
    pub healthy_backends: usize,
    pub total_requests: u64,
    pub avg_latency: Duration,
}
/// 负载均衡器
#[derive(Debug)]
pub struct LoadBalancer {
    config: LoadBalancerConfig,
    hash_ring: RwLock<ConsistentHashRing>,
    router: IntelligentRouter,
    circuit_breakers: CircuitBreakerRegistry,
    backends: RwLock<HashMap<String, Backend>>,
    request_count: AtomicU64,
    total_latency_ns: AtomicU64,
}
impl LoadBalancer {
    pub fn new(config: LoadBalancerConfig) -> Self {
        let hash_config: _ = HashRingConfig {
            virtual_nodes: config.virtual_nodes,
            ..Default::default()
        };
        let router_config: _ = RouterConfig {
            strategy: config.strategy,
            ..Default::default()
        };
        Self {
            config: config.clone(),
            hash_ring: RwLock::new(ConsistentHashRing::new(hash_config)),
            router: IntelligentRouter::new(router_config),
            circuit_breakers: CircuitBreakerRegistry::new(CircuitBreakerConfig::default()),
            backends: RwLock::new(HashMap::new()),
            request_count: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
        }
    }
    pub async fn add_backend(&self, id: &str, address: &str, weight: usize) {
        let backend: _ = Backend {
            id: id.to_string(),
            address: address.to_string(),
            weight,
            healthy: true,
        };
        // 添加到后端列表
        self.backends.write().unwrap().insert(id.to_string(), backend);
        // 添加到哈希环
        self.hash_ring.write().unwrap().add_node(id, weight);
        // 添加到路由器
        self.router.add_node(id);
        info!("Added backend {} at {} with weight {}", id, address, weight);
    }
    pub async fn remove_backend(&self, id: &str) {
        self.backends.write().unwrap().remove(id);
        self.hash_ring.write().unwrap().remove_node(id);
        self.router.remove_node(id);
        info!("Removed backend {}", id);
    }
    pub async fn update_backend_load(&self, id: &str, load: f64) {
        self.router.update_node_load(id, load);
    }
    pub async fn mark_backend_unhealthy(&self, id: &str) {
        if let Some(backend) = self.backends.write().unwrap().get_mut(id) {
            backend.healthy = false;
        }
        self.router.update_node_health(id, 0.0);
        warn!("Backend {} marked as unhealthy", id);
    }
    pub async fn mark_backend_healthy(&self, id: &str) {
        if let Some(backend) = self.backends.write().unwrap().get_mut(id) {
            backend.healthy = true;
        }
        self.router.update_node_health(id, 1.0);
        info!("Backend {} marked as healthy", id);
    }
    pub async fn route_request(&self, request: &Request) -> Result<Backend, String> {
        let start: _ = Instant::now();
        self.request_count.fetch_add(1, Ordering::SeqCst);
        // 使用路由器选择节点
        let node_id: _ = self.router.route(&request.key)
            .ok_or_else(|| "No available backends".to_string())?;
        // 检查熔断器
        if self.config.enable_circuit_breaker {
            let breaker: _ = self.circuit_breakers.get_or_create(&node_id);
            if !breaker.allow_request() {
                return Err(format!("Circuit breaker open for {}", node_id));
            }
        }
        // 获取后端信息
        let backend: _ = self.backends.read().unwrap()
            .get(&node_id)
            .filter(|b| b.healthy)
            .cloned()
            .ok_or_else(|| format!("Backend {} not available", node_id))?;
        // 记录延迟
        let latency: _ = start.elapsed();
        self.total_latency_ns.fetch_add(latency.as_nanos() as u64, Ordering::SeqCst);
        Ok(backend)
    }
    pub async fn get_stats(&self) -> LoadBalancerStats {
        let backends: _ = self.backends.read().unwrap();
        let total: _ = backends.len();
        let healthy: _ = backends.values().filter(|b| b.healthy).count();
        let requests: _ = self.request_count.load(Ordering::SeqCst);
        let total_latency: _ = self.total_latency_ns.load(Ordering::SeqCst);
        let avg_latency: _ = if requests > 0 {
            Duration::from_nanos(total_latency / requests)
        } else {
            Duration::ZERO
        };
        LoadBalancerStats {
            total_backends: total,
            healthy_backends: healthy,
            total_requests: requests,
            avg_latency,
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_consistent_hash_basic() {
        let config: _ = HashRingConfig::default();
        let mut ring = ConsistentHashRing::new(config);
        ring.add_node("node-1", 100);
        assert_eq!(ring.node_count(), 1);
        let node: _ = ring.get_node("test-key");
        assert!(node.is_some());
    }
    #[test]
    fn test_circuit_breaker_basic() {
        let config: _ = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let breaker: _ = CircuitBreaker::new("test", config);
        assert!(breaker.is_closed());
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.is_open());
    }
}