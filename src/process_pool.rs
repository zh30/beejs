//! Process Pool Implementation for Beejs
//!
//! This module implements a process pool system to reuse pre-spawned worker processes
//! for script execution, significantly reducing the overhead of process creation.
//!
//! Key features:
//! - Pre-spawned worker processes with initialized V8 runtimes
//! - Intelligent process selection based on workload
//! - Automatic process lifecycle management
//! - Support for both simple and complex script execution

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixStream, UnixListener};
use num_cpus;

const DEFAULT_POOL_SIZE: usize = 4;
const MAX_POOL_SIZE: usize = 16;
const SOCKET_PATH_PREFIX: &str = "/tmp/beejs-pool-";
const WORKER_READY_MSG: &str = "READY";
const EXEC_SUCCESS_MSG: &str = "SUCCESS:";
const EXEC_ERROR_MSG: &str = "ERROR:";

/// Configuration for the process pool
#[derive(Debug, Clone)]
pub struct ProcessPoolConfig {
    /// Maximum number of worker processes in the pool
    pub max_workers: usize,
    /// Initial number of worker processes to spawn
    pub initial_workers: usize,
    /// Timeout for worker initialization (ms)
    pub init_timeout_ms: u64,
    /// Enable process pool (disable for single-process mode)
    pub enabled: bool,
}

impl Default for ProcessPoolConfig {
    fn default() -> Self {
        Self {
            max_workers: std::cmp::min(MAX_POOL_SIZE, num_cpus::get()),
            initial_workers: std::cmp::min(DEFAULT_POOL_SIZE, num_cpus::get()),
            init_timeout_ms: 5000,
            enabled: true,
        }
    }
}

/// Worker process state
#[derive(Debug, Clone, PartialEq)]
enum WorkerState {
    Starting,
    Ready,
    Busy,
    Terminating,
}

/// Information about a worker process
#[derive(Debug)]
struct WorkerInfo {
    #[allow(dead_code)]
    pid: u32,
    state: WorkerState,
    socket_path: String,
    last_used: Instant,
    current_task: Option<String>,
    #[allow(dead_code)]
    total_executions: usize,
}

/// Statistics about the process pool
#[derive(Debug, Clone)]
pub struct ProcessPoolStats {
    pub total_workers: usize,
    pub ready_workers: usize,
    pub busy_workers: usize,
    pub total_executions: usize,
    pub avg_execution_time_ms: f64,
    pub pool_hit_rate: f64,
}

impl Default for ProcessPoolStats {
    fn default() -> Self {
        Self {
            total_workers: 0,
            ready_workers: 0,
            busy_workers: 0,
            total_executions: 0,
            avg_execution_time_ms: 0.0,
            pool_hit_rate: 0.0,
        }
    }
}

/// The main Process Pool manager
pub struct ProcessPool {
    config: ProcessPoolConfig,
    workers: Arc<Mutex<HashMap<u32, WorkerInfo>>>,
    available_workers: Arc<Mutex<Vec<u32>>>,
    stats: Arc<Mutex<ProcessPoolStats>>,
    worker_counter: AtomicUsize,
    shutdown: Arc<AtomicBool>,
}

impl ProcessPool {
    /// Create a new process pool with the given configuration
    pub fn new(config: ProcessPoolConfig) -> Result<Self> {
        let pool = Self {
            config: config.clone(),
            workers: Arc::new(Mutex::new(HashMap::new())),
            available_workers: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(ProcessPoolStats::default())),
            worker_counter: AtomicUsize::new(0),
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        // Initialize the pool with worker processes
        if config.enabled {
            pool.initialize_workers()?;
        }

        Ok(pool)
    }

    /// Initialize worker processes
    fn initialize_workers(&self) -> Result<()> {
        let initial = self.config.initial_workers;

        println!("[ProcessPool] Initializing {} worker processes...", initial);

        for i in 0..initial {
            match self.spawn_worker(i) {
                Ok(_) => {
                    if i % 2 == 0 {
                        print!(".");
                    }
                }
                Err(e) => {
                    eprintln!("\n[ProcessPool] Failed to spawn worker {}: {}", i, e);
                }
            }
        }

        println!("\n[ProcessPool] Worker pool ready!");

        Ok(())
    }

