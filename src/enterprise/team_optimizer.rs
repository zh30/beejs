//! 智能任务分配引擎
//! 提供团队协作优化功能，包括任务分配、工作负载平衡、代码所有权分析等

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务结构
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub required_skills: Vec<String>,
    pub estimated_hours: u32,
    pub priority: String,
    pub complexity: String,
}

/// 团队成员结构
#[derive(Debug, Clone)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub skills: Vec<String>,
    pub current_workload: u32, // 当前工作负载百分比 (0-100)
    pub availability: u32,     // 可用性百分比 (0-100)
}

/// 任务分配建议
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    pub recommended_member_id: String,
    pub confidence_score: f64,
    pub reasoning: String,
    pub alternative_candidates: Vec<String>,
}

/// 工作负载平衡报告
#[derive(Debug, Clone)]
pub struct WorkloadBalanceReport {
    pub balance_score: f64, // 0.0-1.0，1.0表示完全平衡
    pub suggestions: Vec<String>,
    pub overloaded_members: Vec<String>,
    pub underutilized_members: Vec<String>,
}

/// 代码文件所有权
#[derive(Debug, Clone)]
pub struct CodeFile {
    pub path: String,
    pub owner_id: String,
    pub expertise_score: f64,
    pub last_modified: String,
}

/// 代码所有者
#[derive(Debug, Clone)]
pub struct CodeOwner {
    pub id: String,
    pub name: String,
    pub expertise_areas: Vec<String>,
    pub ownership_percentage: f64,
}

/// 代码所有权映射
#[derive(Debug, Clone)]
pub struct CodeOwnershipMap {
    pub files: Vec<CodeFile>,
    pub owners: Vec<CodeOwner>,
    pub knowledge_gaps: Vec<String>,
}

/// 知识转移建议
#[derive(Debug, Clone)]
pub struct KnowledgeTransferSuggestion {
    pub from_member_id: String,
    pub to_member_id: String,
    pub skill_area: String,
    pub urgency: String,
    pub suggested_method: String,
}

/// 技能分析器
pub struct SkillAnalyzer {
    // 技能数据库
    skill_database: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
}

