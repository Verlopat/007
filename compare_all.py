import json
import os
from pathlib import Path
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np

def load_jsonl(path):
    with open(path, "r", encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]

# 1. Load all experiment files
log_dir = Path("output/logs")
files = list(log_dir.glob("*.jsonl"))

if not files:
    raise RuntimeError("No sim logs in output/logs")

# Group by (mode, sybil_frac) from filename
by_mode_frac = {}
for path in files:
    name = path.name
    if "tdg_" in name:
        mode = "tdg"
    elif "fixed_" in name:
        mode = "fixed-threshold"
    elif "naive_" in name:
        mode = "naive"
    else:
        continue

    frac = 0.0
    if "10" in name:
        frac = 0.10
    elif "30" in name:
        frac = 0.30
    elif "50" in name:
        frac = 0.50

    key = (mode, frac)
    if key not in by_mode_frac:
        by_mode_frac[key] = []
    by_mode_frac[key].extend(load_jsonl(path))

# 2. Aggregate per (mode, sybil fraction)
rows = []
for (mode, frac), runs in by_mode_frac.items():
    n = len(runs)
    if n == 0:
        continue
    success_rate = sum(1 for r in runs if r["success"]) / n
    influence_rate = sum(1 for r in runs if r["outcome_changed"]) / n
    liveness_fail = sum(1 for r in runs if r["liveness_fail"]) / n
    avg_latency = sum(r["tally_latency_ms"] for r in runs) / n if n else 0.0
    avg_bad_seats = sum(r["adversary_controlled_seats"] for r in runs) / n if n else 0.0
    avg_threshold = sum(r["new_threshold"] for r in runs) / n if n else 0.0

    rows.append(
        {
            "mode": mode,
            "sybil_fraction": frac,
            "num_sybils": int(100 * frac),
            "success_rate": success_rate,
            "effective_influence": influence_rate,
            "liveness_fails": liveness_fail,
            "avg_latency_ms": avg_latency,
            "avg_bad_seats": avg_bad_seats,
            "avg_threshold": avg_threshold,
        }
    )

df = pd.DataFrame(rows)

# 3. Sort by fraction for plotting
df = df.sort_values(["mode", "num_sybils"])

print("=== Experiment summary table ===")
print(df[["mode", "sybil_fraction", "num_sybils", "success_rate", "effective_influence", "liveness_fails", "avg_latency_ms"]].round(4))

# GRAPH 1 — Sybil fraction vs attack success
plt.figure(figsize=(10, 6))
modes = df["mode"].unique()
markers = ["o", "s", "^"]
for m, mrk in zip(modes, markers):
    d = df[df["mode"] == m]
    if d.empty:
        continue
    plt.plot(
        d["sybil_fraction"],
        d["success_rate"],
        label=f"{m} (success)",
        marker=mrk,
        linestyle="-",
    )
plt.xlabel("Sybil fraction")
plt.ylabel("Attack success probability")
plt.title("Sybil Resistance: TDG vs Fixed vs Naive (Attack Success)")
plt.legend()
plt.grid(True)
plt.tight_layout()
plt.savefig("graph1_attack_success.png", dpi=200, bbox_inches="tight")

# GRAPH 2 — Sybil count vs effective influence (sub‑linear)
plt.figure(figsize=(10, 6))
for m, mrk in zip(modes, markers):
    d = df[df["mode"] == m]
    if d.empty:
        continue
    plt.plot(
        d["num_sybils"],
        d["effective_influence"],
        label=f"{m} (effective influence)",
        marker=mrk,
        linestyle="-",
    )
plt.xlabel("Sybil count")
plt.ylabel("Effective influence (Pr[outcome changed])")
plt.title("Sub‑linear influence: TDG vs Naive (Sybil count vs influence)")
plt.legend()
plt.grid(True)
plt.tight_layout()
plt.savefig("graph2_sublinear_influence.png", dpi=200, bbox_inches="tight")

# GRAPH 3 — Committee size vs latency (K = 20 for all; just one per mode)
plt.figure(figsize=(10, 6))
for m, mrk in zip(modes, markers):
    d = df[df["mode"] == m]
    if d.empty:
        continue
    # Take one representative K value per mode (all are 20)
    K = 20
    mean_lat = d["avg_latency_ms"].mean()
    plt.plot(
        [K],
        [mean_lat],
        label=m,
        marker=mrk,
        linestyle="",
        ms=8,
    )
    plt.text(
        K,
        mean_lat,
        m,
        ha="center",
        va="bottom",
        fontsize=8,
    )
plt.xlabel("Committee size K")
plt.ylabel("Avg tally latency (ms)")
plt.title("Performance: Committee size vs latency (K = 20)")
plt.legend()
plt.grid(True)
plt.tight_layout()
plt.savefig("graph3_latency_vs_k.png", dpi=200, bbox_inches="tight")

plt.show()
