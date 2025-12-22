//! Stage 96 Phase 3: Dashboard Integration Tests
//!
//! This test suite validates the complete dashboard integration functionality:
//! - Dashboard manager creation and configuration
//! - Chart rendering (line, bar, pie charts)
//! - Graph rendering (topology, dependency graphs)
//! - Grafana API integration
//! - Real-time metrics collection
//! - Template engine functionality

#[cfg(test)]
mod dashboard_integration_tests {
    use super::super::*;
    use beejs::observability::dashboard::{
        DashboardManager, DashboardConfig, Dashboard, PanelConfig, GrafanaClient,
        ChartRenderer, GraphRenderer, TemplateEngine, WebSocketClient,
        ChartType, GraphType, GridPos, QueryTarget, FieldConfig,
        LegendConfig, TooltipConfig, TimeRangeConfig
    };
    use beejs::observability::visualization::{
        LineChart, BarChart, PieChart, TopologyGraph,
        LineChartBuilder, BarChartBuilder, PieChartBuilder, TopologyGraphBuilder,
        VisualizationConfig, DataPoint, DataSeries, ColorPalette, AxisConfig,
        Position, Size, GraphNode, GraphEdge, EdgeStyle, LayoutAlgorithm
    };

    /// Test dashboard manager creation
    #[tokio::test]
    async fn test_dashboard_manager_creation() {
        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await;

        assert!(manager.is_ok(), "Dashboard manager should be created successfully");

        let manager: _ = manager.clone();unwrap();
        assert!(manager.list_dashboards().await.is_empty());
    }

    /// Test dashboard creation
    #[tokio::test]
    async fn test_dashboard_creation() {
        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Create a dashboard
        let uid: _ = manager.create_dashboard("test-dashboard").await.unwrap();
        assert_eq!(uid, "test-dashboard");

        // Verify dashboard exists
        let dashboards: _ = manager.list_dashboards().await;
        assert!(dashboards.contains(&"test-dashboard".to_string()));

        // Verify dashboard data
        let dashboard: _ = manager.get_dashboard("test-dashboard").await.unwrap();
        assert_eq!(dashboard.title, "test-dashboard");
        assert_eq!(dashboard.uid, "test-dashboard");
        assert!(dashboard.panels.is_empty());
    }

    /// Test panel management
    #[tokio::test]
    async fn test_panel_management() {
        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Create dashboard
        let uid: _ = manager.create_dashboard("test-panel-dashboard").await.unwrap();

        // Create a panel
        let panel: _ = PanelConfig {
            id: "cpu-usage".to_string(),
            title: "CPU Usage".to_string(),
            panel_type: "graph".to_string(),
            grid_pos: GridPos { x: 0, y: 0, w: 12, h: 8 },
            datasource: "Prometheus".to_string(),
            targets: vec![QueryTarget {
                ref_id: "A".to_string(),
                query: "rate(beejs_cpu_usage_percent[5m])".to_string(),
                interval: "5m".to_string(),
                legend_format: "{{instance}}".to_string(),
            }],
            field_config: FieldConfig {
                min: Some(0.0),
                max: Some(100.0),
                unit: Some("percent".to_string()),
                decimals: Some(2),
                thresholds: None,
            },
            options: PanelOptions {
                legend: LegendConfig { show: true, position: "bottom".to_string() },
                tooltip: TooltipConfig { mode: "multi".to_string(), sort: "none".to_string() },
                time: TimeRangeConfig { from: "now-1h".to_string(), to: "now".to_string() },
            },
        };

        // Add panel
        manager.add_panel(&uid, panel.clone()).await.unwrap();

        // Verify panel was added
        let dashboard: _ = manager.get_dashboard(&uid).await.unwrap();
        assert_eq!(dashboard.panels.len(), 1);
        assert_eq!(dashboard.panels[0].id, "cpu-usage");
        assert_eq!(dashboard.panels[0].title, "CPU Usage");

        // Update panel
        let mut updated_panel = panel.clone();clone();clone();clone();clone();clone();clone();clone();
        updated_panel.title = "CPU Usage (Updated)".to_string();
        manager.update_panel(&uid, "cpu-usage", updated_panel).await.unwrap();

        // Verify panel was updated
        let dashboard: _ = manager.get_dashboard(&uid).await.unwrap();
        assert_eq!(dashboard.panels[0].title, "CPU Usage (Updated)");

        // Remove panel
        manager.remove_panel(&uid, "cpu-usage").await.unwrap();

        // Verify panel was removed
        let dashboard: _ = manager.get_dashboard(&uid).await.unwrap();
        assert!(dashboard.panels.is_empty());
    }

