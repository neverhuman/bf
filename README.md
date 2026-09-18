# bf

One Rust hub. `bf doctor` inspects it. `bf serve` / `bf web` serve the embedded React page on loopback.

```bash
cargo build --locked --release --bin bf
./target/release/bf doctor
./target/release/bf web
```

This is the pruned core after landing the fixture workbench: sessions, idempotent commands, SQLite. It does **not** yet discover live Claude/Codex/Cursor/Grok sessions, replace `AGENT_CHAT.md`, or open a TUI. Those are the next stacked PRs.

The hub binds IPv4 loopback. Bootstrap material is in private `endpoint.json`. Do not expose it through a proxy.

```bash
bash scripts/check
```

`scripts/check` requires Node `22.23.2` (see `web/.nvmrc`) and npm `10.9.8`.