impl SkillAnalyzer {
    pub fn new() -> Self {
        Self {
            skill_database: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 计算技能匹配分数
    pub fn calculate_skill_match(&self, task_skills: &[String], member_skills: &[String]) -> f64 {
        if task_skills.is_empty() {
            return 1.0;
        }

        let mut match_count = 0;
        for task_skill in task_skills {
            if member_skills.contains(task_skill) {
                match_count += 1;
            }
        }

        match_count as f64 / task_skills.len() as f64
    }

    /// 分析技能缺口
    pub fn analyze_skill_gaps(&self, task_skills: &[String], member_skills: &[String]) -> Vec<String> {
        let mut gaps = Vec::new();
        for skill in task_skills {
            if !member_skills.contains(skill) {
                gaps.push(skill.clone());
            }
        }
        gaps
    }
}

/// 工作负载平衡器
pub struct WorkloadBalancer {
    // 工作负载历史数据
    workload_history: Arc<RwLock<HashMap<String, Vec<u32>>>>,
}

impl WorkloadBalancer {
    pub fn new() -> Self {
        Self {
            workload_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 计算工作负载平衡分数
    pub fn calculate_balance_score(&self, team: &[TeamMember]) -> f64 {
        if team.is_empty() {
            return 1.0;
        }

        let workloads: Vec<u32> = team.iter().map(|m| m.current_workload).collect();
        let sum: u32 = workloads.iter().sum();
        let avg = sum as f64 / workloads.len() as f64;

        let variance = workloads.iter()
            .map(|&w| {
                let diff = w as f64 - avg;
                diff * diff
            })
            .sum::<f64>() / workloads.len() as f64;

        // 将方差转换为平衡分数 (0-1)
        let max_variance = 2500.0; // 假设最大方差
        let normalized_variance = (variance / max_variance).min(1.0);
        1.0 - normalized_variance
    }

    /// 识别过载成员
    pub fn identify_overloaded_members(&self, team: &[TeamMember]) -> Vec<String> {
        team.iter()
            .filter(|m| m.current_workload > 80)
            .map(|m| m.id.clone())
            .collect()
    }

    /// 识别未充分利用的成员
    pub fn identify_underutilized_members(&self, team: &[TeamMember]) -> Vec<String> {
        team.iter()
            .filter(|m| m.current_workload < 40)
            .map(|m| m.id.clone())
            .collect()
    }
}

/// 知识追踪器
pub struct KnowledgeTracker {
    // 知识图谱
    knowledge_graph: Arc<RwLock<HashMap<String, HashMap<String, u32>>>>,
}

impl KnowledgeTracker {
    pub fn new() -> Self {
        Self {
            knowledge_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录学习活动
    pub async fn record_learning_activity(&self, member_id: &str, skill: &str, hours: u32) {
        let mut graph = self.knowledge_graph.write().await;
        graph
            .entry(member_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(skill.to_string(), hours);
    }

    /// 获取知识图谱
    pub async fn get_knowledge_graph(&self, member_id: &str) -> HashMap<String, u32> {
        let graph = self.knowledge_graph.read().await;
        graph.get(member_id).cloned().unwrap_or_default()
    }

    /// 计算知识重叠
    pub async fn calculate_knowledge_overlap(&self, member1: &str, member2: &str) -> f64 {
        let graph = self.knowledge_graph.read().await;
        let skills1 = graph.get(member1).cloned().unwrap_or_default();
        let skills2 = graph.get(member2).cloned().unwrap_or_default();

        if skills1.is_empty() || skills2.is_empty() {
            return 0.0;
        }

        let common_skills = skills1.keys()
            .filter(|k| skills2.contains_key(*k))
            .count();

        let total_unique_skills = skills1.keys()
            .chain(skills2.keys())
            .collect::<std::collections::HashSet<_>>()
            .len();

        if total_unique_skills == 0 {
            return 0.0;
        }

        common_skills as f64 / total_unique_skills as f64
    }
}

/// 团队协作优化器
pub struct TeamCollaborationOptimizer {
    pub skill_analyzer: Arc<SkillAnalyzer>,
    pub workload_balancer: Arc<WorkloadBalancer>,
    pub knowledge_tracker: Arc<KnowledgeTracker>,
}

impl TeamCollaborationOptimizer {
    pub fn new() -> Self {
        Self {
            skill_analyzer: Arc::new(SkillAnalyzer::new()),
            workload_balancer: Arc::new(WorkloadBalancer::new()),
            knowledge_tracker: Arc::new(KnowledgeTracker::new()),
        }
    }

    /// 智能任务分配建议
    pub async fn suggest_task_assignment(
        &self,
        task: &Task,
        team_members: &[TeamMember],
    ) -> Result<TaskAssignment, String> {
        if team_members.is_empty() {
            return Err("No team members available".to_string());
        }

        let mut best_candidate = None;
        let mut best_score = 0.0;
        let mut scores = Vec::new();

        for member in team_members {
            // 计算技能匹配分数
            let skill_score = self.skill_analyzer.calculate_skill_match(
                &task.required_skills,
                &member.skills,
            );

            // 计算工作负载分数 (可用性越高分数越高)
            let workload_score = (100 - member.current_workload) as f64 / 100.0;

            // 计算综合分数 (技能匹配60% + 工作负载40%)
            let total_score = skill_score * 0.6 + workload_score * 0.4;

            scores.push((member.id.clone(), total_score));

            if total_score > best_score {
                best_score = total_score;
                best_candidate = Some(member.id.clone());
            }
        }

        let recommended_member_id = best_candidate.ok_or("No suitable candidate found")?;

        // 生成推理说明
        let reasoning = format!(
            "基于技能匹配({:.1}%)和工作负载({:.1}%)的综合评估",
            best_score * 100.0,
            best_score * 100.0
        );

        // 获取备选候选人
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let alternative_candidates: Vec<String> = scores
            .iter()
            .skip(1)
            .take(2)
            .map(|(id, _)| id.clone())
            .collect();

        Ok(TaskAssignment {
            recommended_member_id,
            confidence_score: best_score,
            reasoning,
            alternative_candidates,
        })
    }

    /// 平衡团队工作负载
    pub async fn balance_team_workload(
        &self,
        team: &[TeamMember],
    ) -> Result<WorkloadBalanceReport, String> {
        if team.is_empty() {
            return Err("No team members provided".to_string());
        }

        let balance_score = self.workload_balancer.calculate_balance_score(team);
        let overloaded_members = self.workload_balancer.identify_overloaded_members(team);
        let underutilized_members = self.workload_balancer.identify_underutilized_members(team);

        let mut suggestions = Vec::new();

        if !overloaded_members.is_empty() {
            suggestions.push(format!(
                "建议将 {} 的任务重新分配给其他团队成员",
                overloaded_members.join(", ")
            ));
        }

        if !underutilized_members.is_empty() {
            suggestions.push(format!(
                "可以向 {} 分配更多任务以提高效率",
                underutilized_members.join(", ")
            ));
        }

        if balance_score < 0.5 {
            suggestions.push("团队工作负载严重不平衡，建议立即进行调整".to_string());
        } else if balance_score < 0.8 {
            suggestions.push("团队工作负载略有不平衡，可以适当调整".to_string());
        } else {
            suggestions.push("团队工作负载较为平衡".to_string());
        }

        Ok(WorkloadBalanceReport {
            balance_score,
            suggestions,
            overloaded_members,
            underutilized_members,
        })
    }

    /// 分析代码所有权
    pub async fn analyze_code_ownership(
        &self,
        codebase: &str,
    ) -> Result<CodeOwnershipMap, String> {
        // 模拟代码库分析
        let files = vec![
            CodeFile {
                path: format!("{}/src/auth.rs", codebase),
                owner_id: "DEV-001".to_string(),
                expertise_score: 0.95,
                last_modified: "2024-12-01".to_string(),
            },
            CodeFile {
                path: format!("{}/src/api.rs", codebase),
                owner_id: "DEV-002".to_string(),
                expertise_score: 0.88,
                last_modified: "2024-12-05".to_string(),
            },
            CodeFile {
                path: format!("{}/src/db.rs", codebase),
                owner_id: "DEV-001".to_string(),
                expertise_score: 0.92,
                last_modified: "2024-11-28".to_string(),
            },
        ];

        let owners = vec![
            CodeOwner {
                id: "DEV-001".to_string(),
                name: "Alice".to_string(),
                expertise_areas: vec!["Rust".to_string(), "Security".to_string()],
                ownership_percentage: 65.0,
            },
            CodeOwner {
                id: "DEV-002".to_string(),
                name: "Bob".to_string(),
                expertise_areas: vec!["JavaScript".to_string(), "API".to_string()],
                ownership_percentage: 35.0,
            },
        ];

        let knowledge_gaps = vec![
            "DevOps".to_string(),
            "Testing".to_string(),
            "Security".to_string(),
        ];

        Ok(CodeOwnershipMap {
            files,
            owners,
            knowledge_gaps,
        })
    }

    /// 建议知识转移
    pub async fn suggest_knowledge_transfer(
        &self,
        ownership_map: &CodeOwnershipMap,
    ) -> Result<Vec<KnowledgeTransferSuggestion>, String> {
        let mut suggestions = Vec::new();

        // 模拟知识转移建议
        for gap in &ownership_map.knowledge_gaps {
            suggestions.push(KnowledgeTransferSuggestion {
                from_member_id: "DEV-001".to_string(),
                to_member_id: "DEV-002".to_string(),
                skill_area: gap.clone(),
                urgency: "Medium".to_string(),
                suggested_method: "Pair Programming".to_string(),
            });
        }

        Ok(suggestions)
    }
}

// 为默认实现 Default
impl Default for TeamCollaborationOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SkillAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WorkloadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for KnowledgeTracker {
    fn default() -> Self {
        Self::new()
    }
}