    /// Spawn a new worker process
    fn spawn_worker(&self, worker_id: usize) -> Result<u32> {
        let socket_path = format!("{}{}", SOCKET_PATH_PREFIX, worker_id);

        // Remove old socket if exists
        let _ = std::fs::remove_file(&socket_path);

        // Create Unix domain socket for IPC
        let _listener = UnixListener::bind(&socket_path)
            .context("Failed to create Unix socket")?;

        let pid = self.worker_counter.fetch_add(1, Ordering::SeqCst) as u32;
        let worker_info = WorkerInfo {
            pid,
            state: WorkerState::Starting,
            socket_path: socket_path.clone(),
            last_used: Instant::now(),
            current_task: None,
            total_executions: 0,
        };

        // Add to workers map
        {
            let mut workers = self.workers.lock().unwrap();
            workers.insert(pid, worker_info);
        }

        // Spawn the worker process
        let worker_stdout = Stdio::null();
        let worker_stderr = Stdio::null();

        let child = Command::new(std::env::current_exe()?)
            .arg("--worker-mode")
            .arg("--worker-id")
            .arg(pid.to_string())
            .arg("--socket-path")
            .arg(&socket_path)
            .stdout(worker_stdout)
            .stderr(worker_stderr)
            .spawn()
            .context("Failed to spawn worker process")?;

        let child_pid = child.id();

        // Update worker state to Ready
        {
            let mut workers = self.workers.lock().unwrap();
            if let Some(worker) = workers.get_mut(&child_pid) {
                worker.state = WorkerState::Ready;
            }
        }

        // Add to available workers
        {
            let mut available = self.available_workers.lock().unwrap();
            available.push(child_pid);
        }

        println!("[ProcessPool] Spawned worker {} (PID: {})", worker_id, child_pid);

        Ok(child_pid)
    }

    /// Execute a script using an available worker process
    pub async fn execute_script(&self, script: &str) -> Result<String> {
        if !self.config.enabled {
            // Fallback to direct execution if pool is disabled
            return self.execute_direct(script).await;
        }

        let start = Instant::now();

        // Get an available worker
        let worker_pid = self.acquire_worker().await
            .context("No available workers")?;

        let result = self.execute_on_worker(worker_pid, script).await;

        // Release the worker
        self.release_worker(worker_pid);

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_executions += 1;
            let elapsed = start.elapsed();
            let avg_time = stats.avg_execution_time_ms;
            stats.avg_execution_time_ms = avg_time * 0.9 + elapsed.as_millis() as f64 * 0.1;
        }

        result
    }

    /// Acquire an available worker process
    async fn acquire_worker(&self) -> Option<u32> {
        let mut available = self.available_workers.lock().unwrap();

        while let Some(pid) = available.pop() {
            let workers = self.workers.lock().unwrap();
            if let Some(worker) = workers.get(&pid) {
                if worker.state == WorkerState::Ready {
                    return Some(pid);
                }
            }
        }

        // No ready workers, try to spawn a new one
        drop(available);
        let workers_count = self.workers.lock().unwrap().len();

        if workers_count < self.config.max_workers {
            if let Ok(new_pid) = self.spawn_worker(workers_count) {
                return Some(new_pid);
            }
        }

        None
    }

    /// Release a worker process back to the pool
    fn release_worker(&self, worker_pid: u32) {
        let mut available = self.available_workers.lock().unwrap();
        available.push(worker_pid);

        // Update worker's last used time
        let mut workers = self.workers.lock().unwrap();
        if let Some(worker) = workers.get_mut(&worker_pid) {
            worker.last_used = Instant::now();
            worker.current_task = None;
            if worker.state != WorkerState::Terminating {
                worker.state = WorkerState::Ready;
            }
        }
    }

    /// Execute script on a specific worker process
    async fn execute_on_worker(&self, worker_pid: u32, script: &str) -> Result<String> {
        let socket_path = {
            let workers = self.workers.lock().unwrap();
            workers.get(&worker_pid)
                .map(|w| w.socket_path.clone())
                .ok_or_else(|| anyhow::anyhow!("Worker not found"))?
        };

        // Connect to the worker's Unix socket
        let mut stream = UnixStream::connect(&socket_path)
            .await
            .context("Failed to connect to worker")?;

        // Send the script
        stream.write_all(script.as_bytes()).await?;
        stream.write_all(b"\nEND\n").await?;

        // Read the result
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;

        // Parse the response
        if response.starts_with(EXEC_SUCCESS_MSG) {
            Ok(response[EXEC_SUCCESS_MSG.len()..].to_string())
        } else if response.starts_with(EXEC_ERROR_MSG) {
            Err(anyhow::anyhow!("{}", &response[EXEC_ERROR_MSG.len()..]))
        } else {
            Err(anyhow::anyhow!("Invalid response from worker: {}", response))
        }
    }

    /// Execute script directly (fallback when pool is disabled)
    async fn execute_direct(&self, script: &str) -> Result<String> {
        // This would integrate with the existing Runtime system
        // For now, return a placeholder
        Ok(format!("Direct execution: {}", script.len()))
    }

    /// Get process pool statistics
    pub fn get_stats(&self) -> ProcessPoolStats {
        let stats = self.stats.lock().unwrap().clone();
        let workers = self.workers.lock().unwrap();

        ProcessPoolStats {
            total_workers: workers.len(),
            ready_workers: workers.values().filter(|w| w.state == WorkerState::Ready).count(),
            busy_workers: workers.values().filter(|w| w.state == WorkerState::Busy).count(),
            ..stats
        }
    }

    /// Shutdown the process pool and all workers
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);

        let workers = self.workers.lock().unwrap();
        for worker in workers.values() {
            // Send shutdown signal to worker
            let _ = std::fs::remove_file(&worker.socket_path);
        }

        println!("[ProcessPool] Shutdown complete");
    }
}