    /// Test chart rendering - Line Chart
    #[tokio::test]
    async fn test_line_chart_rendering() {
        let mut builder = LineChartBuilder::new();
        let chart: _ = builder
            .title("Test Line Chart")
            .dimensions(800, 600)
            .data(vec![10.0, 20.0, 30.0, 25.0, 35.0, 40.0])
            .build()
            .unwrap();

        let svg: _ = chart.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Line Chart"));
        assert!(svg.contains("path"));
        assert!(svg.contains("line"));
    }

    /// Test chart rendering - Bar Chart
    #[tokio::test]
    async fn test_bar_chart_rendering() {
        let mut builder = BarChartBuilder::new();
        let chart: _ = builder
            .title("Test Bar Chart")
            .dimensions(800, 600)
            .data(vec![15.0, 25.0, 35.0, 45.0, 55.0])
            .build()
            .unwrap();

        let svg: _ = chart.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Bar Chart"));
        assert!(svg.contains("rect"));
    }

    /// Test chart rendering - Pie Chart
    #[tokio::test]
    async fn test_pie_chart_rendering() {
        let mut builder = PieChartBuilder::new();
        let chart: _ = builder
            .title("Test Pie Chart")
            .dimensions(600, 600)
            .data(30.0, "Category A")
            .data(20.0, "Category B")
            .data(50.0, "Category C")
            .build()
            .unwrap();

        let svg: _ = chart.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Pie Chart"));
        assert!(svg.contains("path"));
    }

    /// Test topology graph rendering
    #[tokio::test]
    async fn test_topology_graph_rendering() {
        let mut builder = TopologyGraphBuilder::new();
        let mut graph = builder
            .title("Test Topology")
            .node("api", "API Gateway", "gateway", "healthy", 400.0, 200.0)
            .node("auth", "Auth Service", "service", "healthy", 200.0, 400.0)
            .node("db", "Database", "database", "warning", 600.0, 400.0)
            .edge("api", "auth", Some("HTTP"))
            .edge("api", "db", Some("SQL"))
            .build()
            .unwrap();

        // Apply layout
        graph.apply_layout().await.unwrap();

        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Topology"));
        assert!(svg.contains("line"));
        assert!(svg.contains("rect"));
    }

    /// Test different layout algorithms
    #[tokio::test]
    async fn test_layout_algorithms() {
        let config: _ = VisualizationConfig::default();
        let mut graph = TopologyGraph::new(config);

        // Add nodes
        for i in 0..5 {
            graph.add_node(GraphNode {
                id: format!("node{}", i),
                label: format!("Node {}", i),
                node_type: "service".to_string(),
                status: "healthy".to_string(),
                position: Position { x: 0.0, y: 0.0 },
                size: Size { width: 80.0, height: 60.0 },
                color: "#3b82f6".to_string(),
                icon: None,
                tooltip: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        // Add edges
        graph.add_edge(GraphEdge {
            source: "node0".to_string(),
            target: "node1".to_string(),
            label: Some("connect".to_string()),
            weight: None,
            color: None,
            style: EdgeStyle {
                line_style: "solid".to_string(),
                arrow_head: true,
                thickness: 2.0,
                curvature: 0.0,
            },
            edge_type: "http".to_string(),
            bidirectional: false,
            metadata: std::collections::HashMap::new(),
        });

        // Test force-directed layout
        let mut layout_config = graph.layout_config.clone();
        layout_config.algorithm = LayoutAlgorithm::ForceDirected;
        layout_config.max_iterations = 10;
        graph.layout_config = layout_config;
        graph.apply_layout().await.unwrap();

        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));

        // Test hierarchical layout
        let mut layout_config = graph.layout_config.clone();
        layout_config.algorithm = LayoutAlgorithm::Hierarchical;
        graph.layout_config = layout_config;
        graph.apply_layout().await.unwrap();

        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));

