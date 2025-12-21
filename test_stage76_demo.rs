//! Stage 76 性能分析器演示
//! 展示高级性能分析功能

use beejs::monitor::profiler::{
    AdvancedProfiler, AdvancedProfilerConfig, SamplingConfig,
};

fn main() {
    println!("=== Beejs Stage 76 性能分析器演示 ===\n");

    // 创建配置
    let config = AdvancedProfilerConfig {
        event_buffer_capacity: 10000,
        sampling_config: SamplingConfig {
            base_sample_rate: 0.5,
            enable_dynamic_sampling: true,
            min_sample_interval: std::time::Duration::from_millis(1),
            importance_threshold: 0.3,
            ..Default::default()
        },
        max_call_depth: 50,
        enable_hotspot_analysis: true,
        enable_stack_analysis: true,
        report_config: beejs::monitor::profiler::ReportConfig {
            generate_json: true,
            generate_text: true,
            generate_html: false,
            output_dir: None,
        },
    };

    // 创建性能分析器
    let mut profiler = AdvancedProfiler::new(config);

    // 启动分析
    profiler.start();
    println!("✅ 性能分析器已启动\n");

    // 模拟函数调用
    println!("📊 正在跟踪函数调用...");

    // 函数 A
    {
        let handle = profiler.track_function("function_a", None, None, None);
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 函数 B (嵌套调用)
        {
            let handle_b = profiler.track_function("function_b", None, None, None);
            std::thread::sleep(std::time::Duration::from_millis(5));

            // 函数 C
            {
                let handle_c = profiler.track_function("function_c", None, None, None);
                std::thread::sleep(std::time::Duration::from_millis(3));
                profiler.record_return(handle_c, 1024);
            }

            profiler.record_return(handle_b, 2048);
        }

        profiler.record_return(handle, 3072);
    }

    // 更多函数调用以创建热点
    for i in 0..100 {
        let handle = profiler.track_function(
            &format!("hot_function_{}", i % 5),
            None,
            None,
            None,
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
        profiler.record_return(handle, 512);
    }

    println!("✅ 函数跟踪完成\n");

    // 生成报告
    println!("📈 生成性能报告...\n");
    match profiler.generate_report() {
        Ok(report) => {
            println!("{}", report);
        }
        Err(e) => {
            eprintln!("❌ 生成报告失败: {}", e);
        }
    }

    // 获取实时快照
    let snapshot = profiler.get_realtime_snapshot();
    println!("\n=== 实时性能快照 ===");
    println!("运行状态: {}", if snapshot.is_running { "运行中" } else { "已停止" });
    println!("运行时间: {:.2} 秒", snapshot.get_uptime_seconds());
    println!("活跃跟踪: {}", snapshot.active_traces);
    println!("采样事件: {}", snapshot.sampled_events);
    println!("总跟踪数: {}", snapshot.total_traces);
    println!("每秒跟踪数: {:.2}", snapshot.get_traces_per_second());

    // 停止分析
    profiler.stop();
    println!("\n✅ 性能分析器已停止");
    println!("\n=== Stage 76 性能分析器演示完成 ===");
}
