#!/bin/bash
# 性能基线更新脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 帮助信息
show_help() {
    cat << EOF
性能基线更新脚本

用法:
    $0 [选项]

选项:
    -h, --help              显示此帮助信息
    -v, --version VERSION   指定新版本号 (默认: 当前日期)
    -f, --force             强制更新基线，不进行验证
    -c, --check             仅检查当前基线状态
    -b, --backup            在更新前备份当前基线
    -t, --threshold FILE    指定阈值配置文件
    -o, --output FILE       指定输出文件路径

示例:
    $0 -v v0.1.0-stage96
    $0 --check
    $0 --backup --force
EOF
}

# 默认参数
VERSION=""
FORCE=false
CHECK_ONLY=false
BACKUP=false
THRESHOLD_FILE="$PROJECT_ROOT/config/perf_thresholds.json"
OUTPUT_FILE="$PROJECT_ROOT/.github/perf_baseline.json"

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        -c|--check)
            CHECK_ONLY=true
            shift
            ;;
        -b|--backup)
            BACKUP=true
            shift
            ;;
        -t|--threshold)
            THRESHOLD_FILE="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        *)
            log_error "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 如果未指定版本，使用当前日期
if [[ -z "$VERSION" ]]; then
    VERSION="v$(date +%Y%m%d)-$(git rev-parse --short HEAD)"
    log_info "使用自动生成版本号: $VERSION"
fi

# 检查必要文件
check_requirements() {
    log_info "检查必要文件..."

    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "未找到 Cargo.toml"
        exit 1
    fi

    if [[ ! -f "$THRESHOLD_FILE" ]]; then
        log_warn "未找到阈值配置文件: $THRESHOLD_FILE"
    fi

    if [[ ! -x "$(command -v cargo)" ]]; then
        log_error "未安装 Rust/Cargo"
        exit 1
    fi
}

# 检查当前基线状态
check_baseline() {
    log_info "检查当前基线状态..."

    if [[ -f "$OUTPUT_FILE" ]]; then
        log_info "找到现有基线文件: $OUTPUT_FILE"

        # 显示基线信息
        if command -v jq &> /dev/null; then
            local baseline_version=$(jq -r '.version' "$OUTPUT_FILE" 2>/dev/null || echo "unknown")
            local baseline_timestamp=$(jq -r '.timestamp' "$OUTPUT_FILE" 2>/dev/null || echo "unknown")
            log_info "基线版本: $baseline_version"
            log_info "基线时间戳: $baseline_timestamp"
        fi

        return 0
    else
        log_warn "未找到基线文件: $OUTPUT_FILE"
        return 1
    fi
}

# 备份当前基线
backup_baseline() {
    if [[ -f "$OUTPUT_FILE" ]]; then
        local backup_file="${OUTPUT_FILE}.bak.$(date +%Y%m%d_%H%M%S)"
        cp "$OUTPUT_FILE" "$backup_file"
        log_info "基线已备份到: $backup_file"
    fi
}

# 运行性能基准测试
run_benchmarks() {
    log_info "运行性能基准测试..."

    cd "$PROJECT_ROOT"

    # 构建项目
    log_info "构建项目..."
    cargo build --release

    # 运行基准测试
    log_info "执行基准测试套件..."
    cargo bench -- --output-format json > "perf_results_${VERSION}.json"

    if [[ ! -f "perf_results_${VERSION}.json" ]]; then
        log_error "基准测试失败，未生成结果文件"
        exit 1
    fi

    log_info "基准测试完成，结果保存在: perf_results_${VERSION}.json"
}

# 验证基准测试结果
validate_results() {
    local results_file="perf_results_${VERSION}.json"

    log_info "验证基准测试结果..."

    # 检查文件大小
    local file_size=$(wc -c < "$results_file")
    if [[ $file_size -lt 1024 ]]; then
        log_error "结果文件过小，可能测试失败"
        return 1
    fi

    # 使用 jq 验证 JSON 格式
    if command -v jq &> /dev/null; then
        if ! jq empty "$results_file" 2>/dev/null; then
            log_error "结果文件 JSON 格式无效"
            return 1
        fi

        # 检查必要字段
        local bench_count=$(jq '.benchmarks | length' "$results_file" 2>/dev/null || echo "0")
        if [[ $bench_count -eq 0 ]]; then
            log_error "结果文件中没有基准测试数据"
            return 1
        fi

        log_info "验证通过，包含 $bench_count 个基准测试"
    fi

    return 0
}

# 更新基线
update_baseline() {
    local results_file="perf_results_${VERSION}.json"

    log_info "更新性能基线到版本: $VERSION"

    # 创建基线格式
    if command -v jq &> /dev/null; then
        # 转换基准测试结果为基线格式
        jq -c --arg version "$VERSION" --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" '
            {
                version: $version,
                timestamp: $timestamp,
                benchmarks: .benchmarks | to_entries | map({
                    key: .key,
                    value: {
                        mean: .value.mean,
                        median: .value.median,
                        p95: .value.p95,
                        p99: .value.p99,
                        std_dev: .value.std_dev,
                        samples: .value.samples,
                        unit: .value.unit
                    }
                }) | from_entries
            }
        ' "$results_file" > "$OUTPUT_FILE"
    else
        # 简单复制（如果 jq 不可用）
        cp "$results_file" "$OUTPUT_FILE"
        log_warn "jq 不可用，使用简单复制方式"
    fi

    log_info "基线已更新: $OUTPUT_FILE"
}

# 清理临时文件
cleanup() {
    local results_file="perf_results_${VERSION}.json"
    if [[ -f "$results_file" ]]; then
        rm "$results_file"
        log_info "清理临时文件: $results_file"
    fi
}

# 主函数
main() {
    log_info "开始性能基线更新流程..."

    # 检查要求
    check_requirements

    # 检查模式
    if [[ "$CHECK_ONLY" == true ]]; then
        check_baseline
        exit 0
    fi

    # 检查现有基线
    local has_baseline=false
    if check_baseline; then
        has_baseline=true
    fi

    # 备份现有基线
    if [[ "$BACKUP" == true && "$has_baseline" == true ]]; then
        backup_baseline
    fi

    # 运行基准测试
    run_benchmarks

    # 验证结果
    if [[ "$FORCE" != true ]]; then
        if ! validate_results; then
            log_error "基准测试结果验证失败"
            cleanup
            exit 1
        fi
    fi

    # 更新基线
    update_baseline

    # 清理
    cleanup

    log_info "性能基线更新完成!"
    log_info "新版本: $VERSION"
    log_info "基线文件: $OUTPUT_FILE"
}

# 错误处理
trap 'log_error "脚本执行失败，行号: $LINENO"' ERR

# 运行主函数
main
