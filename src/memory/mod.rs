//! Beejs 内存管理模块
//! Stage 30.2: 内存管理深度优化
//!
//! 提供零拷贝内存分配、分代垃圾回收、内存压缩和泄漏检测的完整解决方案

pub mod zero_copy_allocator;
pub mod generational_gc;
pub mod memory_compression;
pub mod leak_detector;

// 重新导出主要类型
pub use zero_copy_allocator::{
    ZeroCopyAllocator,
    AllocatorConfig,
    AllocatorStats,
    AllocatorStatsSnapshot,
};

pub use generational_gc::{
    GenerationalGC,
    GCConfig,
    GCStats,
    GCStatsSnapshot,
};

pub use memory_compression::{
    MemoryCompression,
    CompressionConfig,
    CompressionStats,
    CompressionStatsSnapshot,
    CompressionAlgorithm,
    CompressedBlock,
};

pub use leak_detector::{
    MemoryLeakDetector,
    LeakDetectorConfig,
    LeakDetectionStats,
    LeakDetectionStatsSnapshot,
    ObjectType,
    LeakReport,
    LeakDetail,
    LeakSeverity,
};

use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 统一内存管理器 - 整合所有内存优化组件
/// 提供一站式的内存管理解决方案
pub struct UnifiedMemoryManager {
    /// 零拷贝分配器
    allocator: Arc<ZeroCopyAllocator>,
    /// 分代垃圾回收器
    gc: Arc<Mutex<GenerationalGC>>,
    /// 内存压缩器
    compressor: Arc<Mutex<MemoryCompression>>,
    /// 泄漏检测器
    leak_detector: Arc<Mutex<MemoryLeakDetector>>,
    /// 集成统计
    integrated_stats: Arc<IntegratedStats>,
    /// 创建时间
    created_at: Instant,
}

/// 集成统计信息
#[derive(Debug, Clone)]
pub struct IntegratedStats {
    /// 总内存分配 (字节)
    pub total_allocated_bytes: u64,
    /// 总内存释放 (字节)
    pub total_freed_bytes: u64,
    /// 当前活跃内存 (字节)
    pub active_memory_bytes: usize,
    /// 零拷贝分配率
    pub zero_copy_ratio: f64,
    /// GC 效率
    pub gc_efficiency: f64,
    /// 压缩效率
    pub compression_efficiency: f64,
    /// 泄漏检测率
    pub leak_detection_rate: f64,
    /// 总体内存节省率
    pub total_savings_ratio: f64,
}

impl UnifiedMemoryManager {
    /// 创建新的统一内存管理器
    pub fn new() -> Self {
        Self::with_configs(
            AllocatorConfig::default(),
            GCConfig::default(),
            CompressionConfig::default(),
            LeakDetectorConfig::default(),
        )
    }

    /// 使用自定义配置创建统一内存管理器
    pub fn with_configs(
        allocator_config: AllocatorConfig,
        gc_config: GCConfig,
        compression_config: CompressionConfig,
        leak_detector_config: LeakDetectorConfig,
    ) -> Self {
        let allocator = Arc::new(ZeroCopyAllocator::new(allocator_config));
        let gc = Arc::new(Mutex::new(GenerationalGC::new(gc_config)));
        let compressor = Arc::new(Mutex::new(MemoryCompression::new(compression_config)));
        let leak_detector = Arc::new(Mutex::new(MemoryLeakDetector::new(leak_detector_config)));

        Self {
            allocator: Arc::clone(&allocator),
            gc: Arc::clone(&gc),
            compressor: Arc::clone(&compressor),
            leak_detector: Arc::clone(&leak_detector),
            integrated_stats: Arc::new(IntegratedStats {
                total_allocated_bytes: 0,
                total_freed_bytes: 0,
                active_memory_bytes: 0,
                zero_copy_ratio: 0.0,
                gc_efficiency: 0.0,
                compression_efficiency: 0.0,
                leak_detection_rate: 0.0,
                total_savings_ratio: 0.0,
            }),
            created_at: Instant::now(),
        }
    }

