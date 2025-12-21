//! Kubernetes Operator for Beejs Cluster Management
//! 实现 Beejs 集群的 Kubernetes Operator，提供自动化集群管理能力

use anyhow::{Result, Context};
use kube::{
    api::{Api, ListParams, PatchParams, Patch},
    client::Client,
    runtime::{
        controller::Action,
        events::{Event, EventRecorder, Recorder},
        finalizer::{finalizer, Event as Finalizer},
        waiter::Condition,
        Controller, WatchStreamExt,
    },
    Resource, ResourceExt,
};
use k8s_openapi::{
    api::{
        apps::v1::{Deployment, StatefulSet},
        core::v1::{ConfigMap, Service, Endpoints, Node, Pod},
    },
    apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition,
    apimachinery::pkg::{
        api::resource::Quantity,
        apis::meta::v1::{LabelSelector, Time},
        runtime::util::structured::StructuredList,
    },
    chrono::Utc,
};
use serde::{Deserialize, Serialize};
use schemars::{JsonSchema};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Custom Resource Definition for BeejsCluster
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "beejs.io",
    version = "v1",
    kind = "BeejsCluster",
    plural = "beejsclusters",
    shortname = "bc",
    namespaced
)]
pub struct BeejsClusterSpec {
    /// Beejs version to deploy
    pub version: String,
    /// Number of replica pods
    pub nodes: usize,
    /// Cluster configuration
    pub config: ClusterConfig,
    /// Resource requirements
    pub resources: ResourceRequirements,
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClusterConfig {
    /// Namespace for the cluster
    pub namespace: Option<String>,
    /// Image repository
    pub image: Option<String>,
    /// Service type
    pub service_type: Option<String>,
    /// Enable monitoring
    pub monitoring: Option<bool>,
    /// Enable auto-scaling
    pub auto_scaling: Option<bool>,
    /// Node selector labels
    pub node_selector: Option<BTreeMap<String, String>>,
    /// Tolerations
    pub tolerations: Option<Vec<Toleration>>,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceRequirements {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
}

/// Toleration configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Toleration {
    pub key: String,
    pub operator: String,
    pub value: Option<String>,
    pub effect: String,
}

/// BeejsCluster status
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct BeejsClusterStatus {
    /// Current phase of the cluster
    pub phase: ClusterPhase,
    /// Number of ready nodes
    pub ready_nodes: usize,
    /// Total number of nodes
    pub total_nodes: usize,
    /// Last update time
    pub last_update: Option<Time>,
    /// Conditions
    pub conditions: Vec<ClusterCondition>,
}

/// Cluster phases
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ClusterPhase {
    /// Cluster is creating
    Creating,
    /// Cluster is running
    Running,
    /// Cluster is updating
    Updating,
    /// Cluster is scaling
    Scaling,
    /// Cluster is failing
    Failing,
    /// Cluster has failed
    Failed,
    /// Cluster is terminating
    Terminating,
}

impl Default for ClusterPhase {
    fn default() -> Self {
        ClusterPhase::Creating
    }
}

/// Cluster conditions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClusterCondition {
    pub last_transition_time: Time,
    pub message: String,
    pub reason: String,
    pub status: String,
    pub type_: String,
}

/// Operator configuration
#[derive(Debug, Clone)]
pub struct OperatorConfig {
    /// Reconciliation interval
    pub reconcile_interval: Duration,
    /// Maximum concurrent reconciliations
    pub max_concurrent: usize,
    /// Leader election enabled
    pub leader_election: bool,
}

/// BeejsOperator main struct
#[derive(Debug)]
pub struct BeejsOperator {
    /// Kubernetes client
    client: Client,
    /// Operator configuration
    config: OperatorConfig,
    /// Cluster API
    clusters: Api<BeejsCluster>,
    /// Deployment API
    deployments: Api<Deployment>,
    /// StatefulSet API
    statefulsets: Api<StatefulSet>,
    /// Service API
    services: Api<Service>,
    /// ConfigMap API
    configmaps: Api<ConfigMap>,
    /// Event recorder
    recorder: Recorder,
}

impl BeejsOperator {
    /// Create a new BeejsOperator
    pub fn new(client: Client, config: OperatorConfig) -> Self {
        let namespaces = client.clone();
        let recorder = EventRecorder::new(client.clone(), "beejs-operator".to_string());

        Self {
            client: client.clone(),
            config,
            clusters: Api::all(client.clone()),
            deployments: Api::all(client.clone()),
            statefulsets: Api::all(client.clone()),
            services: Api::all(client.clone()),
            configmaps: Api::all(client.clone()),
            recorder,
        }
    }

    /// Run the operator
    pub async fn run(&self) -> Result<()> {
        info!("Starting Beejs Kubernetes Operator");

        let controller = Controller::new(self.clusters.clone(), ListParams::default())
            .run(
                self.reconcile(),
                self.error_policy(),
                Arc::new(self.clone()),
            )
            .await
            .context("Failed to start controller")?;

        info!("Beejs Operator started successfully");

        Ok(())
    }