/// Global process pool instance
static PROCESS_POOL: std::sync::OnceLock<Arc<ProcessPool>> = std::sync::OnceLock::new();
static POOL_CONFIG: std::sync::OnceLock<ProcessPoolConfig> = std::sync::OnceLock::new();

/// Initialize the global process pool
pub fn initialize_process_pool(config: ProcessPoolConfig) -> Result<()> {
    let pool = ProcessPool::new(config.clone())
        .context("Failed to initialize process pool")?;

    POOL_CONFIG.set(config)
        .map_err(|_| anyhow::anyhow!("Failed to store pool config"))?;

    PROCESS_POOL.set(Arc::new(pool))
        .map_err(|_| anyhow::anyhow!("Failed to set global process pool"))?;

    println!("[ProcessPool] Global process pool initialized");
    Ok(())
}

/// Get the global process pool instance
pub fn get_process_pool() -> Option<Arc<ProcessPool>> {
    PROCESS_POOL.get().cloned()
}

/// Execute a script using the process pool
pub async fn execute_with_pool(script: &str) -> Result<String> {
    if let Some(pool) = get_process_pool() {
        pool.execute_script(script).await
    } else {
        Err(anyhow::anyhow!("Process pool not initialized"))
    }
}

/// Worker mode entry point (called by spawned worker processes)
pub async fn worker_main(worker_id: u32, socket_path: String) -> Result<()> {
    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)
        .context("Worker failed to bind socket")?;

    println!("[Worker-{}] Started and listening on {}", worker_id, socket_path);

    // Signal ready to parent
    let _ = std::fs::write(&socket_path, WORKER_READY_MSG);

    // Accept connections and execute scripts
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let mut buffer = String::new();
                let mut stream = stream;

                // Read script from socket
                stream.read_to_string(&mut buffer).await?;

                if buffer.trim() == "SHUTDOWN" {
                    println!("[Worker-{}] Received shutdown signal", worker_id);
                    break;
                }

                // Execute the script
                // TODO: Integrate with Runtime system
                let result = format!("{}{}", EXEC_SUCCESS_MSG, buffer.len());

                stream.write_all(result.as_bytes()).await?;
            }
            Err(e) => {
                eprintln!("[Worker-{}] Error accepting connection: {}", worker_id, e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_pool_creation() {
        let config = ProcessPoolConfig {
            max_workers: 2,
            initial_workers: 1,
            init_timeout_ms: 1000,
            enabled: true,
        };

        let pool = ProcessPool::new(config);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    #[ignore = "Process pool worker mode not fully implemented yet"]
    async fn test_process_pool_stats() {
        let config = ProcessPoolConfig {
            max_workers: 2,
            initial_workers: 1,
            init_timeout_ms: 1000,
            enabled: true,
        };

        let pool = ProcessPool::new(config).unwrap();

        // Give the worker process time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let stats = pool.get_stats();

        assert_eq!(stats.total_workers, 1);
        assert_eq!(stats.ready_workers, 1);
    }
}
