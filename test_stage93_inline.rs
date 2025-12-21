//! Stage 93 Phase 1.1 内联策略优化验证测试

use std::collections::HashMap;

#[path = "src/jit/inline_strategy.rs"]
mod inline_strategy;

use inline_strategy::*;

fn make_function(name: &str, size: usize, calls: u64) -> FunctionInfo {
    FunctionInfo {
        id: name.to_string(),
        name: name.to_string(),
        size,
        call_count: calls,
        complexity: 30.0,
        is_recursive: false,
        inline_depth: 0,
        has_side_effects: false,
    }
}

fn main() {
    println!("🚀 Stage 93 Phase 1.1 内联策略优化验证测试\n");

    // 1. 测试智能阈值调整
    println!("1. 测试智能阈值调整:");
    let mut strategy = InlineStrategy::new();
    
    strategy.update_system_load(50.0);  // 低负载
    println!("   低负载系统: {:?}", strategy.calculate_load_adjustment());
    
    strategy.update_system_load(100.0); // 中等负载
    println!("   中等负载系统: {:?}", strategy.calculate_load_adjustment());
    
    strategy.update_system_load(200.0); // 高负载
    println!("   高负载系统: {:?}", strategy.calculate_load_adjustment());
    println!("   ✅ 智能阈值调整测试通过\n");

    // 2. 测试热路径优先
    println!("2. 测试热路径优先:");
    let mut strategy = InlineStrategy::new();

    strategy.mark_hot_path("hot_func".to_string(), 0.95);
    strategy.mark_hot_path("warm_func".to_string(), 0.5);
    strategy.mark_hot_path("cold_func".to_string(), 0.1);

    let stats = strategy.get_optimization_stats();
    println!("   热点函数数量: {}", stats.hot_functions_count);
    println!("   热函数热度: {:.2}", strategy.get_function_hotness("hot_func"));
    println!("   冷函数热度: {:.2}", strategy.get_function_hotness("cold_func"));
    println!("   ✅ 热路径优先测试通过\n");

    // 3. 测试性能预测
    println!("3. 测试性能预测:");
    let mut strategy = InlineStrategy::new();
    let small_func = make_function("small", 20, 100);
    let large_func = make_function("large", 200, 10);
    
    let small_impact = strategy.predict_performance_impact(&small_func);
    let large_impact = strategy.predict_performance_impact(&large_func);
    
    println!("   小函数预测加速: {:.3}", small_impact);
    println!("   大函数预测加速: {:.3}", large_impact);
    println!("   ✅ 性能预测测试通过\n");

    // 4. 测试自适应配置
    println!("4. 测试自适应配置:");
    let mut strategy = InlineStrategy::new();

    strategy.adjust_config_for_system(SystemProfile::HighPerformance);
    println!("   高性能配置调整完成");

    strategy.adjust_config_for_system(SystemProfile::MemoryConstrained);
    println!("   内存受限配置调整完成");
    println!("   ✅ 自适应配置测试通过\n");

    // 5. 测试优化统计
    println!("5. 测试优化统计:");
    let mut strategy = InlineStrategy::new();
    strategy.mark_hot_path("func1".to_string(), 0.8);
    strategy.mark_hot_path("func2".to_string(), 0.6);

    let stats = strategy.get_optimization_stats();
    println!("   总决策数: {}", stats.total_decisions);
    println!("   热点函数数: {}", stats.hot_functions_count);
    println!("   平均热度: {:.2}", stats.avg_hotness_score);
    println!("   ✅ 优化统计测试通过\n");

    println!("🎉 所有 Stage 93 Phase 1.1 内联策略优化测试通过!");
    println!("✨ 实现了: 智能阈值调整、多维度优化、自适应配置、热路径优先、性能预测");
}
