---
title: "How We Achieved 11ms Startup Time with Isolate Pooling"
date: "Dec 12, 2025"
author: "Beejs Team"
readTime: "12 min read"
tag: "Engineering"
---

# How We Achieved 11ms Startup Time

Startup time is critical for serverless and AI agents. Learn about our unique approach to V8 isolate management.

## Isolate Pooling

By maintaining a pool of pre-warmed V8 isolates, we bypass the heavy initialization phase, allowing scripts to start in as little as 11ms.

### Key Benefits:
- Reduced latency
- Better resource utilization
- Scaling with ease
