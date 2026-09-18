# Qualification boundaries

This implementation repairs the original prototype and supplies a persistent fixture workbench. Its first-release gate remains **not passed**: no real browser-driven task has reached an independently reviewed real draft PR.

The fixture verifier executes only byte-identical embedded trusted corpus members. Unknown candidate source, early-exit payloads and forged completion output are refused before execution. Required test identities, actual assertion counts and completion are checked for the trusted corpus. This deliberately restricted lane is not a sandbox or a generic verifier.

The installed Codex CLI reports `codex-cli 0.154.0`. The selected future transport is noninteractive `codex exec --json` with fresh invocation context, explicit model/profile identity, subscription authentication, bounded output, and no resume or API-key fallback. Read-only installation/login probes neither validate an active subscription remotely nor grant execution. The [official noninteractive documentation](https://learn.chatgpt.com/docs/non-interactive-mode) describes JSONL and noninteractive operation; the installed binary help was also inspected. No live model was invoked for this qualification.

A Linux namespace probe using `bwrap --unshare-all` failed on this host with `Failed RTM_NEWADDR: Operation not permitted`. Live arbitrary candidate execution remains unavailable. Filesystem, credentials, inherited descriptors, network destinations, process descendants and resource limits must be tested in the exact disposable runner before enabling it. Existing Operating HOLD, account permissions and live-grant requirements remain unchanged.

SQLite is pinned through `rusqlite = 0.40.2` and the locked bundled `libsqlite3-sys 0.38.2`. Startup checks exact SQLite 3.53.2 and its source identity. That version includes the [documented WAL-reset fix](https://sqlite.org/wal.html#walreset). A bounded worker owns the one hub connection; the OS lock prevents another hub authority. Existing schema-001 rows remain, with old sessions revoked and all migrated grants restricted to fixtures.

Local browser transport is explicitly IPv4 loopback HTTP, protected by a private bootstrap secret, bearer sessions, Host/Origin checks and no cookie authentication. Remote/TLS identity and authenticated runner enrollment remain unimplemented. Do not interpret the local bootstrap as general team authentication.

Fake publication persists intent before dispatch, stores remote state separately, and reconciles interruption at five boundaries in a fresh hub. Those tests do not certify GitHub's remote consistency or permissions. Live branch/PR publication, CI-only preliminary drafts, independent review and unknown-outcome reconciliation require their own qualification.

The canonical backlog, all package acceptance IDs, scenario links and explicit G2/G3 destinations remain in `BUILD_CHECKPOINT.json`. Fixture fragments are not completed BF3 packages or live certificates.
