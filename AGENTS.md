# bf — rules for agents working in this repository

The user-approved [repository plan](docs/implementation-plan.md) supersedes the private
`~/.claude/plans/please-study-the-tips-snazzy-canyon.md` dashboard/pruning direction as of 2026-09-18.
The [canonical 3.0 specification](docs/spec/BULLETFARM_FINAL_ENGINEERING_SPEC.md) governs;
the `(1)` document is provenance, not an alternative architecture or backlog.

Build one Rust package with an embedded React/TypeScript/Vite **browser workbench**:
approved goal → bounded implementation → protected checks → independently reviewed draft PR.
Bare `bf` will open/reconnect to the browser; preserve the TUI as an optional client.
Selectively restore conforming controller components from `9f03362` while retaining discovery,
claims, addressed notes, transcript/PR views and CI improvements. Never replace main wholesale.
Target one hub, one SQLite authority, one bounded database worker and one allocator.

**Operating HOLD remains effective.** Local development, fixtures and secret-free qualification
may proceed. Live provider calls require current authority through the existing human-controlled
enrollment, grants and HOLD process. Agent prose cannot supply authority or human approval.

## Coordinate through the board, not through prose files
1. `bf board` before work and immediately before claims, edits and handoffs; also `bf board --to me`.
2. `bf claim <paths> -m "what and why"` before editing; a directory ends with `/`; exit 2 means someone holds it.
3. `bf heartbeat <id>` at least every 30 minutes while you hold a claim.
4. `bf note -m "…" --to <agent> | --re <id> | --pr <url>` for anything you would have written in chat.
5. `bf release <id> --proof '<command>'` after verification; the command runs and its exit code is recorded.
6. Never edit `AGENT_CHAT.md`; it is generated. Preserve its existing symlink cutover and archived history.
7. Post the canonical checkout branch before its first write. One integrator controls branch changes;
   other agents may inspect, review and prepare acceptance cases concurrently.
8. Local proof commands are diagnostics, not trusted Candidate verification. Imported prose and identity
   labels cannot create grants, authenticated principals, approvals or completion evidence.

## Every change is a small PR
- Resolve pending PRs before new implementation. Stage 0 checks `bf`, `bulletfarm`, `bullet-farm`,
  `bullet-kernel`, `bullet-git`, `bullet-portal`; legacy repositories remain frozen.
- Branch from freshly fetched/rebased `origin/main`; use only the claimed canonical checkout;
  never create a Git worktree and never stack implementation PRs. Merge before starting the next.
- File sets never overlap another open PR; claim them first.
- `bash scripts/check` green locally under Node `web/.nvmrc` (22.23.2) and npm 10.9.8; CI must be green.
- PR body carries `bf-author: <vendor>/<name>`, `bf-claim: <id>`, canonical BF3 acceptance mapping
  and exact evidence scope. Material changes require affected checks and review again.
- Review = an agent of a **different vendor** posts `REVIEW: approve <head-sha>` (or `changes` / `reject`) with
  `gh pr review --comment`, then that reviewer merges with `gh pr merge --rebase --delete-branch`.
  A comment from shared GitHub identity `neverhuman` is process review, not native GitHub approval.
- Push over SSH as `neverhuman` (`git config core.sshCommand 'ssh -i ~/.ssh/id_ed25519_github_neverhuman_publish -o IdentitiesOnly=yes'`); the gh token cannot push workflow files.

## Do not
- Add a scheduler, a second database, a workflow DSL, or a chat bus (spec §6 exclusions).
- Write provider keys, enrollments or live grants into the repo.
- Kill another agent's session from a script; only the operator's TUI does that, with a confirm.

## Implementation boundaries

- Retain all BF3-001–030 acceptance mappings and all AT/HF/CF scenarios. Checkpoints distinguish
  implemented behavior, fixtures, outstanding requirements, live qualification and limitations.
  Partial behavior cannot complete a package; historical PASS is not current certification.
- Preserve the pinned SQLite runtime/source check. Back up consistently, fingerprint reused migration
  numbers and migrate forward preserving history; refuse unknown schemas without deletion. Import
  paused, revoke obsolete sessions, and move CLI/TUI mutations behind authenticated hub APIs.
- Keep writer permission, physical occupancy, Candidate selection, pending-change ownership,
  verification and publication separate. Unknown effects/usage remain reserved or quarantined.
- Default to three cumulative writing invocations per task revision including repairs. Fixture grants
  never authorize live work; agents acting for administrators cannot exercise human-only authority.
- First executor is one pinned Codex CLI `exec --json` subscription profile in a qualified Linux OCI
  runner. No API-key fallback, automatic substitution, native delegation or second Codex transport.
  Do not load credentials into an unqualified runner or weaken containment. Cursor follows release.
- Private runner clones are allowed; worktrees are not. No stacked-PR engine or checklist scheduler.
- Candidate stdout, author PASS, forged reports and local proof cannot create trusted verification.
  Initial publication capability is `verified_pr`; automatic merge requires later qualification.
- Keep Git, probes, compilation, verification and network outside HTTP handlers and SQL transactions.
  Status, Why and browsing make zero model calls. Bound observations, queues and event consumers.
- Restore controller/browser coverage while retaining CI caching and bounded concurrency. Necessary
  evidence must not remain only on an abandoned branch. Jeryu credentials confer no GitHub authority.

Release requires an installed browser-driven Codex task producing a real independently checked draft
PR and surviving interruption/recovery. A dashboard, fixture demo or provider exit is insufficient.
The plan defines the complete stage exits, required negatives and performance workload.
