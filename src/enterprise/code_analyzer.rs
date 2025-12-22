//! 企业级代码库分析器
//! 提供多仓库架构分析、技术债务评估、依赖关系映射等功能
use std::path::Path;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use std::sync::{Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
/// 仓库信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryInfo {
    pub id: String,
    pub name: String,
    pub path: std::path::PathBuf,
    pub language: String,
    pub framework: Option<String>,
    pub dependencies: Vec<String>,
}
/// 代码库指标
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebaseMetrics {
    pub total_lines_of_code: usize,
    pub complexity_score: f64,
    pub code_duplication_rate: f64,
    pub test_coverage: f64,
    pub maintainability_index: f64,
    pub technical_debt_ratio: f64,
    pub cyclomatic_complexity: f64,
    pub out_of_date_dependencies: usize,
    pub deprecated_api_usage: usize,
}
/// 架构模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitecturePattern {
    pub name: String,
    pub description: String,
    pub confidence: f64,
    pub repositories: Vec<String>,
}
/// 债务项目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebtItem {
    pub category: String,
    pub description: String,
    pub severity: String,
    pub estimated_effort: String,
    pub impact: String,
}
/// 技术债务报告
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechnicalDebtReport {
    pub debt_ratio: f64,
    pub debt_items: Vec<DebtItem>,
    pub estimated_remediation_cost: f64,
    pub priority_recommendations: Vec<String>,
}
/// 依赖关系边
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyEdge {
    pub from_repo: String,
    pub to_repo: String,
    pub dependency_type: String,
    pub strength: f64,
}
/// 依赖关系图
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}
/// 循环依赖
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CircularDependency {
    pub path: Vec<String>,
    pub severity: String,
}
/// 重构建议
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefactoringSuggestion {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub estimated_effort: String,
    pub expected_benefit: String,
    pub affected_repositories: Vec<String>,
}
/// 企业分析报告
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnterpriseAnalysisReport {
    pub repositories: Vec<RepositoryInfo>,
    pub metrics: CodebaseMetrics,
    pub architecture_patterns: Vec<ArchitecturePattern>,
    pub technical_debt: TechnicalDebtReport,
    pub dependency_graph: Option<DependencyGraph>,
    pub recommendations: Vec<RefactoringSuggestion>,
}
/// 企业分析摘要
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnterpriseAnalysisSummary {
    pub total_repositories: usize,
    pub total_lines_of_code: usize,
    pub average_complexity: f64,
    pub dominant_language: String,
    pub architectural_health_score: f64,
}
/// 企业分析结果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnterpriseAnalysisResult {
    pub repositories: Vec<RepositoryInfo>,
    pub metrics: CodebaseMetrics,
    pub architecture_patterns: Vec<ArchitecturePattern>,
    pub technical_debt: TechnicalDebtReport,
    pub recommendations: Vec<RefactoringSuggestion>,
}
/// 企业配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnterpriseConfig {
    pub repositories: Vec<RepositoryInfo>,
    pub analysis_depth: AnalysisDepth,
    pub include_test_metrics: bool,
    pub security_scan_enabled: bool,
}
/// 分析深度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisDepth {
    Shallow,
    Medium,
    Deep,
}
/// 企业依赖图
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnterpriseDependencyGraph {
    pub repository_graphs: Vec<(String, DependencyGraph)>,
    pub cross_repository_dependencies: Vec<DependencyEdge>,
    pub circular_dependencies: Vec<CircularDependency>,
}
/// 企业代码库分析器
pub struct EnterpriseCodeAnalyzer {
    metrics_cache: Arc<RwLock<MetricsCache>>,
    pattern_detector: Arc<PatternDetector>,
    debt_analyzer: Arc<DebtAnalyzer>,
}
impl EnterpriseCodeAnalyzer {
    /// 创建新的分析器实例
    pub fn new() -> Self {
        Self {
            metrics_cache: Arc::new(Mutex::new(MetricsCache::new()))
            pattern_detector: Arc::new(Mutex::new(PatternDetector::new()))
            debt_analyzer: Arc::new(Mutex::new(DebtAnalyzer::new()))
        }
    }
    /// 分析企业代码库
    pub async fn analyze_enterprise_codebase(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<EnterpriseAnalysisReport, Box<dyn std::error::Error + Send + Sync>> {
        // 1. 收集代码指标
        let metrics: _ = self.collect_enterprise_metrics(repositories).await?;
        // 2. 检测架构模式
        let patterns: _ = self.detect_architecture_patterns(repositories).await?;
        // 3. 评估技术债务
        let tech_debt: _ = self.assess_technical_debt(&metrics).await?;
        // 4. 生成重构建议
        let recommendations: _ = self.generate_recommendations(&tech_debt, &patterns, &metrics)?;
        Ok(EnterpriseAnalysisReport {
            repositories: repositories.to_vec(),
            metrics,
            architecture_patterns: patterns,
            technical_debt: tech_debt,
            dependency_graph: None, // TODO: 实现依赖图分析
            recommendations: recommendations.clone(),
        })
    }
    /// 检测架构模式
    pub async fn detect_architecture_patterns(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<Vec<ArchitecturePattern>, Box<dyn std::error::Error + Send + Sync>> {
        self.pattern_detector.detect_patterns(repositories).await
    }
    /// 评估技术债务
    pub async fn assess_technical_debt(
        &self,
        metrics: &CodebaseMetrics,
    ) -> Result<TechnicalDebtReport, Box<dyn std::error::Error + Send + Sync>> {
        self.debt_analyzer.analyze_debt(metrics).await
    }
    /// 生成重构建议
    pub async fn suggest_refactoring(
        &self,
        debt_items: &[DebtItem],
    ) -> Result<Vec<RefactoringSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let mut suggestions = Vec::new();
        for item in debt_items {
            let suggestion: _ = match item.category.as_str() {
                "Code Quality" => RefactoringSuggestion {
                    title: format!("重构: {}", item.description),
                    description: format!("建议重构 {} 以提高代码质量", item.description),
                    priority: item.severity.clone(),
                    estimated_effort: item.estimated_effort.clone(),
                    expected_benefit: "提高可维护性".to_string(),
                    affected_repositories: vec!["全部仓库".to_string()],
                },
                "Dependencies" => RefactoringSuggestion {
                    title: "更新依赖项".to_string(),
                    description: "更新过时的依赖项以提高安全性和性能".to_string(),
                    priority: "Medium".to_string(),
                    estimated_effort: "1-2天".to_string(),
                    expected_benefit: "提高安全性和性能".to_string(),
                    affected_repositories: vec!["相关仓库".to_string()],
                },
                _ => RefactoringSuggestion {
                    title: "一般优化".to_string(),
                    description: item.description.clone(),
                    priority: item.severity.clone(),
                    estimated_effort: item.estimated_effort.clone(),
                    expected_benefit: "提高系统整体质量".to_string(),
                    affected_repositories: vec![],
                },
            };
            suggestions.push(suggestion);
        }
        Ok(suggestions)
    }
    /// 映射跨仓库依赖
    pub async fn map_cross_repository_dependencies(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<EnterpriseDependencyGraph, Box<dyn std::error::Error + Send + Sync>> {
        let mut cross_deps = Vec::new();
        // 简化实现：基于依赖名称匹配
        for repo in repositories {
            for dep in &repo.dependencies {
                if dep.starts_with("@company/") {
                    let target_repo: _ = dep.strip_prefix("@company/").unwrap();
                    // 处理带 "-utils" 后缀的仓库名
                    let normalized_name: _ = if target_repo.ends_with("-utils") {
                        target_repo.strip_suffix("-utils").unwrap()
                    } else {
                        target_repo
                    };
                    cross_deps.push(DependencyEdge {
                        from_repo: repo.id.clone(),
                        to_repo: normalized_name.to_string(),
                        dependency_type: "internal".to_string(),
                        strength: 1.0,
                    });
                }
            }
        }
        Ok(EnterpriseDependencyGraph {
            repository_graphs: Vec::new(),
            cross_repository_dependencies: cross_deps,
            circular_dependencies: Vec::new(),
        })
    }
    /// 识别循环依赖
    pub async fn identify_circular_dependencies(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<CircularDependency>, Box<dyn std::error::Error + Send + Sync>> {
        let mut circular_deps = Vec::new();
        // 简化的循环检测算法
        for edge in &graph.edges {
            let (from, to) = edge;
            // 检查是否存在从 to 回到 from 的路径
            if self.has_path(graph, to, from) {
                circular_deps.push(CircularDependency {
                    path: vec![from.clone(), to.clone(), from.clone()],
                    severity: "High".to_string(),
                });
            }
        }
        Ok(circular_deps)
    }
    /// 生成综合报告
    pub async fn generate_comprehensive_report(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<ComprehensiveReport, Box<dyn std::error::Error + Send + Sync>> {
        let analysis: _ = self.analyze_enterprise_codebase(repositories).await?;
        let recommendations: _ = analysis.recommendations.clone();
        let detailed_analysis: _ = EnterpriseAnalysisReport {
            repositories: analysis.repositories,
            metrics: analysis.metrics,
            architecture_patterns: analysis.architecture_patterns,
            technical_debt: analysis.technical_debt,
            dependency_graph: analysis.dependency_graph,
            recommendations: recommendations.clone(),
        };
        Ok(ComprehensiveReport {
            summary: EnterpriseAnalysisSummary {
                total_repositories: repositories.len(),
                total_lines_of_code: detailed_analysis.metrics.total_lines_of_code,
                average_complexity: detailed_analysis.metrics.complexity_score,
                dominant_language: self.get_dominant_language(repositories),
                architectural_health_score: self.calculate_health_score(&detailed_analysis),
            },
            technical_debt_assessment: Some(detailed_analysis.technical_debt.clone()),
            recommendations,
            detailed_analysis,
        })
    }
    // 私有辅助方法
    fn generate_recommendations(
        &self,
        tech_debt: &TechnicalDebtReport,
        patterns: &[ArchitecturePattern],
        metrics: &CodebaseMetrics,
    ) -> Result<Vec<RefactoringSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let mut suggestions = Vec::new();
        for item in &tech_debt.debt_items {
            let suggestion: _ = match item.category.as_str() {
                "Code Quality" => RefactoringSuggestion {
                    title: format!("重构: {}", item.description),
                    description: format!("建议重构 {} 以提高代码质量", item.description),
                    priority: item.severity.clone(),
                    estimated_effort: item.estimated_effort.clone(),
                    expected_benefit: "提高可维护性".to_string(),
                    affected_repositories: vec!["全部仓库".to_string()],
                },
                "Dependencies" => RefactoringSuggestion {
                    title: "更新依赖项".to_string(),
                    description: "更新过时的依赖项以提高安全性和性能".to_string(),
                    priority: "Medium".to_string(),
                    estimated_effort: "1-2天".to_string(),
                    expected_benefit: "提高安全性和性能".to_string(),
                    affected_repositories: vec!["相关仓库".to_string()],
                },
                _ => RefactoringSuggestion {
                    title: "一般优化".to_string(),
                    description: item.description.clone(),
                    priority: item.severity.clone(),
                    estimated_effort: item.estimated_effort.clone(),
                    expected_benefit: "提高系统整体质量".to_string(),
                    affected_repositories: vec![],
                },
            };
            suggestions.push(suggestion);
        }
        // 基于架构模式添加建议
        for pattern in patterns {
            if pattern.name.contains("Microservices") {
                suggestions.push(RefactoringSuggestion {
                    title: "优化微服务通信".to_string(),
                    description: "建议实施服务网格以优化微服务间通信".to_string(),
                    priority: "Medium".to_string(),
                    estimated_effort: "1-2周".to_string(),
                    expected_benefit: "提高性能和可靠性".to_string(),
                    affected_repositories: pattern.repositories.clone(),
                });
            }
        }
        Ok(suggestions)
    }
    async fn collect_enterprise_metrics(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<CodebaseMetrics, Box<dyn std::error::Error + Send + Sync>> {
        // 模拟指标收集
        let total_loc: _ = repositories.len() * 10000; // 每个仓库平均 10k 行
        let complexity: _ = 5.0 + (repositories.len() as f64 * 0.1); // 基于仓库数量的复杂度
        Ok(CodebaseMetrics {
            total_lines_of_code: total_loc,
            complexity_score: complexity,
            code_duplication_rate: 0.15,
            test_coverage: 0.75,
            maintainability_index: 7.2,
            technical_debt_ratio: 0.20,
            cyclomatic_complexity: complexity,
            out_of_date_dependencies: repositories.len() * 2,
            deprecated_api_usage: repositories.len(),
        })
    }
    fn has_path(&self, graph: &DependencyGraph, from: &str, to: &str) -> bool {
        // 简化的路径查找
        graph.edges.iter().any(|(a, b)| a == from && b == to)
    }
    fn get_dominant_language(&self, repositories: &[RepositoryInfo]) -> String {
        let mut lang_counts = std::collections::HashMap::new();
        for repo in repositories {
            *lang_counts.entry(&repo.language).or_insert(0) += 1;
        }
        lang_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }
    fn calculate_health_score(&self, analysis: &EnterpriseAnalysisReport) -> f64 {
        // 基于技术债务比率和代码质量指标计算健康度
        let debt_penalty: _ = analysis.technical_debt.debt_ratio * 30.0;
        let complexity_penalty: _ = if analysis.metrics.complexity_score > 10.0 {
            20.0
        } else {
            0.0
        };
        let coverage_bonus: _ = analysis.metrics.test_coverage * 10.0;
        100.0 - debt_penalty - complexity_penalty + coverage_bonus
    }
}
// 内部辅助结构体
struct MetricsCache {
    cache: std::collections::HashMap<String, CodebaseMetrics>,
}
impl MetricsCache {
    fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }
}
struct PatternDetector;
impl PatternDetector {
    fn new() -> Self {
        Self
    }
    async fn detect_patterns(
        &self,
        repositories: &[RepositoryInfo],
    ) -> Result<Vec<ArchitecturePattern>, Box<dyn std::error::Error + Send + Sync>> {
        let mut patterns = Vec::new();
        // 检测微服务架构
        let express_repos: Vec<_> = repositories
            .iter()
            .filter(|r| r.framework.as_ref().map_or(false, |f| f.contains("Express"))
            .collect();
        if express_repos.len() >= 2 {
            patterns.push(ArchitecturePattern {
                name: "Microservices Architecture".to_string(),
                description: "基于多个 Express 服务的微服务架构".to_string(),
                confidence: 0.85,
                repositories: express_repos.iter().map(|r| r.id.clone()).collect(),
            });
        }
        // 检测前端架构
        let react_repos: Vec<_> = repositories
            .iter()
            .filter(|r| r.framework.as_ref().map_or(false, |f| f.contains("React"))
            .collect();
        if react_repos.len() >= 1 {
            patterns.push(ArchitecturePattern {
                name: "React Frontend".to_string(),
                description: "基于 React 的前端应用".to_string(),
                confidence: 0.95,
                repositories: react_repos.iter().map(|r| r.id.clone()).collect(),
            });
        }
        Ok(patterns)
    }
}
struct DebtAnalyzer;
impl DebtAnalyzer {
    fn new() -> Self {
        Self
    }
    async fn analyze_debt(
        &self,
        metrics: &CodebaseMetrics,
    ) -> Result<TechnicalDebtReport, Box<dyn std::error::Error + Send + Sync>> {
        let mut debt_items = Vec::new();
        // 基于指标生成债务项目
        if metrics.code_duplication_rate > 0.20 {
            debt_items.push(DebtItem {
                category: "Code Quality".to_string(),
                description: "代码重复率过高".to_string(),
                severity: "High".to_string(),
                estimated_effort: "1-2周".to_string(),
                impact: "Maintainability".to_string(),
            });
        }
        if metrics.test_coverage < 0.80 {
            debt_items.push(DebtItem {
                category: "Testing".to_string(),
                description: "测试覆盖率不足".to_string(),
                severity: "Medium".to_string(),
                estimated_effort: "1周".to_string(),
                impact: "Reliability".to_string(),
            });
        }
        if metrics.out_of_date_dependencies > 10 {
            debt_items.push(DebtItem {
                category: "Dependencies".to_string(),
                description: format!("{} 个依赖项过时", metrics.out_of_date_dependencies),
                severity: "Medium".to_string(),
                estimated_effort: "2-3天".to_string(),
                impact: "Security".to_string(),
            });
        }
        let estimated_cost: _ = debt_items.len() as f64 * 5000.0; // 每个债务项 5k 估算
        Ok(TechnicalDebtReport {
            debt_ratio: metrics.technical_debt_ratio,
            debt_items,
            estimated_remediation_cost: estimated_cost,
            priority_recommendations: vec![
                "优先处理高严重性债务".to_string(),
                "建立代码审查流程".to_string(),
                "增加自动化测试".to_string(),
            ],
        })
    }
}
impl Default for EnterpriseCodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
/// 综合报告
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComprehensiveReport {
    pub summary: EnterpriseAnalysisSummary,
    pub technical_debt_assessment: Option<TechnicalDebtReport>,
    pub recommendations: Vec<RefactoringSuggestion>,
    pub detailed_analysis: EnterpriseAnalysisReport,
}