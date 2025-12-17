//! AI批量处理器
//! 专为AI推理工作负载设计的高性能批量处理系统

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// AI任务类型
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AiTaskType {
    /// 文本生成任务
    TextGeneration {
        prompt: String,
        max_tokens: Option<usize>,
        temperature: f32,
    },
    /// 图像分类任务
    ImageClassification {
        image_data: Vec<u8>,
        top_k: Option<usize>,
    },
    /// 嵌入向量生成
    Embedding {
        text: String,
        model_name: String,
    },
    /// 翻译任务
    Translation {
        text: String,
        source_lang: String,
        target_lang: String,
    },
}

/// AI任务结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AiTaskResult {
    TextGeneration {
        generated_text: String,
        tokens_used: usize,
        processing_time: Duration,
    },
    ImageClassification {
        predictions: Vec<(String, f32)>,
        processing_time: Duration,
    },
    Embedding {
        vector: Vec<f32>,
        dimensions: usize,
        processing_time: Duration,
    },
    Translation {
        translated_text: String,
        source_lang: String,
        target_lang: String,
        processing_time: Duration,
    },
}

/// 批处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 最大批处理大小
    pub max_batch_size: usize,
    /// 批处理超时时间（毫秒）
    #[allow(dead_code)]
    pub batch_timeout_ms: u64,
    /// 最大并发批次数
    pub max_concurrent_batches: usize,
    /// 动态批处理调整是否启用
    #[allow(dead_code)]
    pub enable_dynamic_batching: bool,
    /// 预热批处理大小
    #[allow(dead_code)]
    pub warmup_batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            batch_timeout_ms: 100,
            max_concurrent_batches: 10,
            enable_dynamic_batching: true,
            warmup_batch_size: 10,
        }
    }
}

/// AI批量处理器
pub struct AiBatchProcessor {
    config: BatchConfig,
    pending_tasks: Arc<Mutex<VecDeque<(usize, AiTaskType)>>>,
    active_batches: Arc<Mutex<usize>>,
    batch_semaphore: Arc<Semaphore>,
    next_task_id: Arc<Mutex<usize>>,
    stats: Arc<Mutex<BatchStats>>,
}

/// 批处理统计信息
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_tasks_processed: usize,
    pub total_batches_processed: usize,
    pub total_processing_time: Duration,
    pub average_batch_size: f64,
    pub peak_memory_usage: usize,
    #[allow(dead_code)]
    pub cache_hits: usize,
    #[allow(dead_code)]
    pub cache_misses: usize,
}

impl BatchStats {
    pub fn record_batch(&mut self, batch_size: usize, processing_time: Duration, memory_used: usize) {
        self.total_tasks_processed += batch_size;
        self.total_batches_processed += 1;
        self.total_processing_time += processing_time;
        self.average_batch_size = self.total_tasks_processed as f64 / self.total_batches_processed as f64;
        self.peak_memory_usage = self.peak_memory_usage.max(memory_used);
    }

    #[allow(dead_code)]
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

#[allow(dead_code)]
impl AiBatchProcessor {
    /// 创建新的AI批量处理器
    pub fn new(config: BatchConfig) -> Self {
        let max_concurrent_batches = config.max_concurrent_batches;
        Self {
            config,
            pending_tasks: Arc::new(Mutex::new(VecDeque::new())),
            active_batches: Arc::new(Mutex::new(0)),
            batch_semaphore: Arc::new(Semaphore::new(max_concurrent_batches)),
            next_task_id: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(BatchStats::default())),
        }
    }

