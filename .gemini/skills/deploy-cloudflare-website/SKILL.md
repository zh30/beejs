---
name: deploy-cloudflare-website
description: Cloudflare Workers Assets / Vite React 网站自动化打包构建与一键发布技能。仅作用于本项目，当在 Beejs 项目中需要发布、更新或部署 website 前端到 Cloudflare 时自动使用。
---

# Deploy Cloudflare Website Skill (Project Level)

本 Skill 提供了将 **Beejs 官方网站 (`website/`)** 一键部署发布到 **Cloudflare Workers & Assets** 全球边缘节点的标准自动化流程。

---

## 1. 适用场景

- 本项目 `website/` 目录的前端编译与 Cloudflare 部署发布。
- 使用 `wrangler deploy` 命令行一键编译发布，且自动处理 SPA 单页路由 404 降级（`/index.html` fallback）。

---

## 2. 核心配置文件说明

### 2.1 Worker Assets 路由文件 (`website/src/worker.ts`)

在 `website/src/worker.ts` 中配置支持 SPA 路由降级：

```typescript
interface Env {
  ASSETS: {
    fetch: (request: Request) => Promise<Response>;
  };
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // 1. 尝试直接获取静态 Asset 资源 (CSS/JS/图片/index.html)
    const response = await env.ASSETS.fetch(request);
    if (response.status !== 404) {
      return response;
    }

    // 2. 如果返回 404，针对 SPA 路由自动降级转发至 /index.html
    const indexRequest = new Request(new URL('/index.html', url.origin), request);
    return env.ASSETS.fetch(indexRequest);
  },
};
```

---

### 2.2 配置文件 (`website/wrangler.toml`)

```toml
name = "beejs-website"
main = "src/worker.ts"
compatibility_date = "2024-11-01"
assets = { directory = "./dist" }

[build]
command = "pnpm run build"
watch_dir = "src"
```

---

## 3. 标准构建与部署流程

### 步骤 1：进入 website 目录并打包
```bash
cd website
pnpm run build
```

### 步骤 2：部署预演验证 (Dry Run)
```bash
pnpm run deploy:dry-run
```

### 步骤 3：正式一键发布
```bash
pnpm run deploy
```

上线域名：`https://beejs-website.nanhetech.workers.dev`

---

## 4. 故障排查 (Troubleshooting)

| 异常现象 / 错误码 | 原因分析 | 解决方案 |
| :--- | :--- | :--- |
| **Error Code 1042 / 404** | 静态 Assets 配置缺乏 Worker 入口脚本。 | 确保 `website/src/worker.ts` 存在且 `wrangler.toml` 中配置了 `main = "src/worker.ts"`。 |
| **OAuth Token missing `pages:write`** | 使用了 `pages deploy` 但 Token 仅有 Workers 权限。 | 使用项目规定的 `pnpm run deploy` (`wrangler deploy` Worker Assets 模式)。 |
