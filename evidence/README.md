# Local implementation evidence

The source baseline is `6923946`. These artifacts describe the local working tree, not hosted CI or a live release. `source-manifest.json` records source file hashes. `BUILD_CHECKPOINT.json` preserves canonical BF3/acceptance mappings and all remaining work.

- `check.log`: complete `bash scripts/check` exit 0, with 48 Rust tests, 1 UI test, 4 Chromium journeys, typechecking, bundle freshness, formatting, Clippy and both persistent demos.
- `release-build.log`: locked optimized binary build. Benchmark binary SHA-256: `8e09d5deb4db12b888ea5f617b23dddba9afef1ed69ac47947c4bb1b802e365c`.
- `independent-review.md`: read-only reviewer findings and final 28-test correction retest.
- `performance.json`: workload `bf-http-10000-v2-painted-editor` passed on this host. All samples and failures are retained. The 10-second workload uses 10,000 historical projections, 200 work reads, 100 durable small commands and 40 polls from four simulated runner status clients. No provider runs are involved.

| Measurement | Observed | Target |
| --- | ---: | ---: |
| Work status p95 | 2.51 ms | <100 ms |
| Durable small command p95 | 15.18 ms | <250 ms |
| Visible editor input to frame p95 | 15.20 ms | <100 ms |
| Idle hub RSS | 9.94 MiB | <150 MiB |
| Model calls for workload | 0 | 0 |

New-database startup was 158.54 ms; existing-database startup was 78.48 ms. These start at process launch and end at successful authenticated bootstrap; they do not flush OS caches. Initial workbench readiness through its first two animation frames was 329.67 ms. Editor samples measure browser input-event to next-frame delay, not physical keyboard-to-display latency.

Two earlier runs (`performance-before-editor-fix.json` and `performance-after-render-memo.json`) failed editor timing. Both started automated typing before the first rendered frame. `editor-control.json` reproduces the startup delay in a plain textarea with no application code: pre-paint p95 146.8 ms versus painted p95 15.3 ms. Workload v2 waits for visible initial frames and records that startup separately; HTTP rates, task count, editor text and typing rate are unchanged. The earlier failure artifacts are preserved, not overwritten.

These results do not satisfy the live first-release gate. Missing live transport, certified arbitrary-code containment, general independent verification/review and real forge publication remain explicit in the checkpoint. Existing Operating HOLD and grant/account restrictions are unchanged.
