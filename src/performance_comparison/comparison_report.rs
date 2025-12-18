//! Comparison Report Module
//! Stage 37.0 - 性能对比报告生成
//!
//! 该模块实现性能对比报告的生成，包括：
//! - HTML 报告生成
//! - Markdown 报告生成
//! - JSON 数据导出
//! - 图表生成

use crate::performance_comparison::{ComparisonResult, PerformanceSummary};
use std::fs;
use std::path::PathBuf;

/// 报告格式
#[derive(Debug, Clone)]
pub enum ReportFormat {
    Html,
    Markdown,
    Json,
    All,
}

/// 报告配置
#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub format: ReportFormat,
    pub output_dir: PathBuf,
    pub include_charts: bool,
    pub include_raw_data: bool,
    pub template_path: Option<PathBuf>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: ReportFormat::Html,
            output_dir: PathBuf::from("performance_reports"),
            include_charts: true,
            include_raw_data: true,
            template_path: None,
        }
    }
}

/// 报告生成器
pub struct ReportGenerator {
    config: ReportConfig,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new() -> Self {
        Self {
            config: ReportConfig::default(),
        }
    }

    /// 创建带配置的报告生成器
    pub fn new_with_config(config: ReportConfig) -> Self {
        Self { config }
    }

    /// 生成完整报告
    pub fn generate_report(&self, result: &ComparisonResult) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        // 创建输出目录
        fs::create_dir_all(&self.config.output_dir)?;

        let mut generated_files = Vec::new();

        match self.config.format {
            ReportFormat::Html => {
                let file = self.generate_html_report(result)?;
                generated_files.push(file);
            }
            ReportFormat::Markdown => {
                let file = self.generate_markdown_report(result)?;
                generated_files.push(file);
            }
            ReportFormat::Json => {
                let file = self.generate_json_report(result)?;
                generated_files.push(file);
            }
            ReportFormat::All => {
                let html_file = self.generate_html_report(result)?;
                let md_file = self.generate_markdown_report(result)?;
                let json_file = self.generate_json_report(result)?;
                generated_files.push(html_file);
                generated_files.push(md_file);
                generated_files.push(json_file);
            }
        }