    /// Reconciliation logic
    fn reconcile(&self) -> Arc<dyn Fn(BeejsCluster) -> Action + Send + Sync + 'static> {
        Arc::new(move |beejs_cluster: BeejsCluster| {
            let client = self.client.clone();
            let clusters = self.clusters.clone();
            let deployments = self.deployments.clone();
            let statefulsets = self.statefulsets.clone();
            let services = self.services.clone();
            let configmaps = self.configmaps.clone();
            let recorder = self.recorder.clone();

            async move {
                let name = beejs_cluster.name_any();
                let namespace = beejs_cluster.namespace().unwrap_or_default();

                info!("Reconciling BeejsCluster: {}/{}", namespace, name);

                // Apply finalizer
                let finalizer_action = finalizer(
                    &clusters,
                    "beejs.io/finalizer",
                    beejs_cluster.clone(),
                    |event| async {
                        match event {
                            Finalizer::Apply(beejs_cluster) => {
                                Self::reconcile_apply(
                                    client.clone(),
                                    beejs_cluster.clone(),
                                    deployments.clone(),
                                    statefulsets.clone(),
                                    services.clone(),
                                    configmaps.clone(),
                                    recorder.clone(),
                                )
                                .await
                            }
                            Finalizer::Cleanup(beejs_cluster) => {
                                Self::reconcile_cleanup(
                                    client.clone(),
                                    beejs_cluster.clone(),
                                    deployments.clone(),
                                    statefulsets.clone(),
                                    services.clone(),
                                    configmaps.clone(),
                                    recorder.clone(),
                                )
                                .await
                            }
                        }
                    },
                )
                .await;

                match finalizer_action {
                    Ok(action) => action,
                    Err(e) => {
                        error!("Reconciliation failed: {}", e);
                        Action::requeue(Duration::from_secs(30))
                    }
                }
            }
        })
    }

    /// Error policy
    fn error_policy(&self) -> Arc<dyn Fn(&kube::Error, &BeejsCluster) -> Action + Send + Sync + 'static> {
        Arc::new(move |_error, _beejs_cluster| {
            warn!("Error occurred during reconciliation");
            Action::requeue(Duration::from_secs(60))
        })
    }

    /// Apply reconciliation
    async fn reconcile_apply(
        client: Client,
        beejs_cluster: BeejsCluster,
        deployments: Api<Deployment>,
        statefulsets: Api<StatefulSet>,
        services: Api<Service>,
        configmaps: Api<ConfigMap>,
        recorder: Recorder,
    ) -> Result<Action> {
        let name = beejs_cluster.name_any();
        let namespace = beejs_cluster.namespace().unwrap_or_default();

        // Create or update ConfigMap
        let configmap = Self::create_configmap(&beejs_cluster)?;
        configmaps
            .patch(
                &format!("{}-config", name),
                &PatchParams::apply("beejs-operator"),
                &Patch::Apply(&configmap),
            )
            .await
            .context("Failed to patch ConfigMap")?;

        // Create or update StatefulSet for the cluster
        let statefulset = Self::create_statefulset(&beejs_cluster)?;
        statefulsets
            .patch(
                &name,
                &PatchParams::apply("beejs-operator"),
                &Patch::Apply(&statefulset),
            )
            .await
            .context("Failed to patch StatefulSet")?;

        // Create or update Service
        let service = Self::create_service(&beejs_cluster)?;
        services
            .patch(
                &format!("{}-svc", name),
                &PatchParams::apply("beejs-operator"),
                &Patch::Apply(&service),
            )
            .await
            .context("Failed to patch Service")?;

        // Update status
        let mut status = beejs_cluster.status.clone().unwrap_or_default();
        status.phase = ClusterPhase::Running;
        status.ready_nodes = beejs_cluster.spec.nodes;
        status.total_nodes = beejs_cluster.spec.nodes;
        status.last_update = Some(Time(Utc::now()));

        let patched = BeejsCluster {
            status: Some(status),
            ..beejs_cluster
        };

        let _ = recorder
            .publish(Event::normal(
                &beejs_cluster,
                &format!("BeejsCluster {} reconciled successfully", name),
            ))
            .await;

        Ok(Action::requeue(Duration::from_secs(30)))
    }

    /// Cleanup reconciliation
    async fn reconcile_cleanup(
        client: Client,
        beejs_cluster: BeejsCluster,
        deployments: Api<Deployment>,
        statefulsets: Api<StatefulSet>,
        services: Api<Service>,
        configmaps: Api<ConfigMap>,
        recorder: Recorder,
    ) -> Result<Action> {
        let name = beejs_cluster.name_any();
        let namespace = beejs_cluster.namespace().unwrap_or_default();

        info!("Cleaning up BeejsCluster: {}/{}", namespace, name);

        // Delete StatefulSet
        if let Err(e) = statefulsets.delete(&name, &Default::default()).await {
            warn!("Failed to delete StatefulSet: {}", e);
        }

        // Delete Service
        if let Err(e) = services
            .delete(&format!("{}-svc", name), &Default::default())
            .await
        {
            warn!("Failed to delete Service: {}", e);
        }

        // Delete ConfigMap
        if let Err(e) = configmaps
            .delete(&format!("{}-config", name), &Default::default())
            .await
        {
            warn!("Failed to delete ConfigMap: {}", e);
        }

        let _ = recorder
            .publish(Event::normal(
                &beejs_cluster,
                &format!("BeejsCluster {} deleted", name),
            ))
            .await;

        Ok(Action::await())
    }

    /// Create ConfigMap for the cluster
    fn create_configmap(beejs_cluster: &BeejsCluster) -> Result<ConfigMap> {
        let name = beejs_cluster.name_any();
        let namespace = beejs_cluster.namespace().unwrap_or_default();

        let mut data = BTreeMap::new();
        data.insert("version".to_string(), beejs_cluster.spec.version.clone());
        data.insert(
            "nodes".to_string(),
            beejs_cluster.spec.nodes.to_string(),
        );

        Ok(ConfigMap {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{}-config", name)),
                namespace: Some(namespace),
                labels: Some(Self::labels_for_cluster(name)),
                ..Default::default()
            },
            data: Some(data),
            ..Default::default()
        })
    }

    /// Create StatefulSet for the cluster
    fn create_statefulset(beejs_cluster: &BeejsCluster) -> Result<StatefulSet> {
        let name = beejs_cluster.name_any();
        let namespace = beejs_cluster.namespace().unwrap_or_default();
        let spec = &beejs_cluster.spec;

        let mut labels = Self::labels_for_cluster(name);
        labels.insert("app".to_string(), "beejs".to_string());

        Ok(StatefulSet {
            metadata: kube::api::ObjectMeta {
                name: Some(name.clone()),
                namespace: Some(namespace),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::apps::v1::StatefulSetSpec {
                service_name: format!("{}-svc", name),
                replicas: Some(spec.nodes as i32),
                selector: LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                template: k8s_openapi::api::core::v1::PodTemplateSpec {
                    metadata: Some(kube::api::ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(k8s_openapi::api::core::v1::PodSpec {
                        containers: vec![k8s_openapi::api::core::v1::Container {
                            name: "beejs".to_string(),
                            image: Some(spec.config.image.clone().unwrap_or_default()),
                            ports: Some(vec![k8s_openapi::api::core::v1::ContainerPort {
                                container_port: 3000,
                                ..Default::default()
                            }]),
                            resources: Some(k8s_openapi::api::core::v1::ResourceRequirements {
                                requests: Some(BTreeMap::from([
                                    ("cpu".to_string(), Quantity(spec.resources.cpu.clone().unwrap_or_default())),
                                    ("memory".to_string(), Quantity(spec.resources.memory.clone().unwrap_or_default())),
                                ])),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Create Service for the cluster
    fn create_service(beejs_cluster: &BeejsCluster) -> Result<Service> {
        let name = beejs_cluster.name_any();
        let namespace = beejs_cluster.namespace().unwrap_or_default();

        let labels = Self::labels_for_cluster(name);

        Ok(Service {
            metadata: kube::api::ObjectMeta {
                name: Some(format!("{}-svc", name)),
                namespace: Some(namespace),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                type_: Some(
                    beejs_cluster
                        .spec
                        .config
                        .service_type
                        .clone()
                        .unwrap_or_else(|| "ClusterIP".to_string()),
                ),
                selector: Some(labels),
                ports: Some(vec![k8s_openapi::api::core::v1::ServicePort {
                    port: 3000,
                    target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(3000)),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    /// Generate labels for cluster resources
    fn labels_for_cluster(name: &str) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("beejs.io/cluster".to_string(), name.to_string());
        labels.insert("beejs.io/component".to_string(), "cluster".to_string());
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_operator() {
        let client = Client::try_default().await.unwrap();
        let config = OperatorConfig {
            reconcile_interval: Duration::from_secs(30),
            max_concurrent: 10,
            leader_election: true,
        };

        let operator = BeejsOperator::new(client, config);
        assert_eq!(operator.config.reconcile_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_create_configmap() {
        let cluster = BeejsCluster::new(
            "test-cluster",
            BeejsClusterSpec {
                version: "v1.0.0".to_string(),
                nodes: 3,
                config: ClusterConfig {
                    namespace: Some("default".to_string()),
                    image: Some("beejs:latest".to_string()),
                    service_type: Some("ClusterIP".to_string()),
                    monitoring: Some(true),
                    auto_scaling: Some(true),
                    node_selector: None,
                    tolerations: None,
                },
                resources: ResourceRequirements {
                    cpu: Some("500m".to_string()),
                    memory: Some("1Gi".to_string()),
                    storage: Some("10Gi".to_string()),
                },
            },
        );

        let configmap = BeejsOperator::create_configmap(&cluster).unwrap();
        assert_eq!(configmap.metadata.name, Some("test-cluster-config".to_string()));
    }

    #[test]
    fn test_labels_for_cluster() {
        let labels = BeejsOperator::labels_for_cluster("test-cluster");
        assert_eq!(
            labels.get("beejs.io/cluster"),
            Some(&"test-cluster".to_string())
        );
        assert_eq!(
            labels.get("beejs.io/component"),
            Some(&"cluster".to_string())
        );
    }
}
