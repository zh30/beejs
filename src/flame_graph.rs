//! 火焰图分析模块
//! 用于可视化代码执行路径和热点分析
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
/// 火焰图结构体
pub struct FlameGraph {
    root: FrameNode,
    frame_count: usize,
}
/// 栈帧结构
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file_path: String,
    pub line_number: u32,
    pub duration: Duration,
}
/// 帧节点结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameNode {
    pub function_name: String,
    pub file_path: String,
    pub line_number: u32,
    pub total_duration: Duration,
    pub call_count: u64,
    pub children: HashMap<String, FrameNode>,
}
impl FrameNode {
    /// 创建新的帧节点
    pub fn new(function_name: String, line_number: u32) -> Self {
        Self {
            function_name,
            file_path: String::new(),
            line_number,
            total_duration: Duration::from_millis(0),
            call_count: 0,
            children: HashMap::new(),
        }
    }
    /// 创建新的帧节点（带文件路径）
    pub fn new_with_path(function_name: String, file_path: String, line_number: u32) -> Self {
        Self {
            function_name,
            file_path,
            line_number,
            total_duration: Duration::from_millis(0),
            call_count: 0,
            children: HashMap::new(),
        }
    }
    /// 添加调用栈（一个完整的调用路径）
    pub fn add_call_stack(&mut self, stack: &[StackFrame]) {
        if stack.is_empty() {
            return;
        }
        // Add the first frame
        self.add_frame_to_tree(stack, 0);
    }
    /// 递归添加帧到树结构
    fn add_frame_to_tree(&mut self, stack: &[StackFrame], index: usize) {
        if index >= stack.len() {
            return;
        }
        let frame: _ = &stack[index];
        if self.function_name == "root" {
            // For root node, always create children based on function name
            let child: _ = self.children.entry(frame.function_name.clone()).or_insert_with(|| {
                FrameNode::new_with_path(
                    frame.function_name.clone(),
                    frame.file_path.clone(),
                    frame.line_number,
                )
            });
            child.total_duration += frame.duration;
            child.call_count += 1;
            // Continue with next frame if exists
            if index + 1 < stack.len() {
                child.add_frame_to_tree(stack, index + 1);
            }
        } else {
            // Update current node if it matches the frame
            if self.function_name == frame.function_name && self.line_number == frame.line_number {
                self.total_duration += frame.duration;
                self.call_count += 1;
            }
            // Add child (always add, regardless of match)
            let child: _ = self.children.entry(frame.function_name.clone()).or_insert_with(|| {
                FrameNode::new_with_path(
                    frame.function_name.clone(),
                    frame.file_path.clone(),
                    frame.line_number,
                )
            });
            child.add_frame_to_tree(stack, index + 1);
        }
    }
    /// 计算最大深度
    pub fn calculate_max_depth(&self) -> usize {
        if self.children.is_empty() {
            return 1;
        }
        let mut max_child_depth = 0;
        for child in self.children.values() {
            let child_depth: _ = child.calculate_max_depth();
            if child_depth > max_child_depth {
                max_child_depth = child_depth;
            }
        }
        1 + max_child_depth
    }
}
impl FlameGraph {
    /// 创建新的火焰图
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            root: FrameNode::new("root".to_string(), 0),
            frame_count: 0,
        })
    }
    /// 添加调用栈
    pub fn add_call_stack(&mut self, stack: &[StackFrame]) {
        self.root.add_call_stack(stack);
        self.frame_count += stack.len();
    }
    /// 添加单个栈帧（为了向后兼容）
    pub fn add_frame(&mut self, frame: StackFrame) -> Result<(), String> {
        self.add_call_stack(&[frame]);
        Ok(())
    }
    /// 获取帧数量
    pub fn get_frame_count(&self) -> usize {
        self.frame_count
    }
    /// 获取最大深度
    pub fn get_max_depth(&self) -> usize {
        // Subtract 1 for the root node itself
        // The root is just a container, so depth is the maximum depth of actual frames
        self.root.calculate_max_depth().saturating_sub(1)
    }
    /// 合并重复帧
    pub fn merge_duplicate_frames(&mut self) {
        // Use unsafe to get around borrow checker for this specific case
        // We need to temporarily take ownership of root to count after merging
        let root_ptr: _ = &mut self.root as *mut FrameNode;
        unsafe {
            let root: _ = &mut *root_ptr;
            self.merge_node_recursive(root);
        }
        // After merging, recount frames (root doesn't count as a frame)
        self.frame_count = Self::count_node_recursive(&self.root).saturating_sub(1);
    }
    /// 递归合并重复帧
    fn merge_node_recursive(&mut self, node: &mut FrameNode) {
        // Collect all children first to avoid borrow issues
        let children: Vec<(String, FrameNode)> = node.children.drain().collect();
        // Merge children with the same function name and line number
        let mut merged_children: HashMap<String, FrameNode> = HashMap::new();
        for (_, mut child) in children {
            // Recursively merge this child's children
            self.merge_node_recursive(&mut child);
            // Create a key based on function name and line number
            let key: _ = format!("{}:{}, child.function_name", child.line_number));
            // If we already have a child with this key, merge them
            if let Some(existing) = merged_children.get_mut(&key) {
                existing.total_duration += child.total_duration;
                existing.call_count += child.call_count;
                // Merge children
                for (child_key, grandchild) in child.children {
                    if let Some(existing_child) = existing.children.get_mut(&child_key) {
                        existing_child.total_duration += grandchild.total_duration;
                        existing_child.call_count += grandchild.call_count;
                    } else {
                        existing.children.insert(child_key, grandchild);
                    }
                }
            } else {
                merged_children.insert(key, child);
            }
        }
        node.children = merged_children;
    }
    /// 递归计算帧数量
    fn count_node_recursive(node: &FrameNode) -> usize {
        let mut count = 1; // Count this node
        for child in node.children.values() {
            count += Self::count_node_recursive(child);
        }
        count
    }
    /// 查找热点路径
    pub fn find_hot_paths(&self, min_duration_ms: u64) -> Vec<FrameNode> {
        let mut hot_paths = Vec::new();
        self.find_hot_paths_recursive(&self.root, &mut hot_paths, min_duration_ms);
        hot_paths.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        hot_paths
    }
    /// 递归查找热点路径
    fn find_hot_paths_recursive(&self, node: &FrameNode, hot_paths: &mut Vec<FrameNode>, min_duration_ms: u64) {
        if node.total_duration.as_millis() as u64 >= min_duration_ms && node.function_name != "root" {
            hot_paths.push(node.clone());
        }
        for child in node.children.values() {
            self.find_hot_paths_recursive(child, hot_paths, min_duration_ms);
        }
    }
    /// 生成 SVG 火焰图
    pub fn generate_svg(&self) -> Result<String, String> {
        let mut svg = String::new();
        svg.push_str("<svg width=\"800\" height=\"400\" xmlns=\"http://www.w3.org/2000/svg\">\n");
        svg.push_str("  <style>\n");
        svg.push_str("    .frame { stroke: #000; stroke-width: 1; }\n");
        svg.push_str("    .label { font-family: Arial; font-size: 10px; fill: #fff; }\n");
        svg.push_str("  </style>\n");
        svg.push_str("  <title>Flamegraph</title>\n");
        self.render_node_svg(&self.root, &mut svg, 0.0, 0.0, 800.0, 400.0);
        svg.push_str("</svg>");
        Ok(svg)
    }
    /// 递归渲染节点到 SVG
    fn render_node_svg(&self, node: &FrameNode, svg: &mut String, x: f64, y: f64, width: f64, height: f64) {
        if node.function_name == "root" {
            // Render children
            let child_height: _ = height / node.children.len() as f64;
            let mut child_y = y;
            for child in node.children.values() {
                self.render_node_svg(child, svg, x, child_y, width, child_height);
                child_y += child_height;
            }
        } else {
            // Calculate width based on duration
            let max_duration: _ = self.get_max_duration(&self.root);
            let node_width: _ = if max_duration.as_millis() > 0 {
                (node.total_duration.as_millis() as f64 / max_duration.as_millis() as f64) * width
            } else {
                width
            };
            // Random color based on function name
            let color: _ = self.get_color_for_function(&node.function_name);
            // Draw rectangle
            svg.push_str(&format!(
                "  <rect class=\"frame\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>\n",
                x, y, node_width, height, color
            ));
            // Add label if there's enough space
            if node_width > 50.0 && height > 15.0 {
                svg.push_str(&format!(
                    "  <text class=\"label\" x=\"{}\" y=\"{}\">{} ({}ms, {} calls)</text>\n",
                    x + 5.0,
                    y + height / 2.0,
                    node.function_name,
                    node.total_duration.as_millis(),
                    node.call_count
                ));
            }
        }
    }
    /// 获取最大持续时间
    fn get_max_duration(&self, node: &FrameNode) -> Duration {
        let mut max_duration = node.total_duration;
        for child in node.children.values() {
            let child_max: _ = self.get_max_duration(child);
            if child_max > max_duration {
                max_duration = child_max;
            }
        }
        max_duration
    }
    /// 为函数生成颜色
    fn get_color_for_function(&self, function_name: &str) -> String {
        // Simple hash-based color generation
        let mut hash: i32 = 0;
        for c in function_name.chars() {
            hash = hash.wrapping_mul(31).wrapping_add(c as i32);
        }
        let hue: _ = (hash % 360) as f64;
        format!("hsl({}, 70%, 50%)", hue)
    }
    /// 导出 JSON
    pub fn export_json(&self) -> Result<String, String> {
        let json: _ = serde_json::to_string_pretty(&self.root)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;
        Ok(json)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_flame_graph_creation() {
        let flame_graph: _ = FlameGraph::new();
        assert!(flame_graph.is_ok());
        let flame_graph: _ = flame_graph.unwrap();
        assert!(flame_graph.get_frame_count() == 0);
    }
    #[test]
    fn test_add_stack_frame() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        let frame: _ = StackFrame {
            function_name: "test_function".to_string(),
            file_path: "test.js".to_string(),
            line_number: 42,
            duration: Duration::from_millis(10),
        };
        let mut flame_graph = flame_graph;
        flame_graph.add_frame(frame).unwrap();
        assert!(flame_graph.get_frame_count() == 1);
    }
    #[test]
    fn test_nested_stack_frames() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        // Add nested frames as a call stack
        let stack: _ = vec![
            StackFrame {
                function_name: "main".to_string(),
                file_path: "main.js".to_string(),
                line_number: 1,
                duration: Duration::from_millis(100),
            },
            StackFrame {
                function_name: "processData".to_string(),
                file_path: "main.js".to_string(),
                line_number: 10,
                duration: Duration::from_millis(50),
            },
            StackFrame {
                function_name: "transform".to_string(),
                file_path: "main.js".to_string(),
                line_number: 20,
                duration: Duration::from_millis(30),
            },
        ];
        let mut flame_graph = flame_graph;
        flame_graph.add_call_stack(&stack);
        assert!(flame_graph.get_frame_count() == 3);
        assert_eq!(flame_graph.get_max_depth(), 3); // Actual depth is 3 for 3 frames
    }
    #[test]
    fn test_frame_node_creation() {
        let node: _ = FrameNode::new("test_function".to_string(), 42);
        assert_eq!(node.function_name, "test_function");
        assert_eq!(node.line_number, 42);
        assert_eq!(node.total_duration, Duration::from_millis(0));
        assert_eq!(node.call_count, 0);
        assert!(node.children.is_empty());
    }
    #[test]
    fn test_svg_generation() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        let frame: _ = StackFrame {
            function_name: "hot_function".to_string(),
            file_path: "benchmark.js".to_string(),
            line_number: 100,
            duration: Duration::from_millis(25),
        };
        let mut flame_graph = flame_graph;
        flame_graph.add_frame(frame).unwrap();
        let svg: _ = flame_graph.generate_svg();
        assert!(svg.is_ok());
        let svg_content: _ = svg.unwrap();
        assert!(svg_content.contains("<svg"));
        assert!(svg_content.contains("hot_function"));
    }
    #[test]
    fn test_frame_merging() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        // Add same frame twice
        let stack1: _ = vec![StackFrame {
            function_name: "merge_test".to_string(),
            file_path: "test.js".to_string(),
            line_number: 5,
            duration: Duration::from_millis(10),
        }];
        let stack2: _ = vec![StackFrame {
            function_name: "merge_test".to_string(),
            file_path: "test.js".to_string(),
            line_number: 5,
            duration: Duration::from_millis(15),
        }];
        let mut flame_graph = flame_graph;
        flame_graph.add_call_stack(&stack1);
        flame_graph.add_call_stack(&stack2);
        flame_graph.merge_duplicate_frames();
        assert_eq!(flame_graph.get_frame_count(), 1); // Should merge to 1 unique frame
    }
    #[test]
    fn test_hot_path_detection() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        // Add multiple frames with different durations
        let frames: _ = vec![
            StackFrame {
                function_name: "cold_function".to_string(),
                file_path: "test.js".to_string(),
                line_number: 1,
                duration: Duration::from_millis(1),
            },
            StackFrame {
                function_name: "hot_function".to_string(),
                file_path: "test.js".to_string(),
                line_number: 10,
                duration: Duration::from_millis(50),
            },
            StackFrame {
                function_name: "hotter_function".to_string(),
                file_path: "test.js".to_string(),
                line_number: 20,
                duration: Duration::from_millis(100),
            },
        ];
        let mut flame_graph = flame_graph;
        for frame in frames {
            flame_graph.add_call_stack(&[frame]);
        }
        let hot_paths: _ = flame_graph.find_hot_paths(2);
        assert!(hot_paths.len() > 0);
        // The hottest function should be first
        assert!(hot_paths[0].function_name.contains("hotter"));
    }
    #[test]
    fn test_depth_calculation() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        // Add frames at different depths
        let stack: _ = vec![
            StackFrame {
                function_name: "level0".to_string(),
                file_path: "test.js".to_string(),
                line_number: 1,
                duration: Duration::from_millis(10),
            },
            StackFrame {
                function_name: "level1".to_string(),
                file_path: "test.js".to_string(),
                line_number: 2,
                duration: Duration::from_millis(10),
            },
            StackFrame {
                function_name: "level2".to_string(),
                file_path: "test.js".to_string(),
                line_number: 3,
                duration: Duration::from_millis(10),
            },
            StackFrame {
                function_name: "level3".to_string(),
                file_path: "test.js".to_string(),
                line_number: 4,
                duration: Duration::from_millis(10),
            },
        ];
        let mut flame_graph = flame_graph;
        flame_graph.add_call_stack(&stack);
        assert_eq!(flame_graph.get_max_depth(), 4); // Actual depth is 4 for 4 frames
    }
    #[test]
    fn test_export_json() {
        let flame_graph: _ = FlameGraph::new().unwrap();
        let frame: _ = StackFrame {
            function_name: "json_test".to_string(),
            file_path: "export.js".to_string(),
            line_number: 15,
            duration: Duration::from_millis(20),
        };
        let mut flame_graph = flame_graph;
        flame_graph.add_call_stack(&[frame]);
        let json: _ = flame_graph.export_json();
        assert!(json.is_ok());
        let json_str: _ = json.unwrap();
        assert!(json_str.contains("json_test"));
        assert!(json_str.contains("export.js"));
    }
}