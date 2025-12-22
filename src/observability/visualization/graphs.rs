//! Graph Components - Advanced graph visualization
//!
//! This module provides specialized graph components for system architecture and relationship visualization:
//! - TopologyGraph: System topology and service mesh visualization
//! - DependencyGraph: Module dependency and import relationships
//! - TraceGraph: Request flow and distributed tracing visualization
//! - NetworkGraph: Network topology and connectivity visualization
use super::*;
use anyhow::{Result, Context, anyhow};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, warn, error};
/// Graph node with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node ID (unique)
    pub id: String,
    /// Node label
    pub label: String,
    /// Node type (e.g., "service", "database", "api")
    pub node_type: String,
    /// Node status (e.g., "healthy", "warning", "error")
    pub status: String,
    /// Position coordinates
    pub position: Position,
    /// Node size
    pub size: Size,
    /// Node color
    pub color: String,
    /// Icon or symbol
    pub icon: Option<String>,
    /// Tooltip text
    pub tooltip: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
/// Graph edge with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Edge label
    pub label: Option<String>,
    /// Edge weight (for algorithm calculations)
    pub weight: Option<f64>,
    /// Edge color
    pub color: Option<String>,
    /// Edge style
    pub style: EdgeStyle,
    /// Edge type (e.g., "http", "grpc", "database")
    pub edge_type: String,
    /// Bidirectional edge
    pub bidirectional: bool,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