    /// 添加任务到批处理队列
    pub async fn add_task(&self, task: AiTaskType) -> usize {
        let task_id = {
            let mut next_id = self.next_task_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        {
            let mut tasks = self.pending_tasks.lock().unwrap();
            tasks.push_back((task_id, task));
        }

        // 尝试立即处理批次
        self.try_process_batch().await;

        task_id
    }

    /// 批量添加任务
    pub async fn add_tasks(&self, tasks: Vec<AiTaskType>) -> Vec<usize> {
        let mut task_ids = Vec::with_capacity(tasks.len());

        for task in tasks {
            let task_id = self.add_task(task).await;
            task_ids.push(task_id);
        }

        task_ids
    }

    /// 尝试处理批次
    async fn try_process_batch(&self) {
        // 检查是否有可用的并发批次
        let permit = self.batch_semaphore.clone().try_acquire_owned();
        if permit.is_err() {
            return; // 达到最大并发限制
        }

        let permit = permit.unwrap();
        let pending_tasks = self.pending_tasks.clone();
        let active_batches = self.active_batches.clone();
        let stats = self.stats.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            *active_batches.lock().unwrap() += 1;

            let _permit = permit;
            Self::run_batch(pending_tasks, stats, config).await;

            *active_batches.lock().unwrap() -= 1;
        });
    }

    /// 运行单个批次
    async fn run_batch(
        pending_tasks: Arc<Mutex<VecDeque<(usize, AiTaskType)>>>,
        stats: Arc<Mutex<BatchStats>>,
        config: BatchConfig,
    ) {
        let start_time = Instant::now();

        // 收集批次任务
        let batch_size = config.max_batch_size;
        let mut batch_tasks = Vec::with_capacity(batch_size);

        {
            let mut tasks = pending_tasks.lock().unwrap();
            for _ in 0..batch_size {
                if let Some(task) = tasks.pop_front() {
                    batch_tasks.push(task);
                } else {
                    break;
                }
            }
        }

        if batch_tasks.is_empty() {
            return;
        }

        // 处理批次
        let _results = Self::process_batch(&batch_tasks).await;

        // 更新统计信息
        {
            let mut stats_guard = stats.lock().unwrap();
            stats_guard.record_batch(
                batch_tasks.len(),
                start_time.elapsed(),
                batch_tasks.len() * 1024, // 估算内存使用
            );
        }

        // 记录结果（实际应用中会发送到结果通道）
        println!(
            "处理批次: {} 个任务, 耗时: {:.2}ms",
            batch_tasks.len(),
            start_time.elapsed().as_secs_f64() * 1000.0
        );
    }

    /// 处理单个批次
    async fn process_batch(tasks: &[(usize, AiTaskType)]) -> Vec<(usize, AiTaskResult)> {
        let mut results = Vec::with_capacity(tasks.len());

        for (task_id, task) in tasks {
            let result = Self::process_single_task(task).await;
            results.push((*task_id, result));
        }

        results
    }

    /// 处理单个AI任务
    async fn process_single_task(task: &AiTaskType) -> AiTaskResult {
        let start_time = Instant::now();

        let result = match task {
            AiTaskType::TextGeneration { prompt, max_tokens: _, temperature: _ } => {
                // 模拟文本生成
                tokio::time::sleep(Duration::from_millis(50)).await;
                AiTaskResult::TextGeneration {
                    generated_text: format!("Generated text for: {}", prompt),
                    tokens_used: prompt.len() / 4,
                    processing_time: start_time.elapsed(),
                }
            }
            AiTaskType::ImageClassification { image_data: _, top_k } => {
                // 模拟图像分类
                tokio::time::sleep(Duration::from_millis(100)).await;
                let predictions = vec![
                    ("cat".to_string(), 0.85),
                    ("dog".to_string(), 0.75),
                    ("bird".to_string(), 0.65),
                ];
                let predictions = if let Some(k) = top_k {
                    predictions.into_iter().take(*k).collect()
                } else {
                    predictions
                };
                AiTaskResult::ImageClassification {
                    predictions,
                    processing_time: start_time.elapsed(),
                }
            }
            AiTaskType::Embedding { text: _, model_name: _ } => {
                // 模拟嵌入向量生成
                tokio::time::sleep(Duration::from_millis(30)).await;
                let dimensions = 384;
                let vector = vec![0.1; dimensions];
                AiTaskResult::Embedding {
                    vector,
                    dimensions,
                    processing_time: start_time.elapsed(),
                }
            }
            AiTaskType::Translation { text, source_lang, target_lang } => {
                // 模拟翻译
                tokio::time::sleep(Duration::from_millis(80)).await;
                let translated = format!("Translated: {} from {} to {}", text, source_lang, target_lang);
                AiTaskResult::Translation {
                    translated_text: translated,
                    source_lang: source_lang.clone(),
                    target_lang: target_lang.clone(),
                    processing_time: start_time.elapsed(),
                }
            }
        };

        result
    }

    /// 获取批处理统计信息
    pub fn get_stats(&self) -> BatchStats {
        self.stats.lock().unwrap().clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        *self.stats.lock().unwrap() = BatchStats::default();
    }

    /// 获取待处理任务数量
    pub fn pending_tasks_count(&self) -> usize {
        self.pending_tasks.lock().unwrap().len()
    }

    /// 获取活跃批次数
    pub fn active_batches_count(&self) -> usize {
        *self.active_batches.lock().unwrap()
    }

    /// 等待所有任务完成
    pub async fn flush(&self) {
        // 等待活跃批次完成
        while *self.active_batches.lock().unwrap() > 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // 处理剩余任务
        while self.pending_tasks_count() > 0 {
            self.try_process_batch().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// 便利函数：创建文本生成批量处理器
#[allow(dead_code)]
pub fn create_text_generation_processor(max_batch_size: usize) -> AiBatchProcessor {
    let config = BatchConfig {
        max_batch_size,
        batch_timeout_ms: 50, // 文本生成需要快速响应
        max_concurrent_batches: 20,
        enable_dynamic_batching: true,
        warmup_batch_size: 5,
    };
    AiBatchProcessor::new(config)
}

/// 便利函数：创建图像分类批量处理器
#[allow(dead_code)]
pub fn create_image_classification_processor(max_batch_size: usize) -> AiBatchProcessor {
    let config = BatchConfig {
        max_batch_size,
        batch_timeout_ms: 200, // 图像分类耗时较长
        max_concurrent_batches: 5,
        enable_dynamic_batching: false, // 图像批处理通常固定大小
        warmup_batch_size: 2,
    };
    AiBatchProcessor::new(config)
}

/// 便利函数：创建嵌入向量批量处理器
#[allow(dead_code)]
pub fn create_embedding_processor(max_batch_size: usize) -> AiBatchProcessor {
    let config = BatchConfig {
        max_batch_size,
        batch_timeout_ms: 100,
        max_concurrent_batches: 15,
        enable_dynamic_batching: true,
        warmup_batch_size: 8,
    };
    AiBatchProcessor::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_single_task() {
        let processor = AiBatchProcessor::new(BatchConfig::default());
        let task = AiTaskType::TextGeneration {
            prompt: "Hello, world!".to_string(),
            max_tokens: Some(100),
            temperature: 0.7,
        };

        let task_id = processor.add_task(task).await;
        assert_eq!(task_id, 0);

        // 等待任务被处理（批次处理是异步的）
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(processor.pending_tasks_count(), 0); // 任务已被处理
    }

    #[tokio::test]
    async fn test_add_multiple_tasks() {
        let processor = AiBatchProcessor::new(BatchConfig::default());
        let tasks = vec![
            AiTaskType::TextGeneration {
                prompt: "Task 1".to_string(),
                max_tokens: Some(50),
                temperature: 0.5,
            },
            AiTaskType::TextGeneration {
                prompt: "Task 2".to_string(),
                max_tokens: Some(50),
                temperature: 0.5,
            },
        ];

        let task_ids = processor.add_tasks(tasks).await;
        assert_eq!(task_ids.len(), 2);
        assert_eq!(task_ids, vec![0, 1]);
    }

    #[test]
    fn test_batch_stats() {
        let mut stats = BatchStats::default();
        stats.record_batch(10, Duration::from_millis(100), 10240);
        stats.record_batch(20, Duration::from_millis(200), 20480);

        assert_eq!(stats.total_tasks_processed, 30);
        assert_eq!(stats.total_batches_processed, 2);
        assert_eq!(stats.average_batch_size, 15.0);
        assert_eq!(stats.peak_memory_usage, 20480);
    }

    #[tokio::test]
    async fn test_processor_creation() {
        let processor = create_text_generation_processor(50);
        assert_eq!(processor.config.max_batch_size, 50);
        assert_eq!(processor.config.batch_timeout_ms, 50);
        assert!(processor.config.enable_dynamic_batching);
    }

    #[tokio::test]
    async fn test_flush_processor() {
        let processor = AiBatchProcessor::new(BatchConfig::default());
        let task = AiTaskType::Embedding {
            text: "Test embedding".to_string(),
            model_name: "test-model".to_string(),
        };

        let _task_id = processor.add_task(task).await;
        processor.flush().await;

        assert_eq!(processor.pending_tasks_count(), 0);
        assert_eq!(processor.active_batches_count(), 0);
    }
}
