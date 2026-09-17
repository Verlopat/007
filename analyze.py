import json
from pathlib import Path

def load_jsonl(path):
    with open(path, "r", encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]

def summarize(rows):
    n = len(rows)
    return {
        "rounds": n,
        "success_rate": (sum(1 for r in rows if r["success"]) / n) if n else 0.0,
        "liveness_fail_rate": (sum(1 for r in rows if r["liveness_fail"]) / n) if n else 0.0,
        "avg_tally_latency_ms": (sum(r["tally_latency_ms"] for r in rows) / n) if n else 0.0,
        "avg_bad_seats": (sum(r["adversary_controlled_seats"] for r in rows) / n) if n else 0.0,
        "avg_new_threshold": (sum(r["new_threshold"] for r in rows) / n) if n else 0.0,
    }

files = sorted(Path("output/logs").glob("sim*.jsonl"))

if not files:
    print("No sim*.jsonl files found in output/logs")
    raise SystemExit(1)

print(f"{'file':<18} {'rounds':>6} {'succ':>8} {'live_fail':>10} {'lat_ms':>10} {'bad_seats':>10} {'new_t':>8}")
print("-" * 78)

for path in files:
    rows = load_jsonl(path)
    s = summarize(rows)
    print(
        f"{path.name:<18} "
        f"{s['rounds']:>6} "
        f"{s['success_rate']:>8.3f} "
        f"{s['liveness_fail_rate']:>10.3f} "
        f"{s['avg_tally_latency_ms']:>10.2f} "
        f"{s['avg_bad_seats']:>10.2f} "
        f"{s['avg_new_threshold']:>8.2f}"
    )