/// Position in 2D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
/// Size dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}
/// Edge style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub line_style: String, // "solid", "dashed", "dotted"
    pub arrow_head: bool,
    pub thickness: f64,
    pub curvature: f64,
}
/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Layout algorithm
    pub algorithm: LayoutAlgorithm,
    /// Node spacing
    pub node_spacing: f64,
    /// Level spacing (for hierarchical layouts)
    pub level_spacing: f64,
    /// Maximum iterations
    pub max_iterations: u32,
    /// Force-directed parameters
    pub force_params: ForceLayoutParams,
}
/// Layout algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    ForceDirected,
    Hierarchical,
    Circular,
    Grid,
    Tree,
    Radial,
}
/// Force-directed layout parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceLayoutParams {
    pub repulsion: f64,
    pub attraction: f64,
    pub damping: f64,
    pub center_force: f64,
}
/// Interaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionConfig {
    pub drag_enabled: bool,
    pub zoom_enabled: bool,
    pub pan_enabled: bool,
    pub select_enabled: bool,
    pub hover_enabled: bool,
    pub click_enabled: bool,
}
/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub node_types: HashSet<String>,
    pub edge_types: HashSet<String>,
    pub status_filter: HashSet<String>,
    pub min_weight: Option<f64>,
    pub max_weight: Option<f64>,
}
/// TopologyGraph - For system topology visualization
pub struct TopologyGraph {
    config: VisualizationConfig,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    layout_config: LayoutConfig,
    interaction_config: InteractionConfig,
    filter_config: FilterConfig,
}
impl TopologyGraph {
    /// Create a new topology graph
    pub fn new(config: VisualizationConfig) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            layout_config: LayoutConfig {
                algorithm: LayoutAlgorithm::ForceDirected,
                node_spacing: 100.0,
                level_spacing: 150.0,
                max_iterations: 100,
                force_params: ForceLayoutParams {
                    repulsion: 800.0,
                    attraction: 0.1,
                    damping: 0.9,
                    center_force: 0.1,
                },
            },
            interaction_config: InteractionConfig {
                drag_enabled: true,
                zoom_enabled: true,
                pan_enabled: true,
                select_enabled: true,
                hover_enabled: true,
                click_enabled: true,
            },
            filter_config: FilterConfig {
                node_types: HashSet::new(),
                edge_types: HashSet::new(),
                status_filter: HashSet::new(),
                min_weight: None,
                max_weight: None,
            },
            config,
        }
    }
    /// Add node to graph
    pub fn add_node(&mut self, node: GraphNode) -> &mut Self {
        self.nodes.push(node);
        self
    }
    /// Add edge to graph
    pub fn add_edge(&mut self, edge: GraphEdge) -> &mut Self {
        self.edges.push(edge);
        self
    }
    /// Set layout configuration
    pub fn layout_config(&mut self, config: LayoutConfig) -> &mut Self {
        self.layout_config = config;
        self
    }
    /// Set interaction configuration
    pub fn interaction_config(&mut self, config: InteractionConfig) -> &mut Self {
        self.interaction_config = config;
        self
    }
    /// Apply layout algorithm
    pub async fn apply_layout(&mut self) -> Result<()> {
        debug!("Applying layout algorithm: {:?}", self.layout_config.algorithm);
        match self.layout_config.algorithm {
            LayoutAlgorithm::ForceDirected => {
                self.apply_force_directed_layout().await?;
            }
            LayoutAlgorithm::Hierarchical => {
                self.apply_hierarchical_layout().await?;
            }
            LayoutAlgorithm::Circular => {
                self.apply_circular_layout().await?;
            }
            LayoutAlgorithm::Grid => {
                self.apply_grid_layout().await?;
            }
            LayoutAlgorithm::Tree => {
                self.apply_tree_layout().await?;
            }
            LayoutAlgorithm::Radial => {
                self.apply_radial_layout().await?;
            }
        }
        Ok(())
    }
    /// Render the graph as SVG
    pub fn render_svg(&self) -> Result<String> {
        debug!("Rendering topology graph: {} nodes, {} edges", self.nodes.len(), self.edges.len());
        let width: _ = self.config.width;
        let height: _ = self.config.height;
        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n  <defs>\n    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n      <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#666\" />\n    </marker>\n  </defs>\n  <style>\n    .node {{ cursor: pointer; }}\n    .node-label {{ font-family: {}; font-size: {}px; fill: {}; text-anchor: middle; }}\n    .edge {{ stroke: #999; stroke-width: 2; fill: none; }}\n    .edge-label {{ font-family: {}; font-size: {}px; fill: {}; }}\n  </style>\n  <rect width=\"100%\" height=\"100%\" fill=\"{}\"/>\n",
            width, height,
            self.config.font_family,
            self.config.font_size,
            self.config.text_color,
            self.config.font_family,
            self.config.font_size - 2,
            self.config.text_color,
            self.config.background_color
        ));
        // Render edges first (so they appear behind nodes)
        for edge in &self.edges {
            if let (Some(source), Some(target)) = (
                self.nodes.iter().find(|n| n.id == edge.source),
                self.nodes.iter().find(|n| n.id == edge.target)
            ) {
                let color: _ = edge.color.as_deref().unwrap_or("#999999");
                let thickness: _ = edge.style.thickness;
                // Draw edge line
                svg.push_str(&format!(
                    r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="edge" stroke="{}" stroke-width="{}"/>
"#,
                    source.position.x, source.position.y,
                    target.position.x, target.position.y,
                    color, thickness
                ));
                // Draw arrowhead
                if edge.style.arrow_head {
                    let dx: _ = target.position.x - source.position.x;
                    let dy: _ = target.position.y - source.position.y;
                    let angle: _ = dy.atan2(dx);
                    let arrow_x: _ = target.position.x - 15.0 * angle.cos();
                    let arrow_y: _ = target.position.y - 15.0 * angle.sin();
                    svg.push_str(&format!(
                        r#"  <polygon points="{},{} {},{} {},{}" fill="{}"/>
"#,
                        target.position.x, target.position.y,
                        arrow_x - 8.0 * (angle + 0.3).cos(), arrow_y - 8.0 * (angle + 0.3).sin(),
                        arrow_x - 8.0 * (angle - 0.3).cos(), arrow_y - 8.0 * (angle - 0.3).sin(),
                        color
                    ));
                }
                // Draw edge label
                if let Some(ref label) = edge.label {
                    let mid_x: _ = (source.position.x + target.position.x) / 2.0;
                    let mid_y: _ = (source.position.y + target.position.y) / 2.0;
                    svg.push_str(&format!(
                        r#"  <text x="{}" y="{}" class="edge-label" text-anchor="middle">{}</text>
"#,
                        mid_x, mid_y - 5, label
                    ));
                }
            }
        }
        // Render nodes
        for node in &self.nodes {
            let node_color: _ = match node.status.as_str() {
                "healthy" => "#22c55e",
                "warning" => "#eab308",
                "error" => "#ef4444",
                "unknown" => "#6b7280",
                _ => &node.color,
            };
            // Draw node shape (rounded rectangle)
            let x: _ = node.position.x - node.size.width / 2.0;
            let y: _ = node.position.y - node.size.height / 2.0;
            svg.push_str(&format!(
                "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"8\" ry=\"8\" fill=\"{}\" stroke=\"#333\" stroke-width=\"2\" class=\"node\"/>\n",
                x, y, node.size.width, node.size.height, node_color
            ));
            // Draw node icon or label
            if let Some(ref icon) = node.icon {
                svg.push_str(&format!(
                    r#"  <text x="{}" y="{}" class="node-label" font-size="24">{}</text>
"#,
                    node.position.x, node.position.y + 8, icon
                ));
            } else {
                svg.push_str(&format!(
                    r#"  <text x="{}" y="{}" class="node-label">{}</text>
"#,
                    node.position.x, node.position.y + 5, node.label
                ));
            }
            // Draw node type indicator
            svg.push_str(&format!(
                r#"  <text x="{}" y="{}" class="node-label" font-size="10" opacity="0.7">{}</text>
"#,
                node.position.x, node.position.y + node.size.height / 2.0 + 15, node.node_type
            ));
        }
        svg.push_str("</svg>");
        debug!("Topology graph rendered successfully: {} bytes", svg.len());
        Ok(svg)
    }
    /// Apply force-directed layout
    async fn apply_force_directed_layout(&mut self) -> Result<()> {
        // Simplified force-directed layout algorithm
        // In a production environment, this would use more sophisticated physics simulation
        let center_x: _ = self.config.width as f64 / 2.0;
        let center_y: _ = self.config.height as f64 / 2.0;
        // Initialize positions if not set
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.position.x == 0.0 && node.position.y == 0.0 {
                let angle: _ = (i as f64 / self.nodes.len() as f64) * std::f64::consts::PI * 2.0;
                let radius: _ = self.layout_config.node_spacing * (1.0 + (i as f64 / 10.0));
                node.position.x = center_x + radius * angle.cos();
                node.position.y = center_y + radius * angle.sin();
            }
        }
        // Apply repulsive forces between nodes
        for _ in 0..self.layout_config.max_iterations {
            for i in 0..self.nodes.len() {
                for j in (i + 1)..self.nodes.len() {
                    let dx: _ = self.nodes[j].position.x - self.nodes[i].position.x;
                    let dy: _ = self.nodes[j].position.y - self.nodes[i].position.y;
                    let distance: _ = (dx * dx + dy * dy).sqrt();
                    if distance > 0.0 {
                        let force: _ = self.layout_config.force_params.repulsion / (distance * distance);
                        let fx: _ = force * dx / distance;
                        let fy: _ = force * dy / distance;
                        self.nodes[i].position.x -= fx;
                        self.nodes[i].position.y -= fy;
                        self.nodes[j].position.x += fx;
                        self.nodes[j].position.y += fy;
                    }
                }
            }
            // Apply attractive forces along edges
            for edge in &self.edges {
                if let (Some(source_idx), Some(target_idx)) = (
                    self.nodes.iter().position(|n| n.id == edge.source),
                    self.nodes.iter().position(|n| n.id == edge.target)
                ) {
                    let dx: _ = self.nodes[target_idx].position.x - self.nodes[source_idx].position.x;
                    let dy: _ = self.nodes[target_idx].position.y - self.nodes[source_idx].position.y;
                    let distance: _ = (dx * dx + dy * dy).sqrt();
                    if distance > 0.0 {
                        let force: _ = distance * self.layout_config.force_params.attraction;
                        let fx: _ = force * dx / distance;
                        let fy: _ = force * dy / distance;
                        self.nodes[source_idx].position.x += fx * self.layout_config.force_params.damping;
                        self.nodes[source_idx].position.y += fy * self.layout_config.force_params.damping;
                        self.nodes[target_idx].position.x -= fx * self.layout_config.force_params.damping;
                        self.nodes[target_idx].position.y -= fy * self.layout_config.force_params.damping;
                    }
                }
            }
        }
        Ok(())
    }
    /// Apply hierarchical layout
    async fn apply_hierarchical_layout(&mut self) -> Result<()> {
        // Simple hierarchical layout based on graph structure
        let mut levels: HashMap<String, Vec<usize>> = HashMap::new();
        let mut level_count = 0;
        // Find root nodes (nodes with no incoming edges)
        for (i, node) in self.nodes.iter().enumerate() {
            let has_incoming: _ = self.edges.iter().any(|e| e.target == node.id);
            if !has_incoming {
                levels.entry("level_0".to_string()).or_insert_with(Vec::new).push(i);
            }
        }
        // Assign other nodes to levels
        for (i, node) in self.nodes.iter().enumerate() {
            if levels.values().any(|v| v.contains(&i)) {
                continue;
            }
            let max_source_level: _ = self.edges
                .iter()
                .filter(|e| e.target == node.id)
                .filter_map(|e| {
                    self.nodes.iter().position(|n| n.id == e.source)
                })
                .filter_map(|idx| {
                    levels.iter().find(|(_, v)| v.contains(&idx))
                        .map(|(k, _)| k)
                        .max()
                        .map(|k| k.parse::<u32>().unwrap_or(0))
                        .unwrap_or(0)
                });
            let level_key: _ = format!("level_{}", max_source_level + 1);
            levels.entry(level_key).or_insert_with(Vec::new).push(i);
        }
        // Position nodes
        for (level_key, node_indices) in levels {
            let level: _ = level_key.parse::<u32>().unwrap_or(0);
            let y: _ = level as f64 * self.layout_config.level_spacing + 100.0;
            for (i, node_idx) in node_indices.iter().enumerate() {
                let x: _ = i as f64 * self.layout_config.node_spacing + 100.0;
                self.nodes[*node_idx].position.x = x;
                self.nodes[*node_idx].position.y = y;
            }
        }
        Ok(())
    }
    /// Apply circular layout
    async fn apply_circular_layout(&mut self) -> Result<()> {
        let center_x: _ = self.config.width as f64 / 2.0;
        let center_y: _ = self.config.height as f64 / 2.0;
        let radius: _ = (self.config.width.min(self.config.height) / 2.0) - 100.0;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let angle: _ = (i as f64 / self.nodes.len() as f64) * std::f64::consts::PI * 2.0;
            node.position.x = center_x + radius * angle.cos();
            node.position.y = center_y + radius * angle.sin();
        }
        Ok(())
    }
    /// Apply grid layout
    async fn apply_grid_layout(&mut self) -> Result<()> {
        let cols: _ = (self.nodes.len() as f64).sqrt().ceil() as usize;
        let spacing: _ = self.layout_config.node_spacing;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let row: _ = i / cols;
            let col: _ = i % cols;
            node.position.x = col as f64 * spacing + 100.0;
            node.position.y = row as f64 * spacing + 100.0;
        }
        Ok(())
    }
    /// Apply tree layout
    async fn apply_tree_layout(&mut self) -> Result<()> {
        // Simplified tree layout - find root and arrange in tree structure
        let roots: Vec<&GraphNode> = self.nodes.iter()
            .filter(|node| {
                !self.edges.iter().any(|e| e.target == node.id)
            })
            .collect();
        if roots.is_empty() {
            // No roots found, use first node
            if let Some(root) = self.nodes.first() {
                self.apply_circular_layout().await?;
            }
            return Ok(());
        }
        let mut visited = HashSet::new();
        let mut current_y = 100.0;
        for root in roots {
            let subtree_height: _ = self.calculate_subtree_height(root, &mut visited);
            let subtree_width: _ = self.layout_config.node_spacing * subtree_height as f64;
            self.position_subtree(root, 100.0, current_y, &mut visited, 0);
            current_y += subtree_width + self.layout_config.level_spacing;
        }
        Ok(())
    }
    /// Calculate subtree height
    fn calculate_subtree_height(&self, node: &GraphNode, visited: &mut HashSet<String>) -> usize {
        if visited.contains(&node.id) {
            return 0;
        }
        visited.insert(node.id.clone());
        let children: Vec<&GraphNode> = self.edges
            .iter()
            .filter(|e| e.source == node.id)
            .filter_map(|e| self.nodes.iter().find(|n| n.id == e.target))
            .collect();
        if children.is_empty() {
            1
        } else {
            let max_child_height: _ = children.iter()
                .map(|child| self.calculate_subtree_height(child, visited))
                .max()
                .unwrap_or(0);
            1 + max_child_height
        }
    }
    /// Position subtree nodes
    fn position_subtree(
        &mut self,
        node: &GraphNode,
        x: f64,
        y: f64,
        visited: &mut HashSet<String>,
        depth: u32,
    ) {
        if visited.contains(&node.id) {
            return;
        }
        visited.insert(node.id.clone());
        // Update node position
        if let Some(node_mut) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            node_mut.position.x = x;
            node_mut.position.y = y;
        }
        let children: Vec<&GraphNode> = self.edges
            .iter()
            .filter(|e| e.source == node.id)
            .filter_map(|e| self.nodes.iter().find(|n| n.id == e.target))
            .collect();
        if !children.is_empty() {
            let child_y: _ = y + self.layout_config.level_spacing;
            let child_spacing: _ = self.layout_config.node_spacing * (children.len() as f64 + 1.0);
            for (i, child) in children.iter().enumerate() {
                let child_x: _ = x - child_spacing / 2.0 + (i as f64 + 1.0) * child_spacing;
                self.position_subtree(child, child_x, child_y, visited, depth + 1);
            }
        }
    }
    /// Apply radial layout
    async fn apply_radial_layout(&mut self) -> Result<()> {
        // Simplified radial layout - arrange nodes in concentric circles by depth
        let center_x: _ = self.config.width as f64 / 2.0;
        let center_y: _ = self.config.height as f64 / 2.0;
        // Calculate depth for each node
        let mut depths: HashMap<String, u32> = HashMap::new();
        let mut queue = VecDeque::new();
        // Start with nodes that have no incoming edges
        for node in &self.nodes {
            let has_incoming: _ = self.edges.iter().any(|e| e.target == node.id);
            if !has_incoming {
                depths.insert(node.id.clone(), 0);
                queue.push_back(node.id.clone());
            }
        }
        // BFS to calculate depths
        while let Some(node_id) = queue.pop_front() {
            let current_depth: _ = *depths.get(&node_id).unwrap_or(&0);
            for edge in &self.edges {
                if edge.source == node_id {
                    if !depths.contains_key(&edge.target) {
                        depths.insert(edge.target.clone(), current_depth + 1);
                        queue.push_back(edge.target.clone());
                    }
                }
            }
        }
        // Group nodes by depth
        let mut depth_groups: HashMap<u32, Vec<&GraphNode>> = HashMap::new();
        for node in &self.nodes {
            let depth: _ = *depths.get(&node.id).unwrap_or(&0);
            depth_groups.entry(depth).or_insert_with(Vec::new).push(node);
        }
        // Position nodes in circles
        for (depth, nodes) in depth_groups {
            let radius: _ = (depth + 1) as f64 * self.layout_config.level_spacing;
            for (i, node) in nodes.iter().enumerate() {
                let angle: _ = (i as f64 / nodes.len() as f64) * std::f64::consts::PI * 2.0;
                let x: _ = center_x + radius * angle.cos();
                let y: _ = center_y + radius * angle.sin();
                if let Some(node_mut) = self.nodes.iter_mut().find(|n| n.id == node.id) {
                    node_mut.position.x = x;
                    node_mut.position.y = y;
                }
            }
        }
        Ok(())
    }
}
impl Visualizable for TopologyGraph {
    fn render(&self) -> String {
        self.render_svg().unwrap_or_else(|e| {
            error!("Failed to render topology graph: {}", e);
            format!("<svg width=\"{}\" height=\"{}\"><text>Error: {}</text></svg>",
                    self.config.width, self.config.height, e)
        })
    }
    fn update_data(&mut self, _data: Vec<f64>) -> Result<()> {
        // Topology graphs use node/edge data, not simple arrays
        warn!("update_data not applicable to TopologyGraph - use add_node/add_edge instead");
        Ok(())
    }
    fn get_config(&self) -> &VisualizationConfig {
        &self.config
    }
}
/// Builder for TopologyGraph
pub struct TopologyGraphBuilder {
    config: VisualizationConfig,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    layout_config: LayoutConfig,
    interaction_config: InteractionConfig,
}
impl TopologyGraphBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: VisualizationConfig::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            layout_config: LayoutConfig {
                algorithm: LayoutAlgorithm::ForceDirected,
                node_spacing: 100.0,
                level_spacing: 150.0,
                max_iterations: 100,
                force_params: ForceLayoutParams {
                    repulsion: 800.0,
                    attraction: 0.1,
                    damping: 0.9,
                    center_force: 0.1,
                },
            },
            interaction_config: InteractionConfig {
                drag_enabled: true,
                zoom_enabled: true,
                pan_enabled: true,
                select_enabled: true,
                hover_enabled: true,
                click_enabled: true,
            },
        }
    }
    /// Set title
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.config.title = title.to_string();
        self
    }
    /// Set dimensions
    pub fn dimensions(&mut self, width: u32, height: u32) -> &mut Self {
        self.config.width = width;
        self.config.height = height;
        self
    }
    /// Add node
    pub fn node(
        &mut self,
        id: &str,
        label: &str,
        node_type: &str,
        status: &str,
        x: f64,
        y: f64,
    ) -> &mut Self {
        self.nodes.push(GraphNode {
            id: id.to_string(),
            label: label.to_string(),
            node_type: node_type.to_string(),
            status: status.to_string(),
            position: Position { x, y },
            size: Size { width: 80.0, height: 60.0 },
            color: "#3b82f6".to_string(),
            icon: None,
            tooltip: None,
            metadata: HashMap::new(),
        });
        self
    }
    /// Add edge
    pub fn edge(
        &mut self,
        source: &str,
        target: &str,
        label: Option<&str>,
    ) -> &mut Self {
        self.edges.push(GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            label: label.map(|s| s.to_string()),
            weight: None,
            color: None,
            style: EdgeStyle {
                line_style: "solid".to_string(),
                arrow_head: true,
                thickness: 2.0,
                curvature: 0.0,
            },
            edge_type: "default".to_string(),
            bidirectional: false,
            metadata: HashMap::new(),
        });
        self
    }
    /// Set layout algorithm
    pub fn layout(&mut self, algorithm: LayoutAlgorithm) -> &mut Self {
        self.layout_config.algorithm = algorithm;
        self
    }
    /// Build the graph
    pub fn build(&mut self) -> Result<TopologyGraph> {
        let mut graph = TopologyGraph::new(self.config.clone());
        graph.nodes = self.nodes.clone();
        graph.edges = self.edges.clone();
        graph.layout_config = self.layout_config.clone();
        graph.interaction_config = self.interaction_config.clone();
        Ok(graph)
    }
}
impl Default for TopologyGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{BTreeMap};
    #[test]
    fn test_topology_graph_builder() {
        let mut builder = TopologyGraphBuilder::new();
        builder
            .title("System Topology")
            .dimensions(1200, 800)
            .node("api", "API Gateway", "gateway", "healthy", 400.0, 200.0)
            .node("auth", "Auth Service", "service", "healthy", 200.0, 400.0)
            .node("db", "Database", "database", "warning", 600.0, 400.0)
            .edge("api", "auth", Some("HTTP"))
            .edge("api", "db", Some("SQL"));
        let graph: _ = builder.build().unwrap();
        assert_eq!(graph.config.title, "System Topology");
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }
    #[test]
    fn test_topology_graph_render() {
        let mut builder = TopologyGraphBuilder::new();
        let mut graph = builder
            .title("Test Topology")
            .node("n1", "Node 1", "service", "healthy", 100.0, 100.0)
            .node("n2", "Node 2", "service", "healthy", 300.0, 100.0)
            .edge("n1", "n2", Some("call"))
            .build()
            .unwrap();
        // Apply layout
        graph.apply_layout().unwrap();
        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Test Topology"));
        assert!(svg.contains("line"));
        assert!(svg.contains("rect"));
    }
    #[test]
    fn test_topology_graph_empty() {
        let config: _ = VisualizationConfig::default();
        let graph: _ = TopologyGraph::new(config);
        let svg: _ = graph.render();
        assert!(svg.contains("<svg"));
    }
    #[test]
    fn test_node_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), serde_json::Value::String("1.0".to_string()));
        metadata.insert("instances".to_string(), serde_json::Value::Number(serde_json::Number::from(3)));
        let node: _ = GraphNode {
            id: "test-node".to_string(),
            label: "Test Node".to_string(),
            node_type: "service".to_string(),
            status: "healthy".to_string(),
            position: Position { x: 100.0, y: 200.0 },
            size: Size { width: 80.0, height: 60.0 },
            color: "#3b82f6".to_string(),
            icon: Some("⚡".to_string()),
            tooltip: Some("Tooltip text".to_string()),
            metadata,
        };
        assert_eq!(node.id, "test-node");
        assert_eq!(node.icon, Some("⚡".to_string()));
        assert_eq!(node.metadata.get("version").unwrap(), "1.0");
    }
    #[test]
    fn test_edge_bidirectional() {
        let edge: _ = GraphEdge {
            source: "node1".to_string(),
            target: "node2".to_string(),
            label: Some("bidirectional".to_string()),
            weight: Some(1.0),
            color: None,
            style: EdgeStyle {
                line_style: "dashed".to_string(),
                arrow_head: true,
                thickness: 3.0,
                curvature: 0.5,
            },
            edge_type: "grpc".to_string(),
            bidirectional: true,
            metadata: HashMap::new(),
        };
        assert!(edge.bidirectional);
        assert_eq!(edge.style.line_style, "dashed");
        assert_eq!(edge.style.thickness, 3.0);
    }
    #[test]
    fn test_layout_algorithms() {
        let config: _ = VisualizationConfig::default();
        let mut graph = TopologyGraph::new(config);
        // Add some nodes
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
                metadata: HashMap::new(),
            });
        }
        // Test different layout algorithms
        let algorithms: _ = vec![
            LayoutAlgorithm::ForceDirected,
            LayoutAlgorithm::Hierarchical,
            LayoutAlgorithm::Circular,
            LayoutAlgorithm::Grid,
            LayoutAlgorithm::Tree,
            LayoutAlgorithm::Radial,
        ];
        for algorithm in algorithms {
            let mut layout_config = LayoutConfig {
                algorithm,
                node_spacing: 100.0,
                level_spacing: 150.0,
                max_iterations: 10,
                force_params: ForceLayoutParams {
                    repulsion: 800.0,
                    attraction: 0.1,
                    damping: 0.9,
                    center_force: 0.1,
                },
            };
            graph.layout_config = layout_config;
            graph.apply_layout().unwrap();
        }
    }
    #[test]
    fn test_circular_layout() {
        let config: _ = VisualizationConfig {
            width: 1000,
            height: 1000,
            ..Default::default()
        };
        let mut graph = TopologyGraph::new(config);
        // Add 6 nodes
        for i in 0..6 {
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
                metadata: HashMap::new(),
            });
        }
        graph.apply_circular_layout().unwrap();
        // Check that nodes are positioned in a circle
        for node in &graph.nodes {
            assert!(node.position.x != 0.0 || node.position.y != 0.0);
        }
    }
    #[test]
    fn test_grid_layout() {
        let config: _ = VisualizationConfig::default();
        let mut graph = TopologyGraph::new(config);
        // Add 4 nodes
        for i in 0..4 {
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
                metadata: HashMap::new(),
            });
        }
        graph.apply_grid_layout().unwrap();
        // Check that nodes are positioned in a grid
        let positions: Vec<(f64, f64)> = graph.nodes.iter()
            .map(|n| (n.position.x, n.position.y))
            .collect();
        // Should have 2x2 grid
        assert_eq!(positions.len(), 4);
    }
    #[test]
    fn test_filter_config() {
        let mut node_types = HashSet::new();
        node_types.insert("service".to_string());
        node_types.insert("database".to_string());
        let mut status_filter = HashSet::new();
        status_filter.insert("healthy".to_string());
        let filter_config: _ = FilterConfig {
            node_types,
            edge_types: HashSet::new(),
            status_filter,
            min_weight: Some(0.0),
            max_weight: Some(100.0),
        };
        assert!(filter_config.node_types.contains("service"));
        assert!(filter_config.status_filter.contains("healthy"));
        assert_eq!(filter_config.min_weight, Some(0.0));
    }
    #[test]
    fn test_force_layout_params() {
        let params: _ = ForceLayoutParams {
            repulsion: 1000.0,
            attraction: 0.2,
            damping: 0.95,
            center_force: 0.15,
        };
        assert_eq!(params.repulsion, 1000.0);
        assert_eq!(params.attraction, 0.2);
        assert_eq!(params.damping, 0.95);
    }
    #[test]
    fn test_edge_style() {
        let style: _ = EdgeStyle {
            line_style: "dotted".to_string(),
            arrow_head: false,
            thickness: 1.5,
            curvature: 0.3,
        };
        assert_eq!(style.line_style, "dotted");
        assert!(!style.arrow_head);
        assert_eq!(style.thickness, 1.5);
    }
    #[test]
    fn test_interaction_config() {
        let config: _ = InteractionConfig {
            drag_enabled: false,
            zoom_enabled: true,
            pan_enabled: true,
            select_enabled: false,
            hover_enabled: true,
            click_enabled: false,
        };
        assert!(!config.drag_enabled);
        assert!(config.zoom_enabled);
        assert!(config.hover_enabled);
    }
}