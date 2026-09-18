# bf

One Rust hub and an embedded React workbench for taking an approved goal through bounded work,
protected checks and an independently reviewed draft PR.

The [implementation plan](docs/implementation-plan.md) governs development against the
[canonical 3.0 specification](docs/spec/BULLETFARM_FINAL_ENGINEERING_SPEC.md).
The browser is the primary interface being built; the existing TUI remains an optional client.

```bash
cargo build --locked --release --bin bf
./target/release/bf doctor
./target/release/bf web
```

Current main discovers local Claude, Codex, Cursor and Grok sessions and provides claims,
addressed notes, transcript tails and PR views. Bare `bf` currently opens the TUI. The delivery
controller preserved at `9f03362` is being selectively restored; the browser's delivery routes
are not yet functional. Unified persistence, managed live execution, independent verification
and GitHub publication are **not yet qualified**. `bf run` is a stub.
The Operating HOLD remains effective; fixture success does not authorize live work.

The hub binds IPv4 loopback. Bootstrap material is in private `endpoint.json`. Do not expose it through a proxy.
The planned installed client will manage private SSH forwarding and reconnect. Production web
assets are embedded in the binary; installed use does not require Node.

```bash
bash scripts/check
```

`scripts/check` requires Node `22.23.2` (see `web/.nvmrc`) and npm `10.9.8`.
