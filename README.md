# Post-Quantum MPC Simulation Workspace

A Rust workspace for experimenting with a post-quantum, MPC-inspired protocol model and evaluating it with a simulation harness. The repository also includes Python analysis and plotting scripts plus saved charts for comparing attack success, sublinear influence, and latency against a parameter named `k`.

> **Status:** Research / prototype. The implementation and figures are experimental artifacts, not a production-ready cryptographic protocol or blockchain node. Do not use it to protect sensitive data without a separate cryptographic review.

## Overview

The workspace separates protocol-related logic, node execution, simulation, metrics, and shared data types into Rust crates. The `sim-harness` crate drives experiments and emits data that the Python scripts process into summaries and figures. The repository also contains directories named `substrate-node` and `openfhe-development`, indicating supporting or exploratory work around a blockchain node environment and homomorphic-encryption development.

## Workspace crates

| Crate | Purpose |
| --- | --- |
| `crates/pq-mpc-core` | Core protocol-oriented functionality, including a VRF-related module. |
| `crates/mpc-node` | Minimal executable node crate that depends on the workspace protocol components. |
| `crates/sim-harness` | Simulation executable for running experiments and collecting outcomes. |
| `crates/metrics` | Shared metrics-related code. |
| `crates/types` | Shared types used across the workspace. |

The root `Cargo.toml` defines the Rust workspace, while `Cargo.lock` records resolved dependency versions for reproducible builds.

## What the simulation studies

The included experiment and chart names indicate analyses of:

- Attack-success behavior, represented by `graph1_attack_success.png`.
- Sublinear influence behavior, represented by `graph2_sublinear_influence.png`.
- Latency as a function of `k`, represented by `graph3_latency_vs_k.png`.

The exact outcome depends on the simulation parameters and implementation in `crates/sim-harness/src/main.rs`; interpret figures together with the corresponding source and generated data rather than as general security guarantees.

## Requirements

- A current stable Rust toolchain installed through `rustup`, including Cargo.
- Python 3 for the analysis and plotting scripts.
- The Python packages imported by the analysis scripts, as applicable to the scripts’ implementation.
- Sufficient disk space for Cargo build outputs; the repository currently includes a `target/` directory, but it is generated and is not needed to build from source.

## Build and run

Clone the repository and build every crate in the workspace:

```bash
git clone https://github.com/Verlopat/007.git
cd 007

cargo build --workspace
cargo test --workspace
```

Run the simulation harness with Cargo:

```bash
cargo run -p sim-harness
```

Run the node executable, if needed for the experiment you are performing:

```bash
cargo run -p mpc-node
```

Use `cargo run -p <package> -- --help` if a crate exposes command-line options. Review the source for available parameters before treating a run as a reproducible experiment.

## Analysis and plotting

The repository includes three Python utilities:

| Script | Intended role |
| --- | --- |
| `analyze.py` | Processes or summarizes simulation output. |
| `compare_all.py` | Runs or compares a set of experiment configurations. |
| `plot_sims.py` | Produces plots from experiment data. |

Run a script from the repository root after installing its dependencies and ensuring its expected inputs are present:

```bash
python3 analyze.py
python3 compare_all.py
python3 plot_sims.py
```

The saved PNG files and `plots.png` are existing visualization artifacts. Regenerate them after changing simulation assumptions, source code, or data.

## Repository layout

```text
.
├── Cargo.toml                    # Rust workspace definition
├── Cargo.lock                    # Locked Rust dependencies
├── crates/
│   ├── pq-mpc-core/              # Protocol-oriented core and VRF module
│   ├── mpc-node/                 # Node executable
│   ├── sim-harness/              # Experiment/simulation executable
│   ├── metrics/                  # Metrics support
│   └── types/                    # Shared data types
├── analyze.py                    # Analysis helper
├── compare_all.py                # Experiment comparison helper
├── plot_sims.py                  # Plotting helper
├── graph1_attack_success.png
├── graph2_sublinear_influence.png
├── graph3_latency_vs_k.png
├── plots.png                     # Combined/saved plot artifact
├── output/                       # Generated experiment output
├── substrate-node/               # Supporting node-related material
├── openfhe-development/          # Exploratory OpenFHE-related material
└── target/                       # Generated Cargo build output
```

## Reproducibility notes

- Preserve the Rust toolchain version, `Cargo.lock`, simulation inputs, and random seeds when comparing runs.
- Keep generated output separate from source when possible, and record the exact command used to generate each figure.
- Treat `target/`, `output/`, and generated PNG files as build or experiment artifacts unless they are intentionally curated results.
- The repository currently contains generated build output; normally, `target/` should be excluded through `.gitignore`.

## Security limitations

- Code that references post-quantum primitives, MPC, VRFs, homomorphic encryption, or blockchain nodes is not automatically secure by virtue of those labels. Security depends on precise algorithms, parameters, implementation correctness, threat model, and review.
- Simulation results measure behavior of the modeled system, not security of a deployed network.
- Do not make cryptographic security or performance claims beyond what the documented experiments, assumptions, and independent review support.

## Suggested improvements

1. Add a formal threat model, protocol specification, and explicit security assumptions.
2. Document every simulation parameter, input distribution, random seed, and output metric.
3. Add CLI options or configuration files for experiments instead of hard-coding scenarios.
4. Add unit, integration, and property tests for the core protocol and metrics.
5. Provide a Python dependency file such as `requirements.txt` or `pyproject.toml`.
6. Remove generated `target/` content and other transient artifacts from version control; add appropriate `.gitignore` rules.
7. Add benchmark automation and CI that verifies builds, tests, and plot regeneration.
8. Add an explicit license and contribution guidelines.

## License

No license file is currently included. Add an explicit license before distributing or accepting external contributions.
