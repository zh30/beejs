//! 风险评估模块
//!
//! 提供风险评分和评估功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

/// 风险评估错误
#[derive(Error, Debug)]
pub enum RiskAssessmentError {
    #[error("Risk assessment failed: {0}")]
    AssessmentFailed(String),

    #[error("Invalid risk factor: {0}")]
    InvalidRiskFactor(String),
}

/// 风险因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f64, // 0-1
    pub value: f64,  // 0-100
}

/// 风险评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall_score: f64, // 0-100
    pub factors: Vec<RiskFactor>,
    pub level: RiskLevel,
    pub timestamp: std::time::SystemTime,
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 {
            RiskLevel::Critical
        } else if score >= 60.0 {
            RiskLevel::High
        } else if score >= 30.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }
}

/// 风险评估器
#[derive(Debug)]
pub struct RiskAssessor {
    factors: HashMap<String, RiskFactor>>>>>>,
}

impl RiskAssessor {
    pub fn new() -> Self {
        let mut factors = HashMap::new();

        // 添加默认风险因子
        factors.insert(
            "data_sensitivity".to_string(),
            RiskFactor {
                name: "数据敏感度".to_string(),
                weight: 0.3,
                value: 50.0,
            },
        );

        factors.insert(
            "access_frequency".to_string(),
            RiskFactor {
                name: "访问频率".to_string(),
                weight: 0.2,
                value: 40.0,
            },
        );

        factors.insert(
            "user_behavior".to_string(),
            RiskFactor {
                name: "用户行为".to_string(),
                weight: 0.25,
                value: 30.0,
            },
        );

        factors.insert(
            "system_vulnerability".to_string(),
            RiskFactor {
                name: "系统漏洞".to_string(),
                weight: 0.25,
                value: 20.0,
            },
        );

        Self { factors }
    }

    pub fn add_factor(&mut self, factor: RiskFactor) {
        self.factors.insert(factor.name.clone(), factor);
    }

    pub fn update_factor(&mut self, name: &str, value: f64) -> Result<(), RiskAssessmentError> {
        if let Some(factor) = self.factors.get_mut(name) {
            factor.value = value;
            Ok(())
        } else {
            Err(RiskAssessmentError::InvalidRiskFactor(name.to_string())
        }
    }

    pub fn assess(&self) -> RiskScore {
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for factor in self.factors.values() {
            total_weighted_score += factor.value * factor.weight;
            total_weight += factor.weight;
        }

        let overall_score: _ = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        };

        RiskScore {
            overall_score,
            factors: self.factors.values().cloned().collect(),
            level: RiskLevel::from_score(overall_score),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

// 默认实现
impl Default for RiskAssessor {
    fn default() -> Self {
        Self::new()
    }
}
