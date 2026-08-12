#!/usr/bin/env bash
# Beejs performance baseline runner (startup + eval throughput).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "Building release bee..."
cargo build --release -q

BEE="$ROOT/target/release/bee"
OUT_DIR="$ROOT/benchmarks/results"
mkdir -p "$OUT_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
REPORT="$OUT_DIR/baseline_${STAMP}.md"

run_timed() {
  local label="$1"
  shift
  local start end
  start=$(python3 - <<'PY'
import time; print(time.perf_counter())
PY
)
  "$@" >/dev/null
  end=$(python3 - <<'PY'
import time; print(time.perf_counter())
PY
)
  python3 - <<PY
start=float("$start"); end=float("$end")
print(f"{end-start:.6f}")
PY
}

echo "Running startup / eval baseline..."
{
  echo "# Beejs performance baseline"
  echo
  echo "Generated: ${STAMP}"
  echo
  echo "| Metric | Seconds |"
  echo "|--------|---------|"
} >"$REPORT"

iters="${BENCH_ITERS:-20}"
total=0
for i in $(seq 1 "$iters"); do
  t=$("$BEE" eval "1+1" >/dev/null; python3 - <<'PY'
import time, subprocess, os
bee=os.environ.get("BEE")
# placeholder
print(0)
PY
)
done

# Prefer /usr/bin/time when available
if command -v gtime >/dev/null 2>&1; then
  TIME_BIN=gtime
elif /usr/bin/time -p true >/dev/null 2>&1; then
  TIME_BIN=/usr/bin/time
else
  TIME_BIN=""
fi

measure() {
  local label="$1"
  shift
  if [[ -n "$TIME_BIN" ]]; then
    local out
    out=$($TIME_BIN -p "$@" 2>&1 >/dev/null | awk '/^real /{print $2}')
    echo "| $label | ${out}s |" >>"$REPORT"
    echo "$label: ${out}s"
  else
    local start end
    start=$(date +%s.%N)
    "$@" >/dev/null
    end=$(date +%s.%N)
    local dur
    dur=$(python3 -c "print(f'{float('$end')-float('$start'):.6f}')")
    echo "| $label | ${dur}s |" >>"$REPORT"
    echo "$label: ${dur}s"
  fi
}

measure "cold_eval_1plus1" "$BEE" eval "1 + 1"
measure "run_hello" "$BEE" run examples/basics/hello_world.js

# Optional peer comparison when binaries exist
if command -v node >/dev/null 2>&1; then
  measure "node_eval_1plus1" node -e "1+1"
fi
if command -v bun >/dev/null 2>&1; then
  measure "bun_eval_1plus1" bun -e "1+1"
fi

# Criterion-less cargo bench (custom harness)
if cargo bench -q --bench runtime_startup 2>/dev/null; then
  echo "| runtime_startup_bench | ok |" >>"$REPORT"
fi

echo
echo "Wrote $REPORT"
cat "$REPORT"
