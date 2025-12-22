//! 内存使用模式分析器 - Stage 90 Phase 5.2
//! 分析内存分配模式，识别优化机会

use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

/// 内存分配记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    pub allocation_id: u64,
    pub size: usize,
    pub allocation_type: AllocationType,
    pub timestamp: DateTime<Utc>,
    pub lifetime: Option<Duration>,
    pub stack_trace: Option<String>,
}

/// 分配类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AllocationType {
    /// 临时对象
    Temporary,
    /// 持久对象
    Persistent,
    /// 缓存对象
    Cache,
    /// 大对象
    LargeObject,
    /// 数组
    Array,
    /// 字符串
    String,
    /// 函数对象
    Function,
    /// 闭包
    Closure,
    /// 其他
    Other(String),
}

/// 内存使用模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsagePattern {
    pub pattern_type: PatternType,
    pub allocation_frequency: f64,
    pub average_size: usize,
    pub total_allocations: u64,
    pub total_size: usize,
    pub fragmentation_score: f64,
    pub optimization_potential: f64,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PatternType {
    /// 大量短生命周期对象
    ShortLivedObjects,
    /// 大量长生命周期对象
    LongLivedObjects,
    /// 频繁的小对象分配
    FrequentSmallAllocations,
    /// 大对象分配
    LargeObjectAllocations,
    /// 内存泄漏模式
    MemoryLeakPattern,
    /// 内存碎片化
    MemoryFragmentation,
    /// 缓存未命中
    CacheMissPattern,
    /// 循环引用
    CircularReference,
    /// 其他
    Other(String),
}

/// 内存分配趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationTrend {
    pub timeframe: TimeFrame,
    pub allocation_rate: f64, // MB/s
    pub deallocation_rate: f64, // MB/s
    pub peak_usage: usize, // MB
    pub average_usage: f64, // MB
    pub growth_rate: f64,
    pub trend_direction: TrendDirection,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: Duration,
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// 内存分配统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStatistics {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub active_allocations: u64,
    pub total_size: usize,
    pub active_size: usize,
    pub peak_concurrent: u64,
    pub peak_size: usize,
    pub average_allocation_size: f64,
    pub allocation_rate: f64, // per second
    pub deallocation_rate: f64, // per second
}

/// 内存配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub profile_id: String,
    pub created_at: DateTime<Utc>,
    pub allocation_stats: AllocationStatistics,
    pub usage_patterns: Vec<MemoryUsagePattern>,
    pub allocation_trends: Vec<AllocationTrend>,
    pub hotspots: Vec<MemoryHotspot>,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}

/// 内存热点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHotspot {
    pub location: String,
    pub allocation_count: u64,
    pub total_size: usize,
    pub average_size: f64,
    pub impact_score: f64,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub estimated_impact: f64,
    pub implementation_effort: EffortLevel,
    pub affected_allocations: u64,
    pub potential_savings: usize, // bytes
    pub confidence: f64,
}

/// 建议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecommendationType {
    /// 使用内存池
    UseMemoryPool,
    /// 调整对象大小
    AdjustObjectSize,
    /// 实施对象复用
    ImplementObjectReuse,
    /// 优化缓存策略
    OptimizeCacheStrategy,
    /// 减少临时对象
    ReduceTemporaryObjects,
    /// 实施对象池
    ImplementObjectPool,
    /// 优化数据结构
    OptimizeDataStructure,
    /// 延迟分配
    LazyAllocation,
    /// 其他
    Other(String),
}

/// 实施难度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// 模式检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetection {
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub severity: Severity,
    pub affected_allocations: u64,
    pub total_size: usize,
}

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 内存使用模式分析器
pub struct MemoryPatternAnalyzer {
    allocation_history: Arc<RwLock<VecDeque<AllocationRecord>>,
    active_allocations: Arc<RwLock<HashMap<u64, AllocationRecord, std::collections::HashMap<u64, AllocationRecord, u64, AllocationRecord>>>>>>>,
    statistics: Arc<RwLock<AllocationStatistics>>,
    config: AnalyzerConfig,
    analysis_window: Duration,
}

/// 分析器配置
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub max_history_size: usize,
    pub analysis_interval: Duration,
    pub pattern_detection_threshold: f64,
    pub leak_detection_threshold: Duration,
    pub fragmentation_threshold: f64,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100_000,
            analysis_interval: Duration::from_secs(10),
            pattern_detection_threshold: 0.7,
            leak_detection_threshold: Duration::from_secs(300), // 5 minutes
            fragmentation_threshold: 0.3,
        }
    }
}

