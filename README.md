# bf

Ask. Inspect. Steer. Take over.

One binary, one local hub, one page. A person types a goal. A bounded worker proposes code. Independent checks decide whether it is real. A draft PR is the product, not a provider terminal.

```bash
git clone https://github.com/neverhuman/bf
cd bf
cargo run --locked --bin bf
```

That opens the workbench. No family lock. No installer. No subcommand catalog.

## Demo (no keys, no network)

```bash
cargo run --locked --bin bf -- demo --fixture basic
cargo run --locked --bin bf -- demo --fixture interrupted_publish
bash scripts/check
```

The demo uses a fake executor and a fake forge. A PASS here is orchestration, not model quality.

## What you do

| You | It |
| --- | --- |
| type a goal | one mission, one task, `pr_ready` |
| watch the page | status with zero model calls |
| wait | sealed candidate → independent check → draft PR |
| take / stop | you keep the change; unknown effects stay unknown |

## Not this release

Two-person fairness, a second coding provider, production deploys, Slack, and the old four-repo family. Those wait until this loop is boringly solid.

Spec: BulletFarm 3.0 (`tips/BULLETFARM_FINAL_ENGINEERING_SPEC.md` in the family container).