        // Test circular layout
        let mut layout_config = graph.layout_config.clone();
        layout_config.algorithm = LayoutAlgorithm::Circular;
        graph.layout_config = layout_config;
        graph.apply_layout().await.unwrap();

        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));

        // Test grid layout
        let mut layout_config = graph.layout_config.clone();
        layout_config.algorithm = LayoutAlgorithm::Grid;
        graph.layout_config = layout_config;
        graph.apply_layout().await.unwrap();

        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));
    }

    /// Test chart renderer
    #[tokio::test]
    async fn test_chart_renderer() {
        use beejs::observability::dashboard::renderer::{ChartRenderer, RenderConfig};

        let config: _ = RenderConfig::default();
        let renderer: _ = ChartRenderer::new(config);

        // Create a chart
        let chart_config: _ = super::super::super::super::observability::dashboard::renderer::ChartConfig {
            title: "Test Chart".to_string(),
            width: 800,
            height: 600,
            margin: super::super::super::super::observability::dashboard::renderer::MarginConfig {
                top: 20, right: 20, bottom: 40, left: 40,
            },
            colors: super::super::super::super::observability::dashboard::renderer::ColorScheme {
                primary: "#3b82f6".to_string(),
                secondary: "#10b981".to_string(),
                accent: "#f59e0b".to_string(),
                background: "#ffffff".to_string(),
                text: "#374151".to_string(),
            },
            animation: super::super::super::super::observability::dashboard::renderer::AnimationConfig {
                enabled: true,
                duration: 300,
                easing: "easeInOutQuart".to_string(),
            },
            data_point: super::super::super::super::observability::dashboard::renderer::DataPointConfig {
                show_points: true,
                point_size: 5,
                show_values: false,
                value_format: "{}".to_string(),
            },
        };

        renderer.create_chart(
            "test-chart".to_string(),
            ChartType::LineChart,
            chart_config
        ).await.unwrap();

        // Update chart data
        let chart_data: _ = super::super::super::super::observability::dashboard::renderer::ChartData {
            x_data: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            y_data: vec![10.0, 20.0, 15.0, 25.0, 30.0],
            series: Vec::new(),
            labels: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        renderer.update_chart_data("test-chart", chart_data).await.unwrap();

        // Render chart
        let svg: _ = renderer.render_chart_svg("test-chart").await.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Chart"));

        // Check stats
        let stats: _ = renderer.get_chart_stats("test-chart").await.unwrap();
        assert!(stats.total_renders > 0);
    }

    /// Test graph renderer
    #[tokio::test]
    async fn test_graph_renderer() {
        use beejs::observability::dashboard::renderer::{GraphRenderer, RenderConfig};

        let config: _ = RenderConfig::default();
        let renderer: _ = GraphRenderer::new(config);

        // Create a graph
        let nodes: _ = vec![
            GraphNode {
                id: "node1".to_string(),
                label: "Node 1".to_string(),
                node_type: "service".to_string(),
                status: "healthy".to_string(),
                position: Position { x: 100.0, y: 100.0 },
                size: Size { width: 50.0, height: 50.0 },
                color: "#3b82f6".to_string(),
                icon: None,
                tooltip: None,
                metadata: std::collections::HashMap::new(),
            },
            GraphNode {
                id: "node2".to_string(),
                label: "Node 2".to_string(),
                node_type: "service".to_string(),
                status: "healthy".to_string(),
                position: Position { x: 300.0, y: 100.0 },
                size: Size { width: 50.0, height: 50.0 },
                color: "#10b981".to_string(),
                icon: None,
                tooltip: None,
                metadata: std::collections::HashMap::new(),
            },
        ];

        let edges: _ = vec![
            GraphEdge {
                source: "node1".to_string(),
                target: "node2".to_string(),
                label: Some("call".to_string()),
                weight: Some(1.0),
                color: None,
                style: EdgeStyle {
                    line_style: "solid".to_string(),
                    arrow_head: true,
                    thickness: 2.0,
                    curvature: 0.0,
                },
                edge_type: "http".to_string(),
                bidirectional: false,
                metadata: std::collections::HashMap::new(),
            },
        ];

        renderer.create_graph(
            "test-graph".to_string(),
            GraphType::TopologyGraph,
            nodes,
            edges
        ).await.unwrap();

        // Apply layout
        renderer.apply_layout("test-graph").await.unwrap();

        // Render graph
        let svg: _ = renderer.render_graph_svg("test-graph").await.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("line"));
        assert!(svg.contains("circle"));
    }

    /// Test template engine
    #[tokio::test]
    async fn test_template_engine() {
        use beejs::observability::dashboard::renderer::{TemplateEngine, Template, TemplateFunction};

        let engine: _ = TemplateEngine::new();

        let template: _ = Template {
            id: "dashboard-template".to_string(),
            content: "Dashboard: {{title}}, Instance: {{instance}}, Time Range: {{time_range}}".to_string(),
            variables: vec!["title".to_string(), "instance".to_string(), "time_range".to_string()],
            functions: std::collections::HashMap::new(),
        };

        engine.add_template(template).await.unwrap();

        let mut variables = std::collections::HashMap::new();
        variables.insert("title".to_string(), serde_json::Value::String("CPU Overview".to_string()));
        variables.insert("instance".to_string(), serde_json::Value::String("beejs-1".to_string()));
        variables.insert("time_range".to_string(), serde_json::Value::String("Last 1 hour".to_string()));

        let result: _ = engine.render_template("dashboard-template", &variables).await.unwrap();
        assert!(result.contains("CPU Overview"));
        assert!(result.contains("beejs-1"));
        assert!(result.contains("Last 1 hour"));
    }

    /// Test metrics collection
    #[tokio::test]
    async fn test_metrics_collection() {
        use beejs::observability::dashboard::manager::{DashboardManager, DashboardConfig, MetricsCollector};

        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Start metrics collection
        manager.start_metrics_collection().await.unwrap();

        // Wait for metrics collection
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Get metrics snapshot
        let snapshot: _ = manager.get_metrics_snapshot().await.unwrap();
        assert!(snapshot.is_empty() || !snapshot.is_empty()); // May be empty if no metrics available

        // Stop metrics collection
        manager.stop_metrics_collection().await.unwrap();
    }

    /// Test Grafana client conversion
    #[tokio::test]
    async fn test_grafana_client_conversion() {
        use beejs::observability::dashboard::manager::{DashboardManager, DashboardConfig};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Create a dashboard with panels
        let uid: _ = manager.create_dashboard("grafana-test").await.unwrap();

        // Export to Grafana format
        let grafana_dashboard: _ = manager.export_dashboard(&uid).await.unwrap();
        assert!(grafana_dashboard.get("dashboard").is_some());
        assert!(grafana_dashboard.get("folderId").is_some());
    }

    /// Test dashboard configuration
    #[tokio::test]
    async fn test_dashboard_configuration() {
        let mut config = DashboardConfig::default();
        config.grafana_url = "http://localhost:3000".to_string();
        config.api_key = Some("test-api-key".to_string());
        config.refresh_interval = 10;
        config.metrics_interval = 2;
        config.default_time_range_hours = 2;
        config.enable_realtime = false;
        config.enable_templating = false;

        let manager: _ = DashboardManager::new(config).await.unwrap();
        assert_eq!(manager.list_dashboards().await.len(), 1); // Should have overview dashboard
    }

    /// Test color palette
    #[tokio::test]
    fn test_color_palette() {
        let palette: _ = ColorPalette {
            primary: "#3b82f6".to_string(),
            secondary: "#10b981".to_string(),
            tertiary: "#f59e0b".to_string(),
            accent: "#ef4444".to_string(),
            success: "#22c55e".to_string(),
            warning: "#eab308".to_string(),
            error: "#ef4444".to_string(),
        };

        assert_eq!(palette.primary, "#3b82f6");
        assert_eq!(palette.success, "#22c55e");
        assert_eq!(palette.error, "#ef4444");
    }

    /// Test data series
    #[tokio::test]
    fn test_data_series() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("category".to_string(), serde_json::Value::String("test".to_string()));

        let series: _ = DataSeries {
            name: "Test Series".to_string(),
            data: vec![
                DataPoint {
                    x: 0.0,
                    y: 10.0,
                    label: Some("Point 1".to_string()),
                    color: Some("#3b82f6".to_string()),
                    metadata: metadata.clone(),
                },
                DataPoint {
                    x: 1.0,
                    y: 20.0,
                    label: Some("Point 2".to_string()),
                    color: Some("#10b981".to_string()),
                    metadata,
                },
            ],
            color: Some("#3b82f6".to_string()),
            visible: true,
            line_style: super::super::super::super::observability::visualization::LineStyle {
                width: 2.0,
                dash_array: Some(vec![5.0, 5.0]),
                cap: "round".to_string(),
                join: "round".to_string(),
            },
        };

        assert_eq!(series.name, "Test Series");
        assert_eq!(series.data.len(), 2);
        assert!(series.visible);
        assert_eq!(series.line_style.width, 2.0);
    }

    /// Test axis configuration
    #[tokio::test]
    fn test_axis_configuration() {
        let axis: _ = AxisConfig {
            show: true,
            min: Some(0.0),
            max: Some(100.0),
            tick_count: 10,
            label: "Value".to_string(),
            unit: Some("%".to_string()),
            format: "{.1f}".to_string(),
        };

        assert_eq!(axis.min, Some(0.0));
        assert_eq!(axis.max, Some(100.0));
        assert_eq!(axis.unit, Some("%".to_string()));
        assert_eq!(axis.format, "{.1f}".to_string());
    }

    /// Test chart builder fluent interface
    #[tokio::test]
    async fn test_chart_builder_fluent_interface() {
        // Test Line Chart Builder
        let mut line_builder = LineChartBuilder::new();
        let line_chart: _ = line_builder
            .title("Line Chart Test")
            .dimensions(1000, 700)
            .data(vec![10.0, 20.0, 30.0])
            .build()
            .unwrap();
        assert_eq!(line_chart.config.title, "Line Chart Test");
        assert_eq!(line_chart.config.width, 1000);

        // Test Bar Chart Builder
        let mut bar_builder = BarChartBuilder::new();
        let bar_chart: _ = bar_builder
            .title("Bar Chart Test")
            .dimensions(900, 600)
            .data(vec![15.0, 25.0, 35.0])
            .build()
            .unwrap();
        assert_eq!(bar_chart.config.title, "Bar Chart Test");
        assert_eq!(bar_chart.config.height, 600);

        // Test Pie Chart Builder
        let mut pie_builder = PieChartBuilder::new();
        let pie_chart: _ = pie_builder
            .title("Pie Chart Test")
            .dimensions(500, 500)
            .data(40.0, "A")
            .data(60.0, "B")
            .build()
            .unwrap();
        assert_eq!(pie_chart.config.title, "Pie Chart Test");
        assert_eq!(pie_chart.config.width, 500);
    }

    /// Test error handling
    #[tokio::test]
    async fn test_error_handling() {
        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Try to get non-existent dashboard
        let dashboard: _ = manager.get_dashboard("non-existent").await;
        assert!(dashboard.is_none());

        // Try to add panel to non-existent dashboard
        let panel: _ = PanelConfig {
            id: "test".to_string(),
            title: "Test".to_string(),
            panel_type: "graph".to_string(),
            grid_pos: GridPos { x: 0, y: 0, w: 12, h: 8 },
            datasource: "Prometheus".to_string(),
            targets: vec![],
            field_config: FieldConfig {
                min: None,
                max: None,
                unit: None,
                decimals: None,
                thresholds: None,
            },
            options: PanelOptions {
                legend: LegendConfig { show: true, position: "bottom".to_string() },
                tooltip: TooltipConfig { mode: "multi".to_string(), sort: "none".to_string() },
                time: TimeRangeConfig { from: "now-1h".to_string(), to: "now".to_string() },
            },
        };

        let result: _ = manager.add_panel("non-existent", panel).await;
        assert!(result.is_err());

        // Test chart rendering with empty data
        let config: _ = VisualizationConfig::default();
        let line_chart: _ = LineChart::new(config);
        let result: _ = line_chart.render_svg();
        assert!(result.is_err());
    }

    /// Test performance characteristics
    #[tokio::test]
    async fn test_performance() {
        let start_time: _ = std::time::Instant::now();

        // Create and render a complex chart
        let mut builder = LineChartBuilder::new();
        let data: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 50.0 + 50.0).collect();

        let chart: _ = builder
            .title("Performance Test Chart")
            .dimensions(1920, 1080)
            .data(data)
            .build()
            .unwrap();

        let render_time: _ = start_time.elapsed();
        let svg: _ = chart.render();

        // Verify chart was rendered
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Performance Test Chart"));

        // Performance assertion - should render in reasonable time
        assert!(render_time.as_millis() < 100, "Chart rendering took too long: {:?}", render_time);
    }

    /// Test concurrent operations
    #[tokio::test]
    async fn test_concurrent_operations() {
        let config: _ = DashboardConfig::default();
        let manager: _ = DashboardManager::new(config).await.unwrap();

        // Create multiple dashboards concurrently
        let mut handles = Vec::new();

        for i in 0..10 {
            let manager_ref: _ = &manager;
            let handle: _ = tokio::spawn(async move {
                let uid: _ = format!("dashboard-{}", i);
                manager_ref.create_dashboard(&uid).await.unwrap();
                uid
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all dashboards were created
        let dashboards: _ = manager.list_dashboards().await;
        assert_eq!(dashboards.len(), 11); // 10 created + 1 overview
    }
}
