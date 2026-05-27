#!/usr/bin/env python3
"""批量删除未使用的 Context 导入"""
import re

files = [
    'src/edge/cache_strategy.rs',
    'src/observability/prometheus_exporter.rs',
    'src/edge/deployment_optimizer.rs',
    'src/edge/edge_runtime.rs',
    'src/edge/global_router.rs',
    'src/performance_comparison/benchmark_runner.rs',
    'src/stage_38_smart_process_pool.rs',
    'src/wasm_optimized/cache_manager.rs',
    'src/wasm_optimized/zero_copy_loader.rs',
    'src/ai_inference/model_loader.rs',
    'src/ai_inference/onnx_runtime.rs',
    'src/ai_inference/pytorch_engine.rs',
    'src/observability/alerting.rs',
    'src/ai_inference/tensor_ops.rs',
    'src/edge/cloudflare_integration.rs',
    'src/network/zero_copy/async_impl.rs',
    'src/realtime/incremental_sync.rs',
    'src/cli/enhanced_cli.rs',
    'src/observability/jaeger_tracer.rs',
    'src/wasm_optimized/simd_optimizer.rs',
    'src/wasm_optimized/multithread.rs',
    'src/main.rs',
    'src/debugger/session.rs',
    'src/wasm_integration.rs',
    'src/wasm_optimized/executor.rs',
    'src/wasm/high_performance_cache.rs',
    'src/wasm/js_interop.rs',
    'src/wasm/memory_manager.rs',
    'src/wasm/module_cache.rs',
    'src/wasm/module_loader.rs',
    'src/wasm/compiler.rs',
    'src/stage_48_optimized_process_pool.rs',
    'src/precompiled_cache.rs',
    'src/process_pool.rs',
    'src/memory_mapped_file.rs',
    'src/lib_minimal.rs',
    'src/lib_v8_partial.rs',
    'src/lib_v8_simple.rs',
    'src/edge/cdn_provider.rs',
    'src/ai_inference/ai_inference_engine.rs',
]

def remove_context_import(file_path):
    """删除未使用的 Context 导入"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # 删除 use std::task::Context; 和相关导入
        patterns = [
            r'use std::task::Context;\n',
            r'use futures::task::Context;\n',
            r'use std::task::\{[^}]*Context[^}]*\};\n',
        ]

        for pattern in patterns:
            if re.search(pattern, content):
                print(f"  删除 Context 导入: {file_path}")
                content = re.sub(pattern, '', content)

        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    except:
        return False

if __name__ == '__main__':
    print("🗑️  删除未使用的 Context 导入...\n")
    count = 0
    for file_path in files:
        if remove_context_import(file_path):
            count += 1
    print(f"\n✅ 完成! 修改了 {count} 个文件")