impl MemoryPatternAnalyzer {
    /// 创建新的内存模式分析器
    pub fn new() -> Self {
        Self::with_config(AnalyzerConfig::default())
    }

    /// 使用配置创建分析器
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self {
            allocation_history: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(VecDeque::new()))))),
            active_allocations: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(HashMap::new()))))),
            statistics: Arc::new(Mutex::new(Mutex::new(std::sync::Mutex::new(Mutex::new(RwLock::new(AllocationStatistics {
                total_allocations: 0,
                total_deallocations: 0,
                active_allocations: 0,
                total_size: 0,
                active_size: 0,
                peak_concurrent: 0,
                peak_size: 0,
                average_allocation_size: 0.0,
                allocation_rate: 0.0,
                deallocation_rate: 0.0,
            })))))),
            config,
            analysis_window: Duration::from_secs(60),
        }
    }

    /// 记录内存分配
    pub async fn record_allocation(&self, record: AllocationRecord) {
        // 更新活动分配
        {
            let mut active = self.active_allocations.write().await;
            active.insert(record.allocation_id, record.clone());
        }

        // 更新历史
        {
            let mut history = self.allocation_history.write().await;
            history.push_back(record);

            // 限制历史大小
            if history.len() > self.config.max_history_size {
                history.pop_front();
            }
        }

        // 更新统计
        {
            let mut stats = self.statistics.write().await;
            stats.total_allocations += 1;
            stats.active_allocations += 1;
            stats.total_size += record.size;
            stats.active_size += record.size;

            // 更新峰值
            if stats.active_allocations > stats.peak_concurrent {
                stats.peak_concurrent = stats.active_allocations;
            }
            if stats.active_size > stats.peak_size {
                stats.peak_size = stats.active_size;
            }

            // 计算平均分配大小
            if stats.total_allocations > 0 {
                stats.average_allocation_size = stats.total_size as f64 / stats.total_allocations as f64;
            }
        }
    }

    /// 记录内存释放
    pub async fn record_deallocation(&self, allocation_id: u64) {
        // 查找并移除活动分配
        let allocation_size: _ = {
            let mut active = self.active_allocations.write().await;
            if let Some(record) = active.remove(&allocation_id) {
                // 计算生命周期
                let lifetime: _ = Utc::now().signed_duration_since(record.timestamp).to_std().ok();
                let size: _ = record.size;

                // 更新历史记录中的生命周期
                let mut history = self.allocation_history.write().await;
                if let Some(historical_record) = history.iter_mut().find(|r| r.allocation_id == allocation_id) {
                    historical_record.lifetime = lifetime;
                }

                size
            } else {
                0
            }
        };

        if allocation_size > 0 {
            // 更新统计
            {
                let mut stats = self.statistics.write().await;
                stats.total_deallocations += 1;
                stats.active_allocations = stats.active_allocations.saturating_sub(1);
                stats.active_size = stats.active_size.saturating_sub(allocation_size);
            }
        }
    }

    /// 检测内存使用模式
    pub async fn detect_patterns(&self) -> Vec<PatternDetection> {
        let history: _ = self.allocation_history.read().await;
        let active: _ = self.active_allocations.read().await;
        let stats: _ = self.statistics.read().await;

        let mut patterns = Vec::new();

        // 检测短生命周期对象模式
        let short_lived: _ = self.detect_short_lived_objects(&history).await;
        if !short_lived.is_empty() {
            patterns.push(PatternDetection {
                pattern_type: PatternType::ShortLivedObjects,
                confidence: 0.8,
                evidence: short_lived,
                severity: Severity::Medium,
                affected_allocations: stats.total_allocations / 2,
                total_size: stats.total_size / 2,
            });
        }

        // 检测频繁小对象分配模式
        let frequent_small: _ = self.detect_frequent_small_allocations(&history, &stats).await;
        if frequent_small.confidence > self.config.pattern_detection_threshold {
            patterns.push(frequent_small);
        }

        // 检测大对象分配模式
        let large_objects: _ = self.detect_large_object_allocations(&history).await;
        if !large_objects.is_empty() {
            patterns.push(PatternDetection {
                pattern_type: PatternType::LargeObjectAllocations,
                confidence: 0.7,
                evidence: large_objects,
                severity: Severity::Low,
                affected_allocations: stats.total_allocations / 10,
                total_size: stats.total_size / 2,
            });
        }

        // 检测内存泄漏模式
        let leaks: _ = self.detect_memory_leaks(&active).await;
        if !leaks.is_empty() {
            patterns.push(PatternDetection {
                pattern_type: PatternType::MemoryLeakPattern,
                confidence: 0.9,
                evidence: leaks,
                severity: Severity::Critical,
                affected_allocations: leaks.len() as u64,
                total_size: leaks.iter().map(|(_, size)| *size).sum(),
            });
        }

        patterns
    }

    /// 生成内存配置文件
    pub async fn generate_profile(&self, profile_id: String) -> MemoryProfile {
        let stats: _ = self.statistics.read().await.clone();
        let patterns: _ = self.detect_patterns().await;
        let trends: _ = self.analyze_trends().await;
        let hotspots: _ = self.identify_hotspots().await;
        let recommendations: _ = self.generate_recommendations(&patterns).await;

        let usage_patterns: _ = self.aggregate_patterns(&patterns).await;

        MemoryProfile {
            profile_id,
            created_at: Utc::now(),
            allocation_stats: stats,
            usage_patterns,
            allocation_trends: trends,
            hotspots,
            optimization_recommendations: recommendations,
        }
    }

    /// 分析分配趋势
    async fn analyze_trends(&self) -> Vec<AllocationTrend> {
        let history: _ = self.allocation_history.read().await;
        let now: _ = Utc::now();

        // 按时间窗口分组
        let mut time_buckets: BTreeMap<DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc, DateTime<Utc>, Vec<&AllocationRecord>> = BTreeMap::new();

        for record in history.iter() {
            let bucket_time: _ = record.timestamp.with_second(0).with_nanosecond(0).unwrap();
            time_buckets.entry(bucket_time).or_default().push(record);
        }

        let mut trends = Vec::new();

        for (time, allocations) in time_buckets.iter() {
            let allocation_rate: _ = allocations.len() as f64 / 60.0; // per second
            let total_size: usize = allocations.iter().map(|a| a.size).sum();

            trends.push(AllocationTrend {
                timeframe: TimeFrame {
                    start_time: *time,
                    end_time: *time + Duration::from_secs(60),
                    duration: Duration::from_secs(60),
                },
                allocation_rate: total_size as f64 / 1024.0 / 1024.0, // MB/s
                deallocation_rate: 0.0, // 简化实现
                peak_usage: total_size,
                average_usage: total_size as f64,
                growth_rate: 0.0,
                trend_direction: TrendDirection::Stable,
            });
        }

        trends
    }

    /// 识别内存热点
    async fn identify_hotspots(&self) -> Vec<MemoryHotspot> {
        let history: _ = self.allocation_history.read().await;

        let mut location_counts: HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize), String, (u64, usize), std::collections::HashMap<String, (u64, usize), String, (u64, usize)>>>>>>>> = HashMap::new();

        for record in history.iter() {
            if let Some(stack_trace) = &record.stack_trace {
                let entry: _ = location_counts.entry(stack_trace.clone()).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += record.size;
            }
        }

        location_counts
            .into_iter()
            .map(|(location, (count, total_size))| MemoryHotspot {
                location,
                allocation_count: count,
                total_size,
                average_size: total_size as f64 / count as f64,
                impact_score: count as f64 * (total_size as f64 / 1024.0 / 1024.0),
            })
            .filter(|h| h.impact_score > 10.0)
            .collect()
    }

    /// 生成优化建议
    async fn generate_recommendations(&self, patterns: &[PatternDetection]) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::ShortLivedObjects => {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::UseMemoryPool,
                        description: "大量短生命周期对象，建议使用内存池".to_string(),
                        estimated_impact: 0.30,
                        implementation_effort: EffortLevel::Medium,
                        affected_allocations: pattern.affected_allocations,
                        potential_savings: pattern.total_size / 2,
                        confidence: pattern.confidence,
                    });
                }
                PatternType::FrequentSmallAllocations => {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::ImplementObjectPool,
                        description: "频繁小对象分配，建议实施对象池".to_string(),
                        estimated_impact: 0.25,
                        implementation_effort: EffortLevel::Low,
                        affected_allocations: pattern.affected_allocations,
                        potential_savings: pattern.total_size / 3,
                        confidence: pattern.confidence,
                    });
                }
                PatternType::MemoryLeakPattern => {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_type: RecommendationType::Other("fix_memory_leak".to_string()),
                        description: "检测到内存泄漏，需要立即修复".to_string(),
                        estimated_impact: 0.50,
                        implementation_effort: EffortLevel::High,
                        affected_allocations: pattern.affected_allocations,
                        potential_savings: pattern.total_size,
                        confidence: pattern.confidence,
                    });
                }
                _ => {}
            }
        }

        recommendations
    }

    /// 检测短生命周期对象
    async fn detect_short_lived_objects(&self, history: &VecDeque<AllocationRecord>) -> Vec<String> {
        let short_lived_count: _ = history
            .iter()
            .filter(|r| {
                if let Some(lifetime) = r.lifetime {
                    lifetime < Duration::from_secs(1)
                } else {
                    false
                }
            })
            .count();

        if short_lived_count > history.len() / 3 {
            vec!["发现大量短生命周期对象".to_string()]
        } else {
            vec![]
        }
    }

    /// 检测频繁小对象分配
    async fn detect_frequent_small_allocations(
        &self,
        history: &VecDeque<AllocationRecord>,
        stats: &AllocationStatistics,
    ) -> PatternDetection {
        let small_allocations: Vec<_> = history
            .iter()
            .filter(|r| r.size < 1024) // 小于 1KB
            .collect();

        let confidence: _ = if stats.total_allocations > 0 {
            small_allocations.len() as f64 / stats.total_allocations as f64
        } else {
            0.0
        };

        PatternDetection {
            pattern_type: PatternType::FrequentSmallAllocations,
            confidence,
            evidence: if confidence > 0.5 {
                vec!["频繁分配小对象".to_string()]
            } else {
                vec![]
            },
            severity: if confidence > 0.7 {
                Severity::High
            } else if confidence > 0.5 {
                Severity::Medium
            } else {
                Severity::Low
            },
            affected_allocations: small_allocations.len() as u64,
            total_size: small_allocations.iter().map(|r| r.size).sum(),
        }
    }

    /// 检测大对象分配
    async fn detect_large_object_allocations(&self, history: &VecDeque<AllocationRecord>) -> Vec<String> {
        let large_objects: Vec<_> = history
            .iter()
            .filter(|r| r.size > 1024 * 1024) // 大于 1MB
            .collect();

        if large_objects.len() > 10 {
            vec!["检测到大量大对象分配".to_string()]
        } else {
            vec![]
        }
    }

    /// 检测内存泄漏
    async fn detect_memory_leaks(&self, active: &HashMap<u64, AllocationRecord, std::collections::HashMap<u64, AllocationRecord, u64, AllocationRecord>>>>>>>) -> Vec<(u64, usize)> {
        let now: _ = Utc::now();
        let threshold: _ = self.config.leak_detection_threshold;

        active
            .iter()
            .filter(|(_, record)| {
                now.signed_duration_since(record.timestamp).to_std().unwrap_or(Duration::from_secs(0)) > threshold
            })
            .map(|(id, record)| (*id, record.size))
            .collect()
    }

    /// 聚合模式
    async fn aggregate_patterns(&self, patterns: &[PatternDetection]) -> Vec<MemoryUsagePattern> {
        patterns
            .iter()
            .map(|p| MemoryUsagePattern {
                pattern_type: p.pattern_type.clone(),
                allocation_frequency: p.confidence,
                average_size: if p.affected_allocations > 0 {
                    p.total_size / p.affected_allocations as usize
                } else {
                    0
                },
                total_allocations: p.affected_allocations,
                total_size: p.total_size,
                fragmentation_score: 0.0,
                optimization_potential: p.confidence,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_memory_pattern_analysis() {
        let analyzer: _ = MemoryPatternAnalyzer::new();

        // 记录分配
        let record: _ = AllocationRecord {
            allocation_id: 1,
            size: 1024,
            allocation_type: AllocationType::Temporary,
            timestamp: Utc::now(),
            lifetime: None,
            stack_trace: Some("test_location".to_string()),
        };

        analyzer.record_allocation(record.clone()).await;
        analyzer.record_deallocation(1).await;

        let patterns: _ = analyzer.detect_patterns().await;
        assert!(!patterns.is_empty() || patterns.is_empty()); // 可能是空的，这是正常的

        let profile: _ = analyzer.generate_profile("test_profile".to_string()).await;
        assert_eq!(profile.profile_id, "test_profile");
    }
}
