import json
import matplotlib.pyplot as plt
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
        "avg_bad_seats": (sum(r["adversary_controlled_seats"] for r in rows) / n) if n else 0.0,
        "avg_new_threshold": (sum(r["new_threshold"] for r in rows) / n) if n else 0.0,
    }

files = sorted(Path("output/logs").glob("sim_*.jsonl"))
files = [f for f in files if f.name != "sim.jsonl"]  # skip the first 20‑round file

sybils = []
success = []
liveness = []
thresholds = []

for path in files:
    name = path.name
    frac = float(name.replace("sim_", "").replace(".jsonl", ""))
    sybils.append(frac)
    rows = load_jsonl(path)
    s = summarize(rows)
    success.append(s["success_rate"])
    liveness.append(s["liveness_fail_rate"])
    thresholds.append(s["avg_new_threshold"])

plt.figure(figsize=(10, 7))

plt.subplot(2, 2, 1)
plt.plot(sybils, success, "o-", label="Attack success rate")
plt.xlabel("Sybil fraction")
plt.ylabel("Success rate")
plt.title("Attack success vs Sybil fraction")
plt.grid(True)

plt.subplot(2, 2, 2)
plt.plot(sybils, liveness, "s-", label="Liveness fail rate", color="orange")
plt.xlabel("Sybil fraction")
plt.ylabel("Failure rate")
plt.title("Liveness failures vs Sybil fraction")
plt.grid(True)

plt.subplot(2, 2, 3)
plt.plot(sybils, thresholds, "^-", label="Avg threshold", color="green")
plt.xlabel("Sybil fraction")
plt.ylabel("Threshold")
plt.title("Adaptive threshold (t) vs Sybil fraction")
plt.grid(True)

plt.subplot(2, 2, 4)
plt.plot(sybils, [s["avg_bad_seats"] for s in [summarize(load_jsonl(f)) for f in files]], "d-", color="purple")
plt.xlabel("Sybil fraction")
plt.ylabel("Bad seats (avg)")
plt.title("Adversary committee seats vs Sybil fraction")
plt.grid(True)

plt.tight_layout()
plt.savefig("plots.png", dpi=200, bbox_inches="tight")
plt.show()
