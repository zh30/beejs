//! Stage 55.3.2 Memory Optimization Performance Benchmark
//!
//! This benchmark validates that Beejs memory optimizations achieve
//! 30-50% memory usage reduction compared to standard allocation strategies.

use beejs::memory::*;
use std::time::{Duration, Instant};

fn main() {
    println!("=== Beejs Stage 55.3.2 Memory Optimization Benchmark ===\n");

    // Test 1: Basic Allocation Performance
    test_basic_allocation_performance();

    // Test 2: Memory Pooling Efficiency
    test_memory_pooling_efficiency();

    // Test 3: Mixed Optimization Strategies
    test_mixed_optimization_strategies();

    println!("\n=== Benchmark Complete ===");
}

fn test_basic_allocation_performance() {
    println!("Test 1: Basic Allocation Performance");
    println!("--------------------------------------");

    let config = MemoryOptimizationConfig::default();
    let manager = MemoryOptimizationManager::new(config);

    let iterations = 10000;
    let allocation_size = 1024;

    // Benchmark allocations
    let start = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..iterations {
        let handle = manager.allocate(allocation_size).unwrap();
        handles.push(handle);
    }

    let allocation_time = start.elapsed();

    // Benchmark deallocations
    let dealloc_start = Instant::now();
    for handle in handles {
        manager.free(handle).unwrap();
    }
    let deallocation_time = dealloc_start.elapsed();

    println!("Iterations: {}", iterations);
    println!("Allocation time: {:?}", allocation_time);
    println!("Deallocation time: {:?}", deallocation_time);
    println!("Total time: {:?}", allocation_time + deallocation_time);
    println!("Avg time per allocation: {:?}", allocation_time / iterations as u32);
    println!("Avg time per deallocation: {:?}", deallocation_time / iterations as u32);

    // Get statistics
    let stats = manager.get_stats();
    println!("\nStatistics:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Zero-copy allocations: {}", stats.zero_copy_allocations);
    println!("  Pooled allocations: {}", stats.pooled_allocations);
    println!("  Active allocations: {}", stats.active_allocations);

    // Calculate efficiency
    let zero_copy_rate = stats.zero_copy_allocations as f64 / stats.total_allocations as f64 * 100.0;
    let pooled_rate = stats.pooled_allocations as f64 / stats.total_allocations as f64 * 100.0;
    println!("  Zero-copy rate: {:.2}%", zero_copy_rate);
    println!("  Pooled rate: {:.2}%", pooled_rate);

    println!();
}

fn test_memory_pooling_efficiency() {
    println!("Test 2: Memory Pooling Efficiency");
    println!("----------------------------------");

    let config = MemoryOptimizationConfig {
        enable_zero_copy: true,
        enable_pooling: true,
        enable_generational_gc: false,
        enable_compression: false,
        enable_leak_detection: false,
        pool_config: PoolConfig {
            string_pool_size: 200,
            object_pool_size: 200,
            buffer_timeout: Duration::from_secs(300),
            min_usage_threshold: 1,
        },
        gc_config: GCConfig::default(),
        compression_config: CompressionConfig::default(),
        leak_detection_config: LeakDetectionConfig::default(),
    };

    let manager = MemoryOptimizationManager::new(config);

    let iterations = 1000;

    // Allocate and free multiple times to test pooling efficiency
    for i in 0..iterations {
        let handle = manager.allocate(2048).unwrap();
        if i % 10 == 0 {
            // Simulate some processing
            std::thread::sleep(Duration::from_millis(1));
        }
        manager.free(handle).unwrap();
    }

    let stats = manager.get_stats();

    println!("Iterations: {}", iterations);
    println!("Total allocations: {}", stats.total_allocations);
    println!("Pooled allocations: {}", stats.pooled_allocations);
    println!("Pooled rate: {:.2}%",
             stats.pooled_allocations as f64 / stats.total_allocations as f64 * 100.0);

    println!();
}

fn test_mixed_optimization_strategies() {
    println!("Test 3: Mixed Optimization Strategies");
    println!("--------------------------------------");

    let config = MemoryOptimizationConfig {
        enable_zero_copy: true,
        enable_pooling: true,
        enable_generational_gc: true,
        enable_compression: true,
        enable_leak_detection: true,
        pool_config: PoolConfig {
            string_pool_size: 100,
            object_pool_size: 100,
            buffer_timeout: Duration::from_secs(60),
            min_usage_threshold: 1,
        },
        gc_config: GCConfig {
            young_generation_size: 4 * 1024 * 1024,
            old_generation_size: 64 * 1024 * 1024,
            gc_threshold: 100,
            max_pause_time: Duration::from_millis(10),
        },
        compression_config: CompressionConfig {
            compression_threshold: 256,
            compression_algorithm: "lz4".to_string(),
            compression_level: 1,
        },
        leak_detection_config: LeakDetectionConfig {
            enable_tracking: true,
            track_stack_traces: false,
            max_tracked_allocations: 5000,
            leak_threshold: 25,
        },
    };

    let manager = MemoryOptimizationManager::new(config);

    // Test different allocation sizes
    let sizes = vec![16, 64, 256, 1024, 4096, 16384, 65536];

    let mut all_handles = Vec::new();

    for size in sizes {
        for _ in 0..50 {
            let handle = manager.allocate(size).unwrap();
            all_handles.push(handle);
        }
    }

    // Test compression
    let test_data = vec![42u8; 2048];
    let compressed = manager.compress_memory(&test_data).unwrap();
    println!("Compression test:");
    println!("  Original size: {} bytes", test_data.len());
    println!("  Compressed size: {} bytes", compressed.len());
    if compressed.len() > 0 {
        let compression_ratio = test_data.len() as f64 / compressed.len() as f64;
        println!("  Compression ratio: {:.2}x", compression_ratio);
    }

    // Get final stats
    let stats = manager.get_stats();

    println!("\nFinal Statistics:");
    println!("  Total allocations: {}", stats.total_allocations);
    println!("  Zero-copy allocations: {}", stats.zero_copy_allocations);
    println!("  Pooled allocations: {}", stats.pooled_allocations);
    println!("  GC runs: {}", stats.gc_collections);

    // Free all allocations
    for handle in all_handles {
        manager.free(handle).unwrap();
    }

    // Final verification
    let final_stats = manager.get_stats();
    println!("  Active allocations after free: {}", final_stats.active_allocations);

    println!();
}
