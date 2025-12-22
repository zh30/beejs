//! AWS 云平台适配器
//! Stage 39.0: 多云平台适配层
//!
//! 该模块提供 AWS 云平台支持，包括 Lambda、ECS、EKS、EC2 等服务

use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::{CloudAdapter, DeploymentResult, FunctionConfig, Metrics};

/// AWS 适配器
pub struct AwsAdapter {
    region: String,
    credentials: Option<AwsCredentials>,
}

/// AWS 凭据
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

/// AWS Lambda 配置
#[derive(Debug, Clone)]
pub struct LambdaConfig {
    pub memory_size: u32,
    pub timeout: u32,
    pub runtime: String,
    pub handler: String,
    pub environment: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
    pub layers: Vec<String>,
    pub dead_letter_config: Option<String>,
}

/// AWS ECS 配置
#[derive(Debug, Clone)]
pub struct EcsConfig {
    pub cluster: String,
    pub service: String,
    pub task_definition: String,
    pub desired_count: u32,
    pub launch_type: String,
    pub network_configuration: Option<NetworkConfig>,
}

/// AWS EKS 配置
#[derive(Debug, Clone)]
pub struct EksConfig {
    pub cluster: String,
    pub namespace: String,
    pub deployment: String,
    pub replicas: u32,
    pub container_image: String,
    pub service_type: String,
}

/// 网络配置
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
    pub assign_public_ip: bool,
}

/// AWS EC2 配置
#[derive(Debug, Clone)]
pub struct Ec2Config {
    pub instance_type: String,
    pub ami_id: String,
    pub key_name: String,
    pub security_groups: Vec<String>,
    pub user_data: Option<String>,
    pub tags: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
}

impl AwsAdapter {
    /// 创建新的 AWS 适配器
    pub fn new(region: String) -> Self {
        Self {
            region,
            credentials: None,
        }
    }

    /// 设置 AWS 凭据
    pub fn set_credentials(&mut self, credentials: AwsCredentials) {
        self.credentials = Some(credentials);
    }

    /// 部署 Lambda 函数
    pub async fn deploy_lambda(&self, config: &LambdaConfig) -> Result<DeploymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time: _ = Instant::now();

        println!("🚀 开始部署 AWS Lambda 函数");
        println!("  区域: {}", self.region);
        println!("  内存: {} MB", config.memory_size);
        println!("  超时: {} 秒", config.timeout);
        println!("  运行时: {}", config.runtime);

        // 模拟部署延迟
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 模拟部署过程
        println!("  📦 打包函数代码...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  🔄 上传到 AWS...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("  ⚙️  配置环境变量...");
        tokio::time::sleep(Duration::from_millis(30)).await;

        let deployment_time: _ = start_time.elapsed();

        let result: _ = DeploymentResult {
            deployment_id: format!("lambda-{}", chrono::Utc::now().timestamp()),
            status: "success".to_string(),
            endpoint: format!("https://lambda.{}.amazonaws.com/function", self.region),
            deployment_time,
            message: "Lambda 函数部署成功".to_string(),
        };

        println!("✅ Lambda 函数部署成功! 耗时: {:?}", deployment_time);
        Ok(result)
    }

    /// 部署 ECS 服务
    pub async fn deploy_ecs(&self, config: &EcsConfig) -> Result<DeploymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time: _ = Instant::now();

        println!("🚀 开始部署 AWS ECS 服务");
        println!("  集群: {}", config.cluster);
        println!("  服务: {}", config.service);
        println!("  任务定义: {}", config.task_definition);
        println!("  期望数量: {}", config.desired_count);

        // 模拟部署延迟
        tokio::time::sleep(Duration::from_millis(150)).await;

        println!("  🔄 注册任务定义...");
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("  🚀 更新服务...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let deployment_time: _ = start_time.elapsed();

        let result: _ = DeploymentResult {
            deployment_id: format!("ecs-{}-{}", config.cluster, config.service),
            status: "success".to_string(),
            endpoint: format!("https://{}.amazonaws.com", config.cluster),
            deployment_time,
            message: "ECS 服务部署成功".to_string(),
        };

        println!("✅ ECS 服务部署成功! 耗时: {:?}", deployment_time);
        Ok(result)
    }

    /// 部署 EKS 部署
    pub async fn deploy_eks(&self, config: &EksConfig) -> Result<DeploymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time: _ = Instant::now();

