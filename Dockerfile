# ==============================================================================
# Beejs Production Docker Image
# Optimized for high-performance JavaScript/TypeScript runtime
# ==============================================================================

# 阶段 1: 构建阶段
FROM rust:1.75-slim AS builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 src 目录以避免 Cargo.toml 路径问题
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# 构建依赖项（缓存层）
RUN cargo build --release && rm src/main.rs

# 复制源代码
COPY src ./src
COPY tests ./tests
COPY wasm_cache_high_perf ./wasm_cache_high_perf

# 构建生产版本
RUN cargo build --release

# 阶段 2: 运行时阶段 - 最小化镜像
FROM debian:bookworm-slim AS runtime

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# 创建非特权用户
RUN groupadd -r beejs && useradd -r -g beejs beejs

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/beejs /usr/local/bin/beejs
COPY --from=builder /app/target/release/beejs-benchmark /usr/local/bin/beejs-benchmark

# 创建必要的目录
RUN mkdir -p /app/cache /app/logs /app/tmp && \
    chown -R beejs:beejs /app

# 复制示例和文档
COPY --chown=beejs:beejs README.md /app/
COPY --chown=beejs:beejs examples/ /app/examples/

# 切换到非特权用户
USER beejs

# 设置默认环境变量
ENV BEEJS_MODE=production
ENV BEEJS_LOG_LEVEL=info
ENV BEEJS_MAX_CONNECTIONS=10000
ENV BEEJS_BATCH_SIZE=100
ENV BEEJS_CACHE_DIR=/app/cache
ENV BEEJS_TMP_DIR=/app/tmp

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD beejs --health-check || exit 1

# 启动命令
CMD ["beejs", "serve", "--host", "0.0.0.0", "--port", "3000"]

# ==============================================================================
# 多阶段构建说明:
# - builder: 编译 Rust 代码
# - runtime: 最小化生产镜像
# ==============================================================================
