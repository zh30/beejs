// Stage 91 Phase 3 生态系统集成测试套件
// Stage 91 Phase 3 - 生态系统集成
//
// 测试包管理器集成、开发工具支持、框架支持等核心功能

#[cfg(test)]
mod package_manager_tests;
#[cfg(test)]
mod type_generator_tests;
#[cfg(test)]
mod framework_support_tests;
#[cfg(test)]
mod ecosystem_integration_tests;

pub use package_manager_tests::*;
pub use type_generator_tests::*;
pub use framework_support_tests::*;
pub use ecosystem_integration_tests::*;

use std::collections::HashMap;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_package_manager_integration() {
        let test_cases = vec![
            test_npm_compatibility,
            test_yarn_compatibility,
            test_pnpm_compatibility,
            test_package_resolution,
            test_lockfile_parsing,
        ];

        for test_case in test_cases {
            test_case().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_type_generator() {
        let test_cases = vec![
            test_type_generation_from_source,
            test_jsdoc_type_extraction,
            test_dts_emission,
            test_project_type_generation,
        ];

        for test_case in test_cases {
            test_case().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_framework_support() {
        let test_cases = vec![
            test_react_runtime,
            test_vue_runtime,
            test_angular_runtime,
            test_ssr_rendering,
        ];

        for test_case in test_cases {
            test_case().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_ecosystem_integration() {
        let test_cases = vec![
            test_end_to_end_workflow,
            test_performance_metrics,
            test_resource_usage,
        ];

        for test_case in test_cases {
            test_case().await.unwrap();
        }
    }
}
