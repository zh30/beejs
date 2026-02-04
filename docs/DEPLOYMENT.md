# Beejs 生产环境部署指南

## 概述

Beejs 是一个高性能的 JavaScript/TypeScript 运行时，专为 AI 时代的高性能脚本执行而设计。本指南将帮助您在生产环境中部署和优化 Beejs。

## 系统要求

### 最低要求
- **操作系统**: Linux (Ubuntu 20.04+), macOS (10.15+), 或 Windows 10+
- **内存**: 2GB RAM (推荐 4GB+)
- **CPU**: 2 核 (推荐 4 核+)
- **磁盘空间**: 500MB

### 推荐配置
- **内存**: 8GB+ RAM
- **CPU**: 8 核+ CPU
- **磁盘**: SSD

## 部署方式

### 方式一：一键安装 (推荐)

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh
beejs --version
```

### 方式二：二进制部署 (手动)

1. **下载预编译二进制文件**
   ```bash
   # 选择版本和平台
   VERSION=v0.1.0
   TARGET=x86_64-unknown-linux-gnu

   # 下载指定版本
   curl -L https://github.com/zh30/beejs/releases/download/${VERSION}/beejs-${VERSION}-${TARGET}.tar.gz -o beejs.tar.gz
   tar -xzf beejs.tar.gz
   chmod +x beejs
   ```

2. **安装到系统路径**
   ```bash
   mkdir -p ~/.beejs/bin
   mv beejs ~/.beejs/bin/
   export PATH=\"$HOME/.beejs/bin:$PATH\"
   beejs --version
   ```

3. **验证安装**
   ```bash
   beejs --eval 'console.log("Hello from Beejs!"); 1+1'
   ```

### 方式二：源码编译

1. **安装 Rust 工具链**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **编译项目**
   ```bash
   git clone https://github.com/zh30/beejs.git
   cd beejs
   cargo build --release
   ```

3. **安装**
   ```bash
   sudo cp target/release/beejs /usr/local/bin/
   ```

## 性能优化配置

### 内存配置

```bash
# 为大型应用配置更大的堆内存
beejs --max-heap 2147483648 --stack-size 134217728 script.js
```

### V8 优化

```bash
# 性能优先 (推荐用于生产)
beejs --optimize speed script.js

# 代码大小优先 (用于资源受限环境)
beejs --optimize size script.js

# 自动优化 (基于代码复杂度)
beejs --optimize auto script.js
```

### Isolate 池化

Beejs 自动使用 V8 Isolate 池化以提高性能：
- 默认池大小: CPU 核心数 (最大 8)
- 在生产环境中自动启用

## 生产环境最佳实践

### 1. 进程管理

使用 systemd 管理 Beejs 进程：

```ini
# /etc/systemd/system/beejs.service
[Unit]
Description=Beejs Runtime
After=network.target

[Service]
Type=simple
User=beejs
WorkingDirectory=/opt/beejs
ExecStart=/usr/local/bin/beejs --optimize speed --max-heap 1073741824 /opt/beejs/app.js
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

启用并启动服务：
```bash
sudo systemctl enable beejs
sudo systemctl start beejs
```

### 2. 监控

#### 日志配置
```bash
# 启用详细日志
beejs --verbose script.js 2>&1 | tee /var/log/beejs.log
```

#### 性能监控
```bash
# 查看内存使用
ps aux | grep beejs

# 查看 CPU 使用
top -p $(pgrep beejs)
```

### 3. 安全配置

#### 限制资源使用
```bash
# 使用 cgroups 限制内存
sudo cgcreate -g memory:beejs
sudo cgset -r memory.limit_in_bytes=1073741824 beejs
sudo cgexec -g memory:beejs beejs script.js
```

#### 文件权限
```bash
# 确保 Beejs 二进制文件权限正确
sudo chmod 755 /usr/local/bin/beejs
sudo chown root:root /usr/local/bin/beejs
```

### 4. 负载均衡

对于高并发场景，使用 PM2 或类似工具：

```bash
# 安装 PM2
npm install -g pm2

# 启动多个实例
pm2 start beejs --name "beejs-1" -- script.js
pm2 start beejs --name "beejs-2" -- script.js

# 查看状态
pm2 status
```

## 容器化部署

### Docker 部署

1. **创建 Dockerfile**
   ```dockerfile
   FROM ubuntu:22.04

   # 安装依赖
   RUN apt-get update && apt-get install -y \
       ca-certificates \
       && rm -rf /var/lib/apt/lists/*

   # 复制 Beejs 二进制文件
   COPY beejs /usr/local/bin/beejs
   RUN chmod +x /usr/local/bin/beejs

   # 设置工作目录
   WORKDIR /app

   # 复制应用文件
   COPY . .

   # 运行应用
   CMD ["beejs", "script.js"]
   ```

2. **构建镜像**
   ```bash
   docker build -t beejs:latest .
   ```

3. **运行容器**
   ```bash
   docker run -d --name beejs-app beejs:latest
   ```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  beejs:
    build: .
    restart: always
    environment:
      - NODE_ENV=production
    volumes:
      - ./app:/app
    ports:
      - "3000:3000"
```

## 故障排除

### 常见问题

1. **内存不足错误**
   ```bash
   # 增加堆内存大小
   beejs --max-heap 2147483648 script.js
   ```

2. **V8 编译错误**
   ```bash
   # 禁用优化
   beejs --optimize size script.js
   ```

3. **性能问题**
   ```bash
   # 启用详细日志查看性能指标
   beejs --verbose script.js
   ```

### 日志分析

查看错误日志：
```bash
journalctl -u beejs -f
```

查看应用日志：
```bash
tail -f /var/log/beejs.log
```

## 升级指南

### 升级 Beejs

1. **备份当前版本**
   ```bash
   cp ~/.beejs/bin/beejs ~/.beejs/bin/beejs.backup
   ```

2. **安装新版本**
   ```bash
   VERSION=v0.1.0
   TARGET=x86_64-unknown-linux-gnu
   curl -L https://github.com/zh30/beejs/releases/download/${VERSION}/beejs-${VERSION}-${TARGET}.tar.gz -o beejs.tar.gz
   tar -xzf beejs.tar.gz
   cp beejs ~/.beejs/bin/beejs
   ```

3. **验证升级**
   ```bash
   beejs --version
   beejs --eval 'console.log("Upgrade test"); 1+1'
   ```

4. **回滚（如需要）**
   ```bash
   cp ~/.beejs/bin/beejs.backup ~/.beejs/bin/beejs
   ```

## 性能基准

参考性能数据（基于标准硬件）：

- **启动时间**: ~17μs
- **代码执行**: ~18μs (简单操作)
- **算术运算**: ~18μs
- **内存使用**: 比 Node.js 优化 15%
- **并发能力**: 支持 10000+ 并发脚本

## 支持

- **文档**: [https://docs.beejs.dev](https://docs.beejs.dev)
- **GitHub**: [https://github.com/zh30/beejs](https://github.com/zh30/beejs)
- **问题报告**: [https://github.com/zh30/beejs/issues](https://github.com/zh30/beejs/issues)
- **社区**: [https://discord.gg/beejs](https://discord.gg/beejs)

---

*最后更新: 2025-12-18*
