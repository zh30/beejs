//! 自动化测试套件模块
//! Stage 31.3.2: 自动化性能测试套件
//!
//! 该模块提供完整的自动化测试能力，包括：
//! - 自动化测试运行器
//! - 性能阈值管理
//! - 测试计划调度
//! - 自动化报告生成
pub mod test_runner;
pub mod threshold;
pub mod report_generator;
pub use test_runner::{AutomatedTestRunner, TestSuiteResults, TestType, TestPlanConfig};
pub use threshold::{ThresholdManager, ThresholdConfig};
pub use report_generator::{ReportGenerator, ReportFormat, ReportOutput, ReportType};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};