        Ok(generated_files)
    }

    /// 生成 HTML 报告
    fn generate_html_report(&self, result: &ComparisonResult) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("performance_report_{}.html", timestamp);
        let file_path = self.config.output_dir.join(&filename);

        let html = self.render_html_template(result)?;
        fs::write(&file_path, html)?;

        Ok(file_path)
    }

    /// 生成 Markdown 报告
    fn generate_markdown_report(&self, result: &ComparisonResult) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("performance_report_{}.md", timestamp);
        let file_path = self.config.output_dir.join(&filename);

        let markdown = self.render_markdown_template(result)?;
        fs::write(&file_path, markdown)?;

        Ok(file_path)
    }

    /// 生成 JSON 报告
    fn generate_json_report(&self, result: &ComparisonResult) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("performance_report_{}.json", timestamp);
        let file_path = self.config.output_dir.join(&filename);

        let json = serde_json::to_string_pretty(result)?;
        fs::write(&file_path, json)?;

        Ok(file_path)
    }

    /// 渲染 HTML 模板
    fn render_html_template(&self, result: &ComparisonResult) -> Result<String, Box<dyn std::error::Error>> {
        let mut html = String::new();

        // HTML 头部
        html.push_str(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Beejs Performance Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; }
        h2 { color: #34495e; margin-top: 30px; }
        h3 { color: #7f8c8d; }
        .summary {
            background: #ecf0f1;
            padding: 20px;
            border-radius: 5px;
            margin: 20px 0;
        }
        .metric {
            display: inline-block;
            margin: 10px 20px 10px 0;
        }
        .metric-value {
            font-size: 24px;
            font-weight: bold;
            color: #2c3e50;
        }
        .metric-label {
            font-size: 12px;
            color: #7f8c8d;
            text-transform: uppercase;
        }
        .test-result {
            border: 1px solid #ddd;
            padding: 15px;
            margin: 10px 0;
            border-radius: 5px;
        }
        .winner {
            background: #d5f4e6;
            border-color: #27ae60;
        }
        .chart-container {
            margin: 20px 0;
            padding: 20px;
            background: #fafafa;
            border-radius: 5px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background: #3498db;
            color: white;
        }
        .positive { color: #27ae60; font-weight: bold; }
        .negative { color: #e74c3c; font-weight: bold; }
    </style>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
    <div class="container">
"#);

        // 报告标题
        html.push_str(&format!(
            r#"        <h1>Beejs Performance Comparison Report</h1>
        <p><strong>Generated:</strong> {}</p>
        <p><strong>Environment:</strong> {} - {} - {}</p>
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            result.test_environment.os,
            result.test_environment.cpu,
            result.test_environment.memory
        ));

        // 性能摘要
        html.push_str(r#"        <div class="summary">
            <h2>Performance Summary</h2>
"#);

        html.push_str(&format!(
            r#"            <div class="metric">
                <div class="metric-value">{}</div>
                <div class="metric-label">Total Tests</div>
            </div>
            <div class="metric">
                <div class="metric-value">{:.1}%</div>
                <div class="metric-label">Beejs Win Rate</div>
            </div>
            <div class="metric">
                <div class="metric-value">{:.2}x</div>
                <div class="metric-label">Avg Speedup vs Node.js</div>
            </div>
            <div class="metric">
                <div class="metric-value">{:.2}x</div>
                <div class="metric-label">Avg Speedup vs Bun</div>
            </div>
            <div class="metric">
                <div class="metric-value">{:.1}/100</div>
                <div class="metric-label">Overall Score</div>
            </div>
"#,
            result.summary.total_tests,
            result.summary.beejs_wins as f64 / result.summary.total_tests as f64 * 100.0,
            result.summary.average_speedup_vs_nodejs,
            result.summary.average_speedup_vs_bun,
            result.summary.calculate_overall_score()
        ));

        html.push_str("        </div>");

        // 详细结果表格
        html.push_str(r#"
        <h2>Detailed Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Test Name</th>
                    <th>Winner</th>
                    <th>Beejs Avg Time</th>
                    <th>Node.js Avg Time</th>
                    <th>Bun Avg Time</th>
                    <th>Speedup vs Node.js</th>
                    <th>Speedup vs Bun</th>
                </tr>
            </thead>
            <tbody>
"#);

        for test_result in &result.test_results {
            let beejs_time = test_result
                .beejs_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let nodejs_time = test_result
                .nodejs_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let bun_time = test_result
                .bun_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let speedup_nodejs_class = if test_result.speedup_vs_nodejs > 1.0 {
                "positive"
            } else {
                "negative"
            };

            let speedup_bun_class = if test_result.speedup_vs_bun > 1.0 {
                "positive"
            } else {
                "negative"
            };

            html.push_str(&format!(
                r#"                <tr class="test-result {}">
                    <td>{}</td>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td class="{}">{:.2}x</td>
                    <td class="{}">{:.2}x</td>
                </tr>
"#,
                if test_result.winner == "beejs" { "winner" } else { "" },
                test_result.test_name,
                test_result.winner,
                beejs_time,
                nodejs_time,
                bun_time,
                speedup_nodejs_class,
                test_result.speedup_vs_nodejs,
                speedup_bun_class,
                test_result.speedup_vs_bun
            ));
        }

        html.push_str(r#"            </tbody>
        </table>
"#);

        // 如果包含图表，添加图表容器
        if self.config.include_charts {
            html.push_str(r#"
        <h2>Performance Charts</h2>
        <div class="chart-container">
            <canvas id="speedupChart" width="800" height="400"></canvas>
        </div>
        <div class="chart-container">
            <canvas id="winRateChart" width="800" height="400"></canvas>
        </div>
"#);

            // 添加图表脚本
            html.push_str(self.generate_chart_scripts(result)?);
        }

        // HTML 尾部
        html.push_str(r#"
    </div>
</body>
</html>
"#);

        Ok(html)
    }

    /// 渲染 Markdown 模板
    fn render_markdown_template(&self, result: &ComparisonResult) -> Result<String, Box<dyn std::error::Error>> {
        let mut markdown = String::new();

        markdown.push_str("# Beejs Performance Comparison Report\n\n");
        markdown.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        markdown.push_str(&format!(
            "**Environment:** {} - {} - {}\n\n",
            result.test_environment.os,
            result.test_environment.cpu,
            result.test_environment.memory
        ));

        markdown.push_str("## Performance Summary\n\n");
        markdown.push_str(&result.summary.generate_summary());
        markdown.push_str("\n\n");

        markdown.push_str("## Detailed Results\n\n");
        markdown.push_str("| Test Name | Winner | Beejs Time | Node.js Time | Bun Time | Speedup vs Node.js | Speedup vs Bun |\n");
        markdown.push_str("|-----------|--------|------------|--------------|----------|-------------------|----------------|\n");

        for test_result in &result.test_results {
            let beejs_time = test_result
                .beejs_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let nodejs_time = test_result
                .nodejs_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let bun_time = test_result
                .bun_result
                .as_ref()
                .map(|r| format!("{:.2}ms", r.avg_duration.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            markdown.push_str(&format!(
                "| {} | **{}** | {} | {} | {} | {:.2}x | {:.2}x |\n",
                test_result.test_name,
                test_result.winner,
                beejs_time,
                nodejs_time,
                bun_time,
                test_result.speedup_vs_nodejs,
                test_result.speedup_vs_bun
            ));
        }

        Ok(markdown)
    }

    /// 生成图表脚本
    fn generate_chart_scripts(&self, result: &ComparisonResult) -> Result<String, Box<dyn std::error::Error>> {
        let mut scripts = String::new();

        // 准备速度提升图表数据
        let test_names: Vec<String> = result.test_results.iter().map(|r| r.test_name.clone()).collect();
        let speedup_nodejs: Vec<f64> = result.test_results.iter().map(|r| r.speedup_vs_nodejs).collect();
        let speedup_bun: Vec<f64> = result.test_results.iter().map(|r| r.speedup_vs_bun).collect();

        scripts.push_str(&format!(
            r#"
        <script>
            // Speedup Chart
            const speedupCtx = document.getElementById('speedupChart').getContext('2d');
            new Chart(speedupCtx, {{
                type: 'bar',
                data: {{
                    labels: {},
                    datasets: [{{
                        label: 'Speedup vs Node.js',
                        data: {},
                        backgroundColor: 'rgba(52, 152, 219, 0.7)',
                        borderColor: 'rgba(52, 152, 219, 1)',
                        borderWidth: 1
                    }}, {{
                        label: 'Speedup vs Bun',
                        data: {},
                        backgroundColor: 'rgba(46, 204, 113, 0.7)',
                        borderColor: 'rgba(46, 204, 113, 1)',
                        borderWidth: 1
                    }}]
                }},
                options: {{
                    responsive: true,
                    scales: {{
                        y: {{
                            beginAtZero: true,
                            title: {{
                                display: true,
                                text: 'Speedup Factor'
                            }}
                        }}
                    }}
                }}
            }});

            // Win Rate Chart
            const winRateCtx = document.getElementById('winRateChart').getContext('2d');
            new Chart(winRateCtx, {{
                type: 'pie',
                data: {{
                    labels: ['Beejs Wins', 'Node.js Wins', 'Bun Wins'],
                    datasets: [{{
                        data: [{}, {}, {}],
                        backgroundColor: [
                            'rgba(52, 152, 219, 0.7)',
                            'rgba(231, 76, 60, 0.7)',
                            'rgba(241, 196, 15, 0.7)'
                        ],
                        borderColor: [
                            'rgba(52, 152, 219, 1)',
                            'rgba(231, 76, 60, 1)',
                            'rgba(241, 196, 15, 1)'
                        ],
                        borderWidth: 1
                    }}]
                }},
                options: {{
                    responsive: true,
                    plugins: {{
                        title: {{
                            display: true,
                            text: 'Win Rate Distribution'
                        }}
                    }}
                }}
            }});
        </script>
"#,
            serde_json::to_string(&test_names)?,
            serde_json::to_string(&speedup_nodejs)?,
            serde_json::to_string(&speedup_bun)?,
            result.summary.beejs_wins,
            result.summary.nodejs_wins,
            result.summary.bun_wins
        ));

        Ok(scripts)
    }
}