        println!("🚀 开始部署 AWS EKS 部署");
        println!("  集群: {}", config.cluster);
        println!("  命名空间: {}", config.namespace);
        println!("  部署: {}", config.deployment);
        println!("  副本数: {}", config.replicas);
        println!("  容器镜像: {}", config.container_image);

        // 模拟部署延迟
        tokio::time::sleep(Duration::from_millis(200)).await;

        println!("  🔄 应用 Kubernetes 清单...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("  ⏳ 等待 Pod 启动...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let deployment_time: _ = start_time.elapsed();

        let result: _ = DeploymentResult {
            deployment_id: format!("eks-{}-{}", config.cluster, config.deployment),
            status: "success".to_string(),
            endpoint: format!("https://{}.eks.{}.amazonaws.com", config.deployment, self.region),
            deployment_time,
            message: "EKS 部署成功".to_string(),
        };

        println!("✅ EKS 部署成功! 耗时: {:?}", deployment_time);
        Ok(result)
    }

    /// 启动 EC2 实例
    pub async fn launch_ec2(&self, config: &Ec2Config) -> Result<DeploymentResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time: _ = Instant::now();

        println!("🚀 开始启动 AWS EC2 实例");
        println!("  实例类型: {}", config.instance_type);
        println!("  AMI ID: {}", config.ami_id);
        println!("  密钥对: {}", config.key_name);
        println!("  安全组: {:?}", config.security_groups);

        // 模拟启动延迟
        tokio::time::sleep(Duration::from_millis(300)).await;

        println!("  🔄 启动实例...");
        tokio::time::sleep(Duration::from_millis(200)).await;

        println!("  ⏳ 等待实例运行...");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let deployment_time: _ = start_time.elapsed();

        let result: _ = DeploymentResult {
            deployment_id: format!("ec2-{}", chrono::Utc::now().timestamp()),
            status: "success".to_string(),
            endpoint: "https://console.aws.amazon.com/ec2".to_string(),
            deployment_time,
            message: "EC2 实例启动成功".to_string(),
        };

        println!("✅ EC2 实例启动成功! 耗时: {:?}", deployment_time);
        Ok(result)
    }

    /// 获取 Lambda 函数指标
    pub async fn get_lambda_metrics(&self, function_name: &str) -> Result<Metrics, Box<dyn std::error::Error + Send + Sync>> {
        println!("📊 获取 Lambda 函数 '{}' 指标", function_name);

        // 模拟指标获取
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(Metrics {
            cpu_usage: 45.5,
            memory_usage: 62.3,
            network_io: 1234.5,
            disk_io: 567.8,
            request_count: 1000,
            error_count: 5,
            average_latency: 120.5,
        })
    }

    /// 获取 ECS 服务指标
    pub async fn get_ecs_metrics(&self, service_name: &str) -> Result<Metrics, Box<dyn std::error::Error + Send + Sync>> {
        println!("📊 获取 ECS 服务 '{}' 指标", service_name);

        // 模拟指标获取
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(Metrics {
            cpu_usage: 55.2,
            memory_usage: 71.8,
            network_io: 2345.6,
            disk_io: 789.1,
            request_count: 2000,
            error_count: 3,
            average_latency: 95.3,
        })
    }

    /// 获取 EKS 部署指标
    pub async fn get_eks_metrics(&self, deployment_name: &str) -> Result<Metrics, Box<dyn std::error::Error + Send + Sync>> {
        println!("📊 获取 EKS 部署 '{}' 指标", deployment_name);

        // 模拟指标获取
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(Metrics {
            cpu_usage: 38.7,
            memory_usage: 55.9,
            network_io: 3456.7,
            disk_io: 890.2,
            request_count: 1500,
            error_count: 2,
            average_latency: 85.7,
        })
    }
}

impl CloudAdapter for AwsAdapter {
    async fn deploy_function(&self, config: &FunctionConfig) -> Result<DeploymentResult, Box<dyn std::error::Error + Send + Sync>> {
        // 这里应该根据配置决定部署到哪个 AWS 服务
        // 为了简化，我们假设部署到 Lambda
        let lambda_config: _ = LambdaConfig {
            memory_size: config.memory_size.unwrap_or(512),
            timeout: config.timeout.unwrap_or(30),
            runtime: config.runtime.clone(),
            handler: config.handler.clone(),
            environment: config.environment.clone(),
            layers: vec![],
            dead_letter_config: None,
        };

        self.deploy_lambda(&lambda_config).await
    }

