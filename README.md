# bf

`bf` opens or reconnects to a private local workbench. It remembers the current Git project, saved goals, conversations and plan revisions. The built React workbench is embedded in the Rust binary; running it does not require Node, a source checkout or a build directory.

```bash
cargo build --locked --release --bin bf
./target/release/bf
```

The implemented execution lane is a **deterministic fixture demo**. Choose **Try the demo → Review plan → Start work**. The same controller admits the planner, implementation and verifier jobs, then creates a recoverable simulated draft PR. One acceptance binds the exact plan revision. The demo is separate from your repository and account.

Real project goals are saved with a visible qualification block. **This is not the first usable live release:** Codex transport, Linux containment, live independent verification and real PR publication still need implementation and qualification. The Operating HOLD remains effective. See [BUILD_CHECKPOINT.json](BUILD_CHECKPOINT.json) for the complete canonical BF3 mapping and remaining requirements.

```bash
bf demo --fixture basic
bf demo --fixture interrupted_publish
bf doctor
```

Demo data is retained in the hub directory, including source repositories, job workspaces, Git bundles, check receipts and the separate fake forge. Repeating a demo preserves those records. Fixture grants cannot authorize live work. Takeover returns an explicit unsupported error; Stop revokes work and retains occupancy until termination is observed.

The hub binds IPv4 loopback on an automatically assigned port. Its bootstrap credential is in private `endpoint.json`; the launcher exchanges it for a revocable browser session. Browser session tokens are removed from the URL fragment immediately. Never expose this loopback pilot through a network proxy; remote/TLS identity is not implemented.

Development checks:

```bash
bash scripts/check
cargo build --locked --release --bin bf
python3 scripts/benchmark.py
```

`bash scripts/build-web` updates the checked-in embedded bundle after UI changes. `bash scripts/generate-contracts` updates the Rust-derived command types. `scripts/check` verifies both generated inputs, Rust checks, browser journeys and both demos. It installs the pinned Playwright browser for development only.

The performance workload uses 10,000 historical task projections, four simulated runner status clients, 20 status reads and 10 durable mutations per second. Raw distributions and editor measurements are written to `evidence/performance.json`; these measurements do not certify real executor performance.

Canonical specification: `tips/BULLETFARM_FINAL_ENGINEERING_SPEC.md` in the family container. The `(1)` file is provenance only. Older repositories and evidence are preserved.
