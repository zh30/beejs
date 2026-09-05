#!/usr/bin/env python3
"""
benchmarks/idle_memory/runner.py

Reproduces Bun's Idle Memory Benchmark:
"Resident memory 3 minutes after 60 seconds of sustained load, lower is better."
Reference: https://x.com/bunjavascript/status/2095696147813945347

Methodology:
1. Start server process (Bun v1.4.1, Node.js v22, or Beejs v0.4.0).
2. Measure Baseline Idle RSS (MB) before traffic.
3. Apply sustained HTTP load using autocannon (e.g. 64 connections for 60s in full mode, or 10s in quick mode).
4. Measure Peak RSS (MB) and throughput (req/sec) during load.
5. Enter Cooldown Idle period (180s in full mode, or 15s in quick mode) with 0 traffic.
6. Sample RSS every 100 ms throughout the entire cycle.
7. Record Settled Idle RSS (MB) at the end of the cooldown period.
8. Calculate Memory Recovery Ratio (% of temporary memory returned to OS).
9. Output comparative markdown table & JSON time-series.
"""

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import threading
import time
import http.client
import urllib.request
import urllib.parse
from pathlib import Path


def get_rss_mb(pid: int) -> float:
    """Retrieve current RSS memory in Megabytes for a given PID."""
    try:
        # On macOS and Linux, `ps -o rss= -p <PID>` returns RSS in kilobytes
        output = subprocess.check_output(
            ["ps", "-o", "rss=", "-p", str(pid)],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        if output:
            kb = float(output.split()[0])
            return round(kb / 1024.0, 2)
    except Exception:
        pass
    return 0.0


class MemorySampler(threading.Thread):
    def __init__(self, pid: int, sample_interval_sec: float = 0.5):
        super().__init__(daemon=True)
        self.pid = pid
        self.interval = sample_interval_sec
        self.running = True
        self.samples = []  # (relative_timestamp, rss_mb)
        self.peak_rss = 0.0
        self.start_time = time.time()

    def run(self):
        while self.running:
            rss = get_rss_mb(self.pid)
            if rss > 0:
                elapsed = round(time.time() - self.start_time, 2)
                self.samples.append((elapsed, rss))
                if rss > self.peak_rss:
                    self.peak_rss = rss
            time.sleep(self.interval)

    def stop(self):
        self.running = False


for k in ["http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"]:
    if k in os.environ:
        del os.environ[k]
os.environ["NO_PROXY"] = "*"
os.environ["no_proxy"] = "*"


def wait_for_server(url: str, timeout_sec: float = 10.0) -> bool:
    parsed = urllib.parse.urlparse(url)
    start = time.time()
    while time.time() - start < timeout_sec:
        try:
            conn = http.client.HTTPConnection(parsed.hostname, parsed.port, timeout=1.0)
            conn.request("GET", parsed.path or "/")
            resp = conn.getresponse()
            if resp.status in (200, 404):
                conn.close()
                return True
            conn.close()
        except Exception:
            pass
        time.sleep(0.1)
    return False


def run_autocannon(url: str, connections: int, duration_sec: int, autocannon_bin: str) -> dict:
    """Execute autocannon and extract throughput and latency stats."""
    cmd = [
        autocannon_bin,
        "-c", str(connections),
        "-d", str(duration_sec),
        "-j",  # JSON output
        url,
    ]
    env = os.environ.copy()
    for k in ["http_proxy", "https_proxy", "all_proxy", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY"]:
        env.pop(k, None)
    env["NO_PROXY"] = "*"
    env["no_proxy"] = "*"

    try:
        res = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
            env=env,
        )
        data = json.loads(res.stdout)
        req_avg = data.get("requests", {}).get("average", 0)
        lat_avg = data.get("latency", {}).get("average", 0)
        total_requests = data.get("requests", {}).get("total", 0)
        return {
            "requests_per_sec": round(req_avg, 1),
            "latency_avg_ms": round(lat_avg, 2),
            "total_requests": total_requests,
        }
    except Exception as e:
        # Fallback parsing
        return {
            "requests_per_sec": 0.0,
            "latency_avg_ms": 0.0,
            "total_requests": 0,
            "error": str(e),
        }


def benchmark_single(
    runtime_name: str,
    runtime_cmd: list,
    framework: str,
    server_script: Path,
    port: int,
    load_duration: int,
    cooldown_duration: int,
    connections: int,
    autocannon_bin: str,
) -> dict:
    url = f"http://127.0.0.1:{port}/"
    print(f"\n[{runtime_name.upper()} :: {framework.upper()}] Starting server on port {port}...")

    env = os.environ.copy()
    env["PORT"] = str(port)
    env["http_proxy"] = ""
    env["https_proxy"] = ""

    cmd = runtime_cmd + [str(server_script)]
    log_path = Path(f"/tmp/bench_{runtime_name}_{framework}_{port}.log")
    log_file = open(log_path, "w")
    proc = subprocess.Popen(
        cmd,
        stdout=log_file,
        stderr=log_file,
        text=True,
        env=env,
        preexec_fn=os.setsid if hasattr(os, "setsid") else None,
    )

    try:
        # 1. Healthcheck
        if not wait_for_server(url, timeout_sec=10.0):
            print(f"  ❌ Server failed to respond at {url} within 10s!")
            stderr_out = log_path.read_text() if log_path.exists() else ""
            print(f"  log: {stderr_out}")
            return {"error": "Startup timeout", "stderr": stderr_out}

        # 2. Start RSS Sampling
        sampler = MemorySampler(proc.pid, sample_interval_sec=0.1)
        sampler.start()

        # 3. Pre-load baseline observation (3 seconds)
        print("  ⏳ Measuring baseline idle memory (3s)...")
        time.sleep(3.0)
        baseline_rss = get_rss_mb(proc.pid)
        print(f"  📊 Baseline Idle RSS: {baseline_rss:.2f} MB")

        # 4. Sustained Load
        print(f"  🔥 Applying sustained load ({connections} connections, {load_duration}s)...")
        load_stats = run_autocannon(url, connections, load_duration, autocannon_bin)
        peak_during_load = sampler.peak_rss
        print(f"  🚀 Peak RSS during load: {peak_during_load:.2f} MB | Throughput: {load_stats.get('requests_per_sec', 0):.0f} req/s")

        # 5. Cooldown Idle period
        print(f"  ❄️ Entering cooldown idle period ({cooldown_duration}s) with 0 traffic...")
        time.sleep(cooldown_duration)

        # 6. Record Settled Idle RSS
        settled_rss = get_rss_mb(proc.pid)
        sampler.stop()

        # 7. Compute Recovery Metrics
        allocated_during_load = max(0.0, peak_during_load - baseline_rss)
        reclaimed_memory = max(0.0, peak_during_load - settled_rss)
        if allocated_during_load > 0.1:
            recovery_ratio = round((reclaimed_memory / allocated_during_load) * 100.0, 1)
        else:
            recovery_ratio = 100.0 if settled_rss <= baseline_rss else 0.0

        net_retained_above_baseline = max(0.0, settled_rss - baseline_rss)

        print(f"  🎯 Settled Idle RSS: {settled_rss:.2f} MB")
        print(f"  ♻️ Memory Reclamation: {recovery_ratio:.1f}% recovered (Peak {peak_during_load:.2f}MB -> Idle {settled_rss:.2f}MB)")

        return {
            "runtime": runtime_name,
            "framework": framework,
            "baseline_rss_mb": baseline_rss,
            "peak_rss_mb": peak_during_load,
            "settled_rss_mb": settled_rss,
            "net_growth_mb": round(net_retained_above_baseline, 2),
            "recovery_ratio_pct": recovery_ratio,
            "throughput_req_s": load_stats.get("requests_per_sec", 0),
            "latency_ms": load_stats.get("latency_avg_ms", 0),
            "total_requests": load_stats.get("total_requests", 0),
            "samples_count": len(sampler.samples),
            "time_series": sampler.samples,
        }

    finally:
        # Graceful shutdown
        try:
            if hasattr(os, "killpg"):
                os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
            else:
                proc.terminate()
            proc.wait(timeout=3.0)
        except Exception:
            try:
                if hasattr(os, "killpg"):
                    os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
                else:
                    proc.kill()
            except Exception:
                pass


def format_markdown_report(results: list, load_duration: int, cooldown_duration: int, connections: int) -> str:
    md = [
        "# Runtime Idle Memory Benchmark Report",
        "\n> **Benchmark Methodology Alignment**: Direct reproduction of Bun's published benchmark (*\"Resident memory 3 minutes after 60 seconds of sustained load, lower is better\"*).",
        f"\n- **Sustained Load Duration**: `{load_duration}s`",
        f"- **Idle Cooldown Duration**: `{cooldown_duration}s`",
        f"- **Concurrent Connections**: `{connections}`",
        "- **Sampling Frequency**: `Every 100 ms via OS Process RSS`\n",
        "## 📊 1. Core Comparison: Settled Idle Memory (Lower is Better)",
        "\n| Framework | Bun v1.4.1 (RSS) | Node.js v22 (RSS) | Beejs v0.4.0 (RSS) | Lowest Idle RSS Winner |",
        "|---|---|---|---|---|",
    ]

    # Group by framework
    frameworks = sorted(list(set(r["framework"] for r in results if "error" not in r)))
    for fw in frameworks:
        fw_results = {r["runtime"]: r for r in results if r.get("framework") == fw and "error" not in r}
        bun_val = f"{fw_results['bun']['settled_rss_mb']:.1f} MB" if "bun" in fw_results else "N/A"
        node_val = f"{fw_results['node']['settled_rss_mb']:.1f} MB" if "node" in fw_results else "N/A"
        bee_val = f"{fw_results['bee']['settled_rss_mb']:.1f} MB" if "bee" in fw_results else "N/A"

        # Determine winner
        candidates = []
        if "bun" in fw_results:
            candidates.append(("Bun", fw_results["bun"]["settled_rss_mb"]))
        if "node" in fw_results:
            candidates.append(("Node.js", fw_results["node"]["settled_rss_mb"]))
        if "bee" in fw_results:
            candidates.append(("Beejs", fw_results["bee"]["settled_rss_mb"]))

        if candidates:
            winner = min(candidates, key=lambda x: x[1])[0]
        else:
            winner = "N/A"

        md.append(f"| **{fw.capitalize()}** | {bun_val} | {node_val} | {bee_val} | 🏆 **{winner}** |")

    md.append("\n## 📈 2. Detailed Lifecycle Metrics (Baseline -> Peak -> Settled)")
    md.append("\n| Runtime | Framework | Baseline RSS | Peak Under Load | Settled Idle RSS | Net Memory Retained | Recovery Ratio | Throughput |")
    md.append("|---|---|---|---|---|---|---|---|")

    for r in results:
        if "error" in r:
            md.append(f"| {r.get('runtime', '?')} | {r.get('framework', '?')} | ERROR | ERROR | ERROR | ERROR | ERROR | N/A |")
            continue
        md.append(
            f"| `{r['runtime']}` | **{r['framework']}** | {r['baseline_rss_mb']:.1f} MB | {r['peak_rss_mb']:.1f} MB | **{r['settled_rss_mb']:.1f} MB** | +{r['net_growth_mb']:.1f} MB | {r['recovery_ratio_pct']:.1f}% | {r['throughput_req_s']:.0f} req/s |"
        )

    md.append("\n## 🔬 3. Key Observations & Architecture Analysis")
    md.append("- **Idle Memory Reclamation**: Measuring memory 3 minutes (or cooldown) after sustained traffic exposes whether runtimes release heap pages to the OS or retain slab allocators.")
    md.append("- **Peak vs Idle Spread**: Demonstrates the resilience of garbage collection and allocator defragmentation under zero-traffic transitions.")
    md.append("\n*(Generated by Beejs Idle Memory Benchmark Suite)*\n")

    return "\n".join(md)


def main():
    parser = argparse.ArgumentParser(description="Bun-aligned Idle Memory Benchmark Runner")
    parser.add_argument("--mode", choices=["quick", "full"], default="quick", help="quick (10s load, 15s cooldown) or full (60s load, 180s cooldown)")
    parser.add_argument("--connections", type=int, default=64, help="Concurrent connections (default: 64)")
    parser.add_argument("--duration", type=int, default=None, help="Override sustained load duration in seconds")
    parser.add_argument("--cooldown", type=int, default=None, help="Override idle cooldown duration in seconds")
    parser.add_argument("--workloads", default="hono,http,express,fastify", help="Comma-separated workloads to test")
    parser.add_argument("--runtimes", default="bun,node,bee", help="Comma-separated runtimes to test (bun,node,bee)")
    parser.add_argument("--output-json", default="benchmarks/idle_memory/results.json", help="Path to output JSON")
    parser.add_argument("--output-md", default="benchmarks/idle_memory/REPORT.md", help="Path to output Markdown")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    bench_dir = repo_root / "benchmarks" / "idle_memory"

    # Durations based on mode
    if args.mode == "full":
        load_duration = args.duration if args.duration else 60
        cooldown_duration = args.cooldown if args.cooldown else 180
    else:
        load_duration = args.duration if args.duration else 10
        cooldown_duration = args.cooldown if args.cooldown else 15

    connections = args.connections

    # Autocannon binary location
    autocannon_bin = bench_dir / "node_modules" / ".bin" / "autocannon"
    if not autocannon_bin.exists():
        autocannon_bin = "autocannon"
    else:
        autocannon_bin = str(autocannon_bin)

    # Runtime command configurations
    bee_bin = repo_root / "target" / "release" / "bee"
    runtimes_config = {
        "bun": ["bun", "run"],
        "node": ["node"],
        "bee": [str(bee_bin), "run"],
    }

    workloads_map = {
        "hono": bench_dir / "server_hono.js",
        "http": bench_dir / "server_http.js",
        "express": bench_dir / "server_express.js",
        "fastify": bench_dir / "server_fastify.js",
    }

    # Bee supports hono, http, and express
    supported_matrix = {
        "bun": ["hono", "http", "express", "fastify"],
        "node": ["hono", "http", "express", "fastify"],
        "bee": ["hono", "http", "express"],
    }

    selected_runtimes = [r.strip() for r in args.runtimes.split(",") if r.strip() in runtimes_config]
    selected_workloads = [w.strip() for w in args.workloads.split(",") if w.strip() in workloads_map]

    print("=================================================================")
    print("🚀 Bun-Aligned Idle Memory Benchmark (RSS after Sustained Load)")
    print("=================================================================")
    print(f"• Mode: {args.mode.upper()}")
    print(f"• Load Duration: {load_duration}s | Cooldown Idle: {cooldown_duration}s")
    print(f"• Concurrency: {connections} connections")
    print(f"• Runtimes: {', '.join(selected_runtimes)}")
    print(f"• Workloads: {', '.join(selected_workloads)}")
    print("=================================================================")

    all_results = []
    base_port = 3100

    for fw in selected_workloads:
        server_script = workloads_map[fw]
        for rt in selected_runtimes:
            if fw not in supported_matrix.get(rt, []):
                continue

            base_port += 1
            res = benchmark_single(
                runtime_name=rt,
                runtime_cmd=runtimes_config[rt],
                framework=fw,
                server_script=server_script,
                port=base_port,
                load_duration=load_duration,
                cooldown_duration=cooldown_duration,
                connections=connections,
                autocannon_bin=autocannon_bin,
            )
            all_results.append(res)
            time.sleep(1.0)  # brief pause between tests

    # Generate Report
    report_md = format_markdown_report(all_results, load_duration, cooldown_duration, connections)
    print("\n" + report_md)

    # Save outputs
    out_md_path = repo_root / args.output_md
    out_json_path = repo_root / args.output_json
    out_md_path.write_text(report_md, encoding="utf-8")

    # Serialize JSON (stripping time_series for clean summary or preserving in separate field)
    with open(out_json_path, "w", encoding="utf-8") as f:
        json.dump(all_results, f, indent=2)

    print(f"\n[✓] Benchmark completed!")
    print(f"[✓] Markdown Report saved to: {out_md_path}")
    print(f"[✓] Raw JSON data saved to: {out_json_path}")


if __name__ == "__main__":
    main()
