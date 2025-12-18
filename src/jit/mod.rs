//! Stage 30.1: JIT 编译器深度优化模块
//!
//! 提供激进内联、死代码消除、循环展开、逃逸分析等优化技术

pub mod advanced_optimizer;
pub mod dead_code_elimination;
pub mod loop_unrolling;
pub mod escape_analysis;

// 导出主要类型和函数
pub use advanced_optimizer::{
    AdvancedInliningOptimizer, InliningCandidate, InliningDecision,
};

pub use dead_code_elimination::{
    DeadCodeEliminationOptimizer, DeadCodeEliminationDecision,
};

pub use loop_unrolling::{
    LoopUnrollingOptimizer, LoopUnrollingDecision, LoopInfo, LoopType, LoopBounds,
};

pub use escape_analysis::{
    EscapeAnalysisOptimizer, EscapeAnalysisDecision, ObjectInfo, ObjectType, EscapeLevel,
};

/// JIT 优化器组合
pub struct JITOptimizationPipeline {
    pub inlining_optimizer: AdvancedInliningOptimizer,
    pub dce_optimizer: DeadCodeEliminationOptimizer,
    pub unrolling_optimizer: LoopUnrollingOptimizer,
    pub escape_optimizer: EscapeAnalysisOptimizer,
}

impl JITOptimizationPipeline {
    /// 创建新的优化流水线
    pub fn new() -> Self {
        Self {
            inlining_optimizer: AdvancedInliningOptimizer::new(),
            dce_optimizer: DeadCodeEliminationOptimizer::new(),
            unrolling_optimizer: LoopUnrollingOptimizer::new(),
            escape_optimizer: EscapeAnalysisOptimizer::new(),
        }
    }

    /// 执行完整的优化分析
    pub fn analyze_code(&self, code: &str) -> OptimizationReport {
        let inlining_candidates = self.inlining_optimizer.analyze_inlining_candidates(code);
        let dce_decision = self.dce_optimizer.analyze_dead_code(code);
        let loops = self.unrolling_optimizer.analyze_loops(code);
        let objects = self.escape_optimizer.analyze_escape(code);

        OptimizationReport {
            inlining_candidates,
            dce_decision,
            loops,
            objects,
            total_benefit: self.calculate_total_benefit(&dce_decision, &loops, &objects),
        }
    }

    /// 计算总体收益
    fn calculate_total_benefit(
        &self,
        dce: &DeadCodeEliminationDecision,
        loops: &[LoopInfo],
        objects: &[ObjectInfo],
    ) -> f64 {
        let mut total = 0.0;

        // 死代码消除收益
        total += dce.savings_score;

        // 循环展开收益
        for loop_info in loops {
            let decision = self.unrolling_optimizer.make_unrolling_decision(loop_info);
            total += decision.benefit_score;
        }

        // 逃逸分析收益
        for obj_info in objects {
            let decision = self.escape_optimizer.make_escape_decision(obj_info);
            total += decision.allocation_savings;
        }

        total
    }
}

impl Default for JITOptimizationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// 优化报告
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub inlining_candidates: Vec<InliningCandidate>,
    pub dce_decision: DeadCodeEliminationDecision,
    pub loops: Vec<LoopInfo>,
    pub objects: Vec<ObjectInfo>,
    pub total_benefit: f64,
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_optimization_pipeline() {
        let pipeline = JITOptimizationPipeline::new();
        let code = r#"
            let unused = "dead";
            function add(a, b) { return a + b; }
            for (let i = 0; i < 10; i++) {
                console.log(i);
            }
            let obj = { value: 42 };
        "#;

        let report = pipeline.analyze_code(code);

        assert!(report.total_benefit > 0.0);
        assert!(!report.dce_decision.eliminated_items.is_empty());
        assert!(!report.loops.is_empty());
        assert!(!report.objects.is_empty());
    }
}
