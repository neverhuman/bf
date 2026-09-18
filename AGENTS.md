# bf — rules for agents working in this repository

`bf` is the lean BulletFarm: one Rust binary, a no-argument TUI, a SQLite coordination board, and a
thin React mirror. Plan of record: `~/.claude/plans/please-study-the-tips-snazzy-canyon.md` on the host.

## Coordinate through the board, not through prose files
1. `bf board` before you start (and `bf board --to me` for notes addressed to you).
2. `bf claim <paths> -m "what and why"` before editing; a directory ends with `/`; exit 2 means someone holds it.
3. `bf heartbeat <id>` at least every 30 minutes while you hold a claim.
4. `bf note -m "…" --to <agent> | --re <id> | --pr <url>` for anything you would have written in chat.
5. `bf release <id> --proof '<command>'` after verification; the command runs and its exit code is recorded.
6. Never edit `AGENT_CHAT.md`; it is generated. (Until `bf board` lands, use it by hand under the same rules.)

## Every change is a small PR
- Branch from a freshly fetched `origin/main`; never a worktree; never stack PRs.
- File sets never overlap another open PR; claim them first.
- `bash scripts/check` green locally under Node `web/.nvmrc` (22.23.2) and npm 10.9.8; CI must be green.
- PR body carries `bf-author: <vendor>/<name>` and `bf-claim: <id>`.
- Review = an agent of a **different vendor** posts `REVIEW: approve <head-sha>` (or `changes` / `reject`) with
  `gh pr review --comment`, then that reviewer merges with `gh pr merge --rebase --delete-branch`.
- Push over SSH as `neverhuman` (`git config core.sshCommand 'ssh -i ~/.ssh/id_ed25519_github_neverhuman_publish -o IdentitiesOnly=yes'`); the gh token cannot push workflow files.

## Do not
- Add a scheduler, a second database, a workflow DSL, or a chat bus (spec §6 exclusions).
- Write provider keys, enrollments or live grants into the repo.
- Kill another agent's session from a script; only the operator's TUI does that, with a confirm.