    /// 分配内存
    pub fn allocate(&self, size: usize) -> *mut u8 {
        // 记录分配
        {
            let leak_detector = self.leak_detector.lock().unwrap();
            leak_detector.track_allocation(
                self.allocator.generate_address_for_tracking(),
                size,
                ObjectType::Normal,
                None,
            );
        }

        // 执行分配
        self.allocator.allocate(size)
    }

    /// 释放内存
    pub fn deallocate(&self, ptr: *mut u8, size: usize) {
        // 记录释放
        {
            let leak_detector = self.leak_detector.lock().unwrap();
            leak_detector.track_deallocation(ptr as usize);
        }

        // 执行释放
        self.allocator.deallocate(ptr, size);
    }

    /// 触发垃圾回收
    pub fn trigger_gc(&self) {
        let gc = self.gc.lock().unwrap();
        gc.trigger_full_gc();
    }

    /// 检测内存泄漏
    pub fn detect_leaks(&self) -> LeakReport {
        let leak_detector = self.leak_detector.lock().unwrap();
        leak_detector.detect_leaks()
    }

    /// 压缩数据
    pub fn compress_data(&self, data: &[u8], address: usize) -> Result<(), memory_compression::CompressionError> {
        // 简化的压缩实现
        // 实际应用中需要处理压缩结果
        let compressor = self.compressor.lock().unwrap();
        let _ = compressor.compress(data, address)?;
        Ok(())
    }

    /// 获取集成统计信息
    pub fn get_integrated_stats(&self) -> IntegratedStatsSnapshot {
        let allocator_stats = self.allocator.get_stats();
        let gc = self.gc.lock().unwrap();
        let gc_stats = gc.get_stats();
        let compressor = self.compressor.lock().unwrap();
        let compression_stats = compressor.get_stats();
        let leak_detector = self.leak_detector.lock().unwrap();
        let leak_stats = leak_detector.get_stats();

        // 计算集成指标
        let zero_copy_ratio = if allocator_stats.total_allocations > 0 {
            allocator_stats.zero_copy_allocations as f64 / allocator_stats.total_allocations as f64 * 100.0
        } else {
            0.0
        };

        let gc_efficiency = if gc_stats.young_gc_count > 0 {
            gc_stats.total_collected_objects as f64 / gc_stats.young_gc_count as f64
        } else {
            0.0
        };

        let compression_efficiency = compression_stats.compression_efficiency;

        let leak_detection_rate = leak_stats.leak_detection_rate;

        let total_savings = allocator_stats.total_allocated_bytes.saturating_sub(allocator_stats.total_freed_bytes);
        let total_savings_ratio = if allocator_stats.total_allocated_bytes > 0 {
            (allocator_stats.total_allocated_bytes - total_savings) as f64
                / allocator_stats.total_allocated_bytes as f64 * 100.0
        } else {
            0.0
        };

        IntegratedStatsSnapshot {
            total_allocated_bytes: allocator_stats.total_allocated_bytes,
            total_freed_bytes: allocator_stats.total_freed_bytes,
            active_memory_bytes: allocator_stats.active_allocations,
            zero_copy_ratio,
            gc_efficiency,
            compression_efficiency,
            leak_detection_rate,
            total_savings_ratio,
            allocator_stats,
            gc_stats,
            compression_stats,
            leak_stats,
            uptime_seconds: self.created_at.elapsed().as_secs(),
        }
    }