    async fn invoke_function(&self, name: &str, payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 调用 AWS Lambda 函数: {}", name);
        println!("  负载大小: {} bytes", payload.len());

        // 模拟函数调用
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 返回模拟响应
        Ok(format!("Response from {}", name).into_bytes())
    }

    async fn scale_service(&self, service: &str, replicas: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔄 扩缩容 AWS 服务: {} -> {} 个副本", service, replicas);

        // 模拟扩缩容延迟
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("✅ 扩缩容成功");
        Ok(())
    }

    async fn get_metrics(&self, service: &str) -> Result<Metrics, Box<dyn std::error::Error + Send + Sync>> {
        // 模拟指标获取
        tokio::time::sleep(Duration::from_millis(50)).await;

        Ok(Metrics {
            cpu_usage: 50.0,
            memory_usage: 60.0,
            network_io: 1000.0,
            disk_io: 500.0,
            request_count: 1000,
            error_count: 0,
            average_latency: 100.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    /// 测试创建 AWS 适配器
    #[tokio::test]
    async fn test_aws_adapter_creation() {
        let adapter: _ = AwsAdapter::new("us-west-2".to_string());
        assert_eq!(adapter.region, "us-west-2");
        println!("✅ 测试通过: AWS 适配器创建");
    }

    /// 测试 Lambda 函数部署
    #[tokio::test]
    async fn test_lambda_deployment() {
        let adapter: _ = AwsAdapter::new("us-east-1".to_string());

        let config: _ = LambdaConfig {
            memory_size: 512,
            timeout: 30,
            runtime: "nodejs18.x".to_string(),
            handler: "index.handler".to_string(),
            environment: HashMap::new(),
            layers: vec![],
            dead_letter_config: None,
        };

        let result: _ = adapter.deploy_lambda(&config).await.expect("部署失败");
        assert_eq!(result.status, "success");
        println!("✅ 测试通过: Lambda 函数部署");
    }

    /// 测试 ECS 服务部署
    #[tokio::test]
    async fn test_ecs_deployment() {
        let adapter: _ = AwsAdapter::new("eu-west-1".to_string());

        let config: _ = EcsConfig {
            cluster: "my-cluster".to_string(),
            service: "my-service".to_string(),
            task_definition: "my-task:1".to_string(),
            desired_count: 3,
            launch_type: "FARGATE".to_string(),
            network_configuration: None,
        };

        let result: _ = adapter.deploy_ecs(&config).await.expect("部署失败");
        assert_eq!(result.status, "success");
        println!("✅ 测试通过: ECS 服务部署");
    }

    /// 测试 EKS 部署
    #[tokio::test]
    async fn test_eks_deployment() {
        let adapter: _ = AwsAdapter::new("ap-southeast-1".to_string());

        let config: _ = EksConfig {
            cluster: "my-eks-cluster".to_string(),
            namespace: "default".to_string(),
            deployment: "my-deployment".to_string(),
            replicas: 3,
            container_image: "nginx:latest".to_string(),
            service_type: "ClusterIP".to_string(),
        };

        let result: _ = adapter.deploy_eks(&config).await.expect("部署失败");
        assert_eq!(result.status, "success");
        println!("✅ 测试通过: EKS 部署");
    }

    /// 测试 EC2 实例启动
    #[tokio::test]
    async fn test_ec2_launch() {
        let adapter: _ = AwsAdapter::new("us-west-2".to_string());

        let config: _ = Ec2Config {
            instance_type: "t3.micro".to_string(),
            ami_id: "ami-0abcdef1234567890".to_string(),
            key_name: "my-keypair".to_string(),
            security_groups: vec!["sg-12345678".to_string()],
            user_data: None,
            tags: HashMap::new(),
        };

        let result: _ = adapter.launch_ec2(&config).await.expect("启动失败");
        assert_eq!(result.status, "success");
        println!("✅ 测试通过: EC2 实例启动");
    }

    /// 测试指标获取
    #[tokio::test]
    async fn test_metrics_collection() {
        let adapter: _ = AwsAdapter::new("us-east-1".to_string());

        let metrics: _ = adapter.get_lambda_metrics("test-function").await.expect("获取指标失败");
        assert!(metrics.cpu_usage > 0.0);
        assert!(metrics.memory_usage > 0.0);
        println!("✅ 测试通过: 指标获取");
    }
}
