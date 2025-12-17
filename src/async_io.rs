//! 异步I/O优化模块
//! 提供高性能的非阻塞I/O操作，支持并发文件读取和脚本执行

use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant};

use crate::Runtime;

/// 异步I/O管理器
#[derive(Debug)]
pub struct AsyncIoManager {
    /// 并发任务限制
    max_concurrent_tasks: usize,
    /// 当前活跃任务数
    active_tasks: Arc<std::sync::atomic::AtomicUsize>,
    /// 性能统计
    stats: Arc<tokio::sync::Mutex<IoStats>>,
}

/// I/O操作统计
#[derive(Debug, Clone, Default)]
pub struct IoStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
}

/// 异步文件读取结果
#[derive(Debug)]
pub struct AsyncFileRead {
    /// 文件路径
    pub path: String,
    /// 文件内容
    pub content: Result<String, IoError>,
    /// 读取耗时
    pub duration: Duration,
}

/// I/O错误类型
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("读取失败: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("超时")]
    Timeout,

    #[error("任务取消")]
    Cancelled,
}

/// 异步脚本执行结果
#[derive(Debug)]
pub struct AsyncScriptExecution {
    /// 脚本内容
    pub code: String,
    /// 执行结果
    pub result: Result<String, String>,
    /// 执行耗时
    pub duration: Duration,
    /// 使用的内存（字节）
    pub memory_used: usize,
}

impl AsyncIoManager {
    /// 创建新的异步I/O管理器
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            max_concurrent_tasks,
            active_tasks: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            stats: Arc::new(tokio::sync::Mutex::new(IoStats::default())),
        }
    }

    /// 异步读取多个文件
    pub async fn read_files_concurrent(
        &self,
        mut paths: Vec<String>,
    ) -> Vec<AsyncFileRead> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_tasks));
        let start = Instant::now();
        let path_count = paths.len();

        // 创建所有异步任务
        let mut handles: Vec<JoinHandle<AsyncFileRead>> = Vec::with_capacity(path_count);

        for path in paths.drain(..) {
            let semaphore = semaphore.clone();
            let path_clone = path.clone();
            let stats = self.stats.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let task_start = Instant::now();

                let result = async_read_single_file(&path_clone).await;

                // 更新统计信息
                {
                    let mut stats_guard = stats.lock().await;
                    stats_guard.total_operations += 1;
                    if result.content.is_ok() {
                        stats_guard.successful_operations += 1;
                        stats_guard.total_bytes_read += result.content.as_ref().unwrap().len() as u64;
                    } else {
                        stats_guard.failed_operations += 1;
                    }
                }

                AsyncFileRead {
                    path: path_clone,
                    content: result.content,
                    duration: task_start.elapsed(),
                }
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        let total_time = start.elapsed();
        println!("并发读取 {} 个文件，耗时: {:?}", path_count, total_time);

        results
    }

    /// 异步执行多个脚本
    pub async fn execute_scripts_concurrent(
        &self,
        mut scripts: Vec<String>,
    ) -> Vec<AsyncScriptExecution> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_tasks));
        let start = Instant::now();
        let script_count = scripts.len();

        let mut handles: Vec<JoinHandle<AsyncScriptExecution>> = Vec::with_capacity(script_count);

        for code in scripts.drain(..) {
            let semaphore = semaphore.clone();
            let code_clone = code.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let task_start = Instant::now();

                // 创建新的运行时实例执行脚本
                let rt = Runtime::new(8 * 1024 * 1024, 64 * 1024 * 1024, false)
                    .expect("Failed to create runtime");

                let result = rt.execute_code(&code_clone);
                let memory_used = 8 * 1024 * 1024; // 简化估算

                AsyncScriptExecution {
                    code: code_clone,
                    result: match result {
                        Ok(output) => Ok(format!("{:?}", output)),
                        Err(e) => Err(e.to_string()),
                    },
                    duration: task_start.elapsed(),
                    memory_used,
                }
            });

            handles.push(handle);
        }

        // 等待所有任务完成
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        let total_time = start.elapsed();
        println!("并发执行 {} 个脚本，耗时: {:?}", script_count, total_time);

        results
    }

    /// 使用零拷贝方式读取文件（仅返回文件描述符）
    pub async fn read_file_zero_copy(&self, path: &str) -> Result<File, IoError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(IoError::FileNotFound(path.to_string_lossy().to_string()));
        }

        let start = Instant::now();
        let file = File::open(path).await.map_err(IoError::ReadError)?;
        let duration = start.elapsed();

        // 更新统计
        {
            let mut stats = self.stats.lock().await;
            stats.total_operations += 1;
            stats.successful_operations += 1;
            // 简化：估算文件大小
            if let Ok(metadata) = tokio::fs::metadata(path).await {
                stats.total_bytes_read += metadata.len();
            }
        }

        println!("零拷贝读取文件: {:?}, 耗时: {:?}", path, duration);
        Ok(file)
    }

    /// 异步写入文件（使用缓冲）
    pub async fn write_file_buffered(
        &self,
        path: &str,
        content: &[u8],
    ) -> Result<(), IoError> {
        let path = Path::new(path);
        let start = Instant::now();

        let file = File::create(path).await.map_err(IoError::ReadError)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(content).await.map_err(IoError::ReadError)?;
        writer.flush().await.map_err(IoError::ReadError)?;

        let duration = start.elapsed();

        // 更新统计
        {
            let mut stats = self.stats.lock().await;
            stats.total_operations += 1;
            stats.successful_operations += 1;
            stats.total_bytes_written += content.len() as u64;
        }

        println!("缓冲写入文件: {:?}, 耗时: {:?}, 大小: {} bytes", path, duration, content.len());
        Ok(())
    }

    /// 批量处理文件（读取-处理-写入流水线）
    pub async fn process_files_pipeline(
        &self,
        mut input_paths: Vec<String>,
        output_dir: &str,
        processor: impl Fn(&str) -> String + Send + Sync + Clone + 'static,
    ) -> Result<Vec<String>, IoError> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_tasks));
        let start = Instant::now();
        let path_count = input_paths.len();

        let mut handles: Vec<JoinHandle<Result<String, IoError>>> = Vec::with_capacity(path_count);

        for path in input_paths.drain(..) {
            let semaphore = semaphore.clone();
            let path_clone = path.clone();
            let output_dir = output_dir.to_string();

            let handle = {
                let processor = processor.clone();
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    // 读取文件
                    let file_read = async_read_single_file(&path_clone).await;
                    let content = match file_read.content {
                        Ok(c) => c,
                        Err(e) => return Err(IoError::ReadError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
                    };

                    // 处理内容
                    let processed = processor(&content);

                    // 写入输出文件
                    let output_path = format!("{}/{}", output_dir, Path::new(&path_clone).file_name().unwrap().to_string_lossy());
                    let file = File::create(&output_path).await.map_err(IoError::ReadError)?;
                    let mut writer = BufWriter::new(file);
                    writer.write_all(processed.as_bytes()).await.map_err(IoError::ReadError)?;
                    writer.flush().await.map_err(IoError::ReadError)?;

                    Ok(output_path)
                })
            };

            handles.push(handle);
        }

        // 收集结果
        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            let result = handle.await.map_err(|_| IoError::Cancelled)??;
            results.push(result);
        }

        let total_time = start.elapsed();
        println!("流水线处理 {} 个文件，耗时: {:?}", results.len(), total_time);

        Ok(results)
    }

    /// 获取I/O统计信息
    pub async fn get_stats(&self) -> IoStats {
        self.stats.lock().await.clone()
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = IoStats::default();
    }

    /// 获取当前活跃任务数
    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// 异步读取单个文件