    /// 获取详细报告
    pub fn get_detailed_report(&self) -> MemoryManagerReport {
        let integrated_stats = self.get_integrated_stats();
        let leak_report = self.detect_leaks();

        MemoryManagerReport {
            timestamp: Instant::now(),
            integrated_stats,
            leak_report,
            performance_metrics: self.calculate_performance_metrics(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// 计算性能指标
    fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let allocator_stats = self.allocator.get_stats();
        let gc = self.gc.lock().unwrap();
        let gc_stats = gc.get_stats();

        PerformanceMetrics {
            allocation_throughput: if allocator_stats.total_allocations > 0 {
                allocator_stats.total_allocations as f64 / self.created_at.elapsed().as_secs_f64()
            } else {
                0.0
            },
            gc_throughput: if gc_stats.young_gc_count > 0 {
                gc_stats.total_collected_objects as f64 / gc_stats.young_gc_count as f64
            } else {
                0.0
            },
            memory_efficiency: self.calculate_memory_efficiency(),
            compression_ratio: self.calculate_compression_ratio(),
        }
    }

    /// 生成优化建议
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let stats = self.get_integrated_stats();

        if stats.zero_copy_ratio < 80.0 {
            recommendations.push("建议增加内存池大小以提高零拷贝分配率".to_string());
        }

        if stats.gc_efficiency < 0.8 {
            recommendations.push("建议调整 GC 参数以提高垃圾回收效率".to_string());
        }

        if stats.compression_efficiency < 30.0 {
            recommendations.push("建议启用更多数据压缩以减少内存使用".to_string());
        }

        if stats.leak_detection_rate > 5.0 {
            recommendations.push("检测到内存泄漏，建议检查对象生命周期管理".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("内存管理表现良好，无需特殊优化".to_string());
        }

        recommendations
    }

    /// 计算内存效率
    fn calculate_memory_efficiency(&self) -> f64 {
        let allocator_stats = self.allocator.get_stats();
        let total_allocated = allocator_stats.total_allocated_bytes;
        let total_freed = allocator_stats.total_freed_bytes;

        if total_allocated > 0 {
            (total_allocated - total_freed) as f64 / total_allocated as f64 * 100.0
        } else {
            0.0
        }
    }

    /// 计算压缩比
    fn calculate_compression_ratio(&self) -> f64 {
        let compressor = self.compressor.lock().unwrap();
        let compression_stats = compressor.get_stats();
        compression_stats.avg_compression_ratio
    }

    /// 清理资源
    pub fn cleanup(&self) {
        self.allocator.cleanup_idle_pools();
    }

    /// 停止所有组件
    pub fn stop(&mut self) {
        if let Some(gc) = Arc::get_mut(&mut self.gc) {
            gc.lock().unwrap().stop();
        }
        if let Some(compressor) = Arc::get_mut(&mut self.compressor) {
            compressor.lock().unwrap().stop();
        }
        if let Some(leak_detector) = Arc::get_mut(&mut self.leak_detector) {
            leak_detector.lock().unwrap().stop();
        }
    }
}

/// 集成统计快照
#[derive(Debug, Clone)]
pub struct IntegratedStatsSnapshot {
    pub total_allocated_bytes: u64,
    pub total_freed_bytes: u64,
    pub active_memory_bytes: usize,
    pub zero_copy_ratio: f64,
    pub gc_efficiency: f64,
    pub compression_efficiency: f64,
    pub leak_detection_rate: f64,
    pub total_savings_ratio: f64,
    pub allocator_stats: AllocatorStatsSnapshot,
    pub gc_stats: GCStatsSnapshot,
    pub compression_stats: CompressionStatsSnapshot,
    pub leak_stats: LeakDetectionStatsSnapshot,
    pub uptime_seconds: u64,
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 分配吞吐量 (分配/秒)
    pub allocation_throughput: f64,
    /// GC 吞吐量 (对象/次)
    pub gc_throughput: f64,
    /// 内存效率 (%)
    pub memory_efficiency: f64,
    /// 压缩比
    pub compression_ratio: f64,
}

/// 内存管理器报告
#[derive(Debug, Clone)]
pub struct MemoryManagerReport {
    /// 生成时间
    pub timestamp: Instant,
    /// 集成统计
    pub integrated_stats: IntegratedStatsSnapshot,
    /// 泄漏报告
    pub leak_report: LeakReport,
    /// 性能指标
    pub performance_metrics: PerformanceMetrics,
    /// 优化建议
    pub recommendations: Vec<String>,
}

impl Default for UnifiedMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UnifiedMemoryManager {
    fn drop(&mut self) {
        self.stop();
    }
}
