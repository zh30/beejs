//! 智能负载均衡器 - Stage 90 Phase 5.3

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// 负载均衡策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    AIAdaptive,
}

/// 工作者负载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerLoad {
    pub worker_id: String,
    pub current_load: f64,
    pub capacity: f64,
    pub utilization: f64,
}

/// 负载均衡决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancingDecision {
    pub selected_worker: String,
    pub confidence: f64,
    pub reasoning: String,
}

/// 智能负载均衡器
pub struct LoadBalancer {
    workers: HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad, String, WorkerLoad, std::collections::HashMap<String, WorkerLoad, String, WorkerLoad>>>>>>>,
    strategy: BalancingStrategy,
}

impl LoadBalancer {
    pub fn new(strategy: BalancingStrategy) -> Self {
        Self {
            workers: HashMap::new(),
            strategy,
        }
    }

    pub fn add_worker(&mut self, worker: WorkerLoad) {
        self.workers.insert(worker.worker_id.clone(), worker);
    }

    pub fn select_worker(&self) -> Option<BalancingDecision> {
        match self.strategy {
            BalancingStrategy::RoundRobin => self.round_robin(),
            BalancingStrategy::LeastConnections => self.least_connections(),
            BalancingStrategy::AIAdaptive => self.ai_adaptive(),
            BalancingStrategy::WeightedRoundRobin => self.weighted_round_robin(),
        }
    }

    fn round_robin(&self) -> Option<BalancingDecision> {
        self.workers.values().next().cloned().map(|w| BalancingDecision {
            selected_worker: w.worker_id,
            confidence: 0.8,
            reasoning: "轮询策略".to_string(),
        })
    }

    fn least_connections(&self) -> Option<BalancingDecision> {
        let min_load_worker: _ = self.workers.values()
            .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap());

        min_load_worker.map(|w| BalancingDecision {
            selected_worker: w.worker_id.clone(),
            confidence: 0.9,
            reasoning: "最少连接策略".to_string(),
        })
    }

    fn weighted_round_robin(&self) -> Option<BalancingDecision> {
        let total_capacity: f64 = self.workers.values().map(|w| w.capacity).sum();
        if total_capacity == 0.0 {
            return None;
        }

        let best_worker: _ = self.workers.values()
            .max_by(|a, b| {
                (a.capacity / total_capacity)
                    .partial_cmp(&(b.capacity / total_capacity))
                    .unwrap()
            });

        best_worker.map(|w| BalancingDecision {
            selected_worker: w.worker_id.clone(),
            confidence: 0.85,
            reasoning: "加权轮询策略".to_string(),
        })
    }

    fn ai_adaptive(&self) -> Option<BalancingDecision> {
        // AI 驱动的智能负载均衡
        let mut best_worker = None;
        let mut best_score = f64::MIN;

        for worker in self.workers.values() {
            // 综合评分：利用率 + 响应时间 + 资源可用性
            let utilization_score: _ = 1.0 - worker.utilization;
            let load_score: _ = 1.0 / (1.0 + worker.current_load);
            let capacity_score: _ = worker.capacity / 100.0;

            let score: _ = utilization_score * 0.4 + load_score * 0.4 + capacity_score * 0.2;

            if score > best_score {
                best_score = score;
                best_worker = Some(worker);
            }
        }

        best_worker.map(|w| BalancingDecision {
            selected_worker: w.worker_id.clone(),
            confidence: best_score,
            reasoning: "AI 自适应策略".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_load_balancer() {
        let mut lb = LoadBalancer::new(BalancingStrategy::AIAdaptive);

        lb.add_worker(WorkerLoad {
            worker_id: "worker1".to_string(),
            current_load: 10.0,
            capacity: 100.0,
            utilization: 0.1,
        });

        lb.add_worker(WorkerLoad {
            worker_id: "worker2".to_string(),
            current_load: 5.0,
            capacity: 100.0,
            utilization: 0.05,
        });

        let decision: _ = lb.select_worker();
        assert!(decision.is_some());
        assert_eq!(decision.unwrap().selected_worker, "worker2");
    }
}