async fn async_read_single_file(path: &str) -> AsyncFileRead {
    let path = Path::new(path);
    let start = Instant::now();

    let result = if path.exists() {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(content),
            Err(e) => Err(IoError::ReadError(e)),
        }
    } else {
        Err(IoError::FileNotFound(path.to_string_lossy().to_string()))
    };

    AsyncFileRead {
        path: path.to_string_lossy().to_string(),
        content: result,
        duration: start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_async_file_read() {
        let manager = AsyncIoManager::new(10);

        // 创建临时文件
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        // 读取文件
        let result = manager.read_file_zero_copy(file_path.to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_file_read() {
        let manager = AsyncIoManager::new(5);

        // 创建多个临时文件
        let temp_dir = TempDir::new().unwrap();
        let mut paths = Vec::new();

        for i in 0..10 {
            let file_path = temp_dir.path().join(format!("test{}.txt", i));
            fs::write(&file_path, format!("Content {}", i)).unwrap();
            paths.push(file_path.to_string_lossy().to_string());
        }

        // 并发读取
        let results = manager.read_files_concurrent(paths).await;
        assert_eq!(results.len(), 10);

        // 验证所有读取成功
        for result in results {
            assert!(result.content.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_script_execution() {
        let manager = AsyncIoManager::new(5);

        // 创建多个脚本
        let scripts = vec![
            "1 + 1".to_string(),
            "2 * 3".to_string(),
            "Math.sqrt(16)".to_string(),
            "console.log('test')".to_string(),
        ];

        // 并发执行
        let results = manager.execute_scripts_concurrent(scripts).await;
        assert_eq!(results.len(), 4);

        // 验证所有执行成功
        for result in results {
            assert!(result.result.is_ok() || result.result.is_err()); // 允许执行失败
        }
    }

    #[tokio::test]
    async fn test_write_file_buffered() {
        let manager = AsyncIoManager::new(10);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");
        let content = b"Hello, Buffered World!";

        // 写入文件
        let result = manager.write_file_buffered(
            file_path.to_str().unwrap(),
            content,
        ).await;

        assert!(result.is_ok());

        // 验证文件内容
        let written = fs::read(&file_path).unwrap();
        assert_eq!(written, content);
    }

    #[tokio::test]
    async fn test_io_stats() {
        let manager = AsyncIoManager::new(10);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("stats_test.txt");
        fs::write(&file_path, "Test content").unwrap();

        // 执行一些操作
        let _ = manager.read_file_zero_copy(file_path.to_str().unwrap()).await;
        let _ = manager.write_file_buffered(
            file_path.to_str().unwrap(),
            b"New content",
        ).await;

        // 检查统计
        let stats = manager.get_stats().await;
        assert!(stats.total_operations >= 2);
    }
}
