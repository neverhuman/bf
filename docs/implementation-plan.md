# bf: one workbench, one controller, one verified delivery path

User-approved implementation direction, 2026-09-18. This repository plan supersedes
`~/.claude/plans/please-study-the-tips-snazzy-canyon.md` and its dashboard/pruning direction.
Source of intent: the implementation plan supplied by the user, not agent-created authority.

### Current user instruction supersedes the earlier TUI-first decision

The implementing Codex session received the following direct user instruction on 2026-09-18,
after the earlier TUI-first campaign. These are quotations from that message and its supplied plan:

> A previous agent produced the plan below to accomplish the user's task. Implement the plan in a fresh context. Treat the plan as the source of user intent, re-read files as needed, and carry the work through implementation and verification.

> **Primary interface:** browser workbench; bare `bf` opens or reconnects to it.

> **Existing TUI:** preserve as an optional client of the same API.

> This plan supersedes the direction that reduces the MVP to a session dashboard and deletes the delivery core.

> Recover conforming components from `9f03362` through selective integration, preserving subsequent discovery, coordination, TUI and CI improvements.

> Move CLI and TUI mutations to the authenticated hub API. They must not open a second writable authority.

These explicit directions replace the earlier private plan's TUI-first/read-only-browser and pruning
decisions. Reviewers should assess this implementation against the current request. This does not
change the HOLD or confer live grants. Stage 0 changes documentation only; controller restoration and
coordination migration remain subsequent, serialized implementation PRs with their own checks/reviews.

## 1. Audit and decisions

Restore the tested delivery controller, retain discovery/coordination, and finish one real browser-to-PR
journey. The product remains `neverhuman/bf`: one Rust package with embedded React/TypeScript/Vite.
Preserve legacy BulletFarm repositories and their evidence.

The [canonical 3.0 specification](spec/BULLETFARM_FINAL_ENGINEERING_SPEC.md) governs. This unmodified
copy comes from `/home/ubuntu/bullet/tips/BULLETFARM_FINAL_ENGINEERING_SPEC.md`, SHA-256
`1d8064b40041e9383b66b790eb8eaa41924657db5f5d1bb0a5373fc5c501b9eb`.
The `(1)` document is provenance; its architecture/backlog numbering must not become defaults.
`tips` contains three Markdown files; schemas/backlog/validator referenced by `START_HERE.md` are
absent. Recover available artifacts from history and derive missing traceability from the canonical
document. Never report the absent validator as executed.

Original supplied audit baseline: main `6e8d0bf`, inspected September 18, 2026. Current Stage 0
implementation base: main `0cc243a`, including PR #11's transcript/PR views. Earlier controller: `9f03362`.

| Observed state | Consequence |
|---|---|
| Four-provider discovery, TUI, claims, notes and PR listing | Retain useful functionality |
| Delivery controller removed by simplification | Recover selectively through reviewed changes |
| Browser calls absent project/draft/work/event routes | Restore browser integration and coverage |
| Hub `hub.sqlite`, direct coordination `bf.sqlite` | Migrate to one authority |
| Replay lacks actor scope; command/operation inserts not atomic | Restore authorization/atomicity before live execution |
| Startup mainly checks migration-table existence | Explicit preserving migration of actual variants |
| Current `bf run` stub; earlier execution was fixtures | No qualified live coding path |
| Stop signals group but checks selected PID disappearance | Prove complete managed-job termination |
| Verifier only executes trusted fixture programs | Regression evidence, not general verification |
| Durable fake forge in history | Real recoverable GitHub adapter unfinished |
| Both snapshots pass different 48-test Rust suites | Counts/green CI do not establish browser delivery |
| Required `check` and current base; no native approval requirement | Independent review is a recorded process obligation |

At 17:23 UTC all six queues were empty, then PR #11 opened. Stage 0 observed it merged as
`0cc243ab26cf7c584564b1aba7e5eb0160a0da5d` after Grok review of exact head
`ebbb589243283c7b9f2bda3e906bbcfd712a8ec3` and hosted `check` SUCCESS. All six queues were empty
again before this plan's first edit.

Established decisions: browser primary; bare `bf` opens/reconnects; laptop connects privately to an
independent Ubuntu hub through saved SSH; one pinned Codex subscription transport first; first repository
`neverhuman/bf`; Cursor second; TUI optional on shared API. Product boundary is approved goal → bounded
implementation → protected checks → independently reviewed draft PR. Operating HOLD stays effective;
local development/fixtures/qualification continue, live execution waits for current human-controlled authority.

## 2. Product journey

Typing `bf` opens installed workbench, reconnects selected hub, restores project/conversation/draft/work
selection, and shows actionable connection/account problems without a provider terminal. First launch
offers local use or saved SSH host. Use existing SSH configuration and host-key verification; users do
not manage ports, bearer tokens or mission IDs. Hub outlives launcher/laptop/browser; return displays
reconciled state. Embed production assets: no source checkout, Node or build directory needed at runtime.

Four compact views share one project selector: **Work** (goal, plan, tasks, next actions), **Agents**
(managed jobs and observed sessions), **Inbox** (decisions, findings, notes, handoffs), **Changes**
(Candidate diff, checks, draft PR, integration). A drawer holds requirements/activity/changes/checks/
limits/controls; protocol traces and codes stay under Details.

Journey: Describe goal → Review plan → Start work → Follow changes/checks → Open verified draft PR.
Review shows objective, acceptance, source base, scope, account/model, invocation limits, deadline,
checks and destination; advanced settings collapsed. One approval binds exact revision plus bounded
repairs. Expanded scope, higher limits, new permissions or destination create a specific visible decision.

Separate provider installation/version, local authentication, observed auth/quota failures, qualified
profile and eligibility under grants/limits/HOLD. Observed Cursor/Grok remains useful before managed
execution. Deduplicate wrappers/children by observed session identity; combine PID with process start
identity. Unknown stays Unknown; OS sleep does not prove model activity. External sessions stay
observational until supported explicit adoption; unsupported actions explain their limitation.

Move claims/notes behind hub: claims bind repository, paths, authenticated agent, purpose, expiry and
work; multi-resource acquisition atomic. Notes reference work/claims/findings/PRs. Handoffs include
exact revision/checks/blockers. Local proof is diagnostic, never trusted Candidate evidence. Preserve
generated `AGENT_CHAT.md`, archived history and existing symlink cutover; do not repeat it. Imported
prose cannot create grants, human approvals or completion evidence. Keyboard navigation, multiline/
Unicode editing, visible focus, announcements and narrow layouts are release requirements.

## 3. Serialized implementation checkpoints

Split large stages into small PRs retaining their acceptance boundary. One integrator; independent
reviewers may inspect and prepare acceptance concurrently. Serialize implementation PRs and shared
checkout branch changes. Never create Git worktrees.

### Stage 0 — Resolve queue and establish one plan

Read board/instructions. Inventory pending PRs across `bf`, `bulletfarm`, `bullet-farm`, `bullet-kernel`,
`bullet-git`, `bullet-portal`. Review exact heads; fix/merge valid work or close rejected work with
reasons. Confirm resolved queue, fetch/rebase current `origin/main`, claim canonical checkout before
edits. Commit agreed plan and update instructions: private home-directory plans cannot govern repo.

Exit: resolved starting queue, shared direction, explicit claims, clean current base.

### Stage 1 — Restore controller and regression coverage

Selectively recover conforming `9f03362` components preserving discovery/coordination/TUI/CI:
domain contracts and semantic validation; atomic commands/authorization/operation lookup; durable jobs,
reservations/accounting/reconciliation; Candidate verification boundaries; publication intents and
persistent fake forge; browser routes/journeys/fault tests; qualification records and `BUILD_CHECKPOINT.json`.
Do not reset main or replace checkout wholesale. Each checkpoint distinguishes implemented behavior,
fixture evidence, outstanding requirements, live qualification and limitations. Preserve every BF3
acceptance mapping and AT/HF/CF scenario; partial implementation never completes a whole package.
Historical evidence keeps its source identity and cannot certify changed code.

Exit: restored browser fixture journey/negatives, retained later features.

### Stage 2 — Storage, authorization and contracts

One hub authority/local SQLite store/bounded worker plus immutable artifacts. Preserve pinned fixed
SQLite runtime/startup source check: inspected bundled 3.53.2 contains the WAL-reset fix documented in
3.51.3 and later (https://sqlite.org/wal.html#walreset).

Recognize original controller, reduced hub and coordination schemas. Quiesce writers, consistently
back up, fingerprint actual schemas including reused migration numbers, migrate forward/import with
provenance. Preserve historical identifiers/notes/evidence/artifacts, reconcile claims without
authenticating labels, revoke obsolete sessions and keep restored execution paused. Refuse unknown
schemas without deleting data. Schema recognition must precede connecting incompatible restored tables.
CLI/TUI mutations use authenticated hub API, never another writable authority.

| Interface | Required behavior |
|---|---|
| `POST /v3/commands` | Typed command, exact raw bytes, command ID, target, expected version |
| `GET /v3/operations/{id}` | Requested durable operation scoped to current authorization |
| Project/mission/draft/work reads | Owner-scoped, typed, paginated |
| Agent/board/inbox/PR reads | Bounded projections with freshness |
| Event stream | Bounded updates, reconnect cursor, snapshot recovery |
| Runner endpoints | Authenticated poll/heartbeat/events/artifacts/results bound to generation |

Generate Rust/TypeScript commands **and responses**; validate structure/semantics. Reject duplicate keys,
invalid UTF-8, forbidden unknown fields, unsafe integers, stale versions and invalid references. Preserve
exact received bytes for idempotency, never hash reserialization. Atomically authorize/check versions/
mutate/audit/create operations; historical replay rechecks current access. Private bootstrap, expiring
revocable sessions, strict Host/Origin, no anonymous demo-owner fallback. Agents acting for admins
remain distinct from human-only authority.

Exit: single authority survives migration/concurrency/revocation/storage failure/restart without losing
acknowledged state.

### Stage 3 — Real controller drives fixtures

Adapters use production job/effect paths. Inputs are admitted repository/owner/task/profile/grant/limits;
no default demo owner/grants/task. Before launch persist job identity/purpose/generation/deadline,
complete reservations, budget holds, protected verification/review reserve, events and launch intent.
Implement dependency readiness/full graph activation/current grants: unknown/deleted/cancelled/invalid
prerequisites never mean completed. Fake forge survives hub memory; demos retain distinct run directories.
Separate write permission, physical occupancy, selection, pending-change ownership, verification and
publication. Pause stops admission; Stop remains Stopping until trusted termination proof. Cancellation
fences new work and reconciles in-flight effects. Ambiguous spawns stay reserved/quarantined; retries
cannot create another accepted incarnation. Default three cumulative writing invocations per task
revision including repairs; bounded transport retries only when safely repeatable.

Exit: same controller handles normal/interrupted/cancelled/ambiguous fixture journeys.

### Stage 4 — Browser planning and acceptance

Persist owner-bound conversations/revisioned drafts. Planning is a bounded mission job with common
accounting/recovery. Canonical Fast mode first under standing bounded planning grant; goal submission
does not authorize implementation. Persist requirements/acceptance/base/scope/checks/full graph;
structural admission finishes before Start work. Accept one exact revision once: duplicate/reconnecting/
concurrent clients get same operation or precise stale conflict. Fresh implementation context includes
accepted requirements/source/decisions/checks/remaining limits, not planner mutable conversation. Restore
draft without overwriting newer client edits. Status/Why/accounts/monitoring make zero model calls.
First live pilot one coherent task/PR; dependencies via fixtures, no stacked-PR or checklist scheduler.

Exit: whole fixture journey through browser without IDs/raw JSON/provider terminal.

### Stage 5 — Qualify isolated Codex execution

Pin reviewed Codex CLI `exec --json` executable/configuration with existing subscription authentication.
Audit saw 0.154.0 → installed 0.155.0 drift; name actual tested binary. No API-key fallback, account pools,
second transport or automatic substitution. Recheck installed behavior and official noninteractive and
permissions documentation during qualification.

One Linux OCI profile under trusted supervisor: private filesystem/Git/temp/caches; bounded CPU/memory/
process/storage/output/wall time; no host home/hub DB/publisher secrets/privileged sockets; pinned command
sandbox restricts reads/writes; command network disabled; separate provider service connectivity; native
delegation/unapproved hooks/MCP/connectors/browser tools disabled. Trusted controller may access
subscription auth, candidate commands cannot. Explicitly test paths/env/FDs/`/proc`/sockets/alternate
tools: permission profiles do not constrain every tool surface.

Earlier namespace probe failed. Qualify containment without secrets before loading credentials. If
boundary cannot be enforced, refuse before model spawn and identify environment need. Use compatible
Linux host with same profile, never weaken boundary/add another executor design. Stop descendants/
daemons by incarnation, not parent PID. Restart finds survivors before freeing capacity. No live call
until existing human-controlled HOLD/enrollment/grants process authorizes it.

Exit: exact profile passes isolation/freshness/framing/cancellation/accounting/recovery qualification.

### Stage 6 — General verification and recoverable GitHub publication

Seal Candidate against admitted base: complete diff/content IDs/modes/artifacts; reject out-of-scope,
unsafe paths, missing bytes or substituted bases. Protected checks use immutable verifier-controlled
inputs under separate authority. For `bf`, authoritative behavioral test is external browser harness:
candidate app sandboxed, protected harness/reporter outside. Require expected discovery, required
cases/assertions executing, completion, exact Candidate/recipe, correct producer/generation. Stdout,
forged JUnit, exit status and author PASS cannot manufacture verdict. Repository test output supports
evidence unless reporting boundary separately qualified.

Real GitHub adapter: durable exact intent before dispatch binds Candidate/selection/grant/checks/
destination/expected remote state; credential-separated import never executes candidate hooks/helpers/
configuration; one stable PR identity per revision; uncertain response reconciled by authoritative
branch/PR reads; retain unresolved outcomes instead of blindly repeating. Hold on human edits/closure.
PR-dependent checks can use preliminary draft after applicable gates; stay Checking until all required
checks/independent review pass. First capability `verified_pr`; automatic merge deferred.

Exit: real draft PR published/checked/recovered without duplicates or premature readiness.

### Stage 7 — Useful self-dogfood task

In `neverhuman/bf`: **Add work-list pagination so users can reach older tasks without losing current
selection or draft.** Freeze acceptance: 60 seeded tasks show first 50; Load more reveals remaining 10
without duplicates; selection/unsent goal survive; project switch resets pagination; keyboard operation/
announcements work; browsing makes no extra model calls; approved UI scope only, protecting authority,
migrations and CI policy. Protected regression fails baseline and passes Candidate.

Run installed laptop client; interrupt managed job, restart hub during recovery, simulate lost publication
response; show accurate workbench recovery. Record exact versions/source/approvals/checks/usage/
uncertainty/elapsed time/human interventions/PR URL. After independent review/current CI, merge pilot
normally; product publication and repository merge are distinct events.

Exit: user completes real journey, inspects independently checked draft PR and recovers interruption.

## 4. CI, verification and release gates

Every PR starts freshly fetched/rebased with claim/author/BF3 mapping, local `scripts/check` and hosted
required checks passing, independent different-vendor exact-head review. Material changes repeat
affected checks/review. Merge before new implementation; retain necessary evidence beyond branches.
Shared-identity comments are not native approvals; Jeryu credentials do not authorize GitHub. Retain
caching/bounded concurrency and restore coverage. Normal CI uses fixtures/disposable services without
provider credentials; live qualification is separate/sanitized/tied to exact source.

| Area | Required negatives and recovery |
|---|---|
| Browser | First launch/return, Unicode/multiline, stale draft, double acceptance, reconnect |
| Authorization | Anonymous, duplicate keys, revoked sessions, cross-owner reads, historical replay |
| Storage | Existing schemas, interrupted migration, disk failure, atomic rollback, concurrency |
| Jobs | Dependency failure, partial reservation, ambiguous spawn, stale generation, restart, exhausted allowance |
| Cancellation | Surviving child, daemon escape, delayed termination, lost supervisor |
| Verification | Missing tests, zero assertions, early exit, forged output, wrong issuer, stale Candidate, partial artifacts |
| Publication | Crashes around dispatch/application/receipt/settlement, lost response, revoked grant, human edits/closure |
| Responsiveness | Slow clients, floods, saturated worker queues, prompt cancellation, durable results |
| Installation | Embedded assets, no checkout/Node, launcher exit, restart, SSH reconnect |
| Discovery | Wrapper dedup, PID reuse, stale sessions, unavailable transcript, unsupported actions |

Frozen release-build performance workload: 10,000 historical tasks, four simulated runners, 20 status
reads/s, 10 durable small mutations/s, slow-client/event-flood variants; warm-up then >=60 seconds steady
state. Targets: status/Why p95 <100 ms; durable command p95 <250 ms; idle RSS <150 MiB; editor p95 <100 ms;
zero monitoring model calls. Record distributions/failures/workload identity/hardware/cold-warm startup.
Old short measurements remain historical. Keep Git/probes/builds/checks/network outside HTTP handlers/
SQL transactions; bounded cached observations, indexed paginated projections, slow-consumer disconnect
with snapshot recovery.

**Release only when installed browser-driven Codex work produces a real independently checked draft PR,
survives interruption/recovery, and passes applicable authority/isolation/verification/publication tests.**
Dashboard, fixture demo and successful provider exit cannot satisfy this gate.

## 5. Full vision and operating defaults

| Gate | Destination |
|---|---|
| G0 — BF3-001–007 | Contracts, authority, budgets, activation, durable kernel, deterministic adapters |
| G1 — BF3-008–013 | Isolation, recovery, certified executor, independent checks, recoverable draft PR |
| Early browser slice | Partial BF3-017/018 for first useful journey, not complete G2 |
| G2 — BF3-014–022 | Two owners/runners/families, fairness, takeover/return/respecification, scoped context, planning modes, integration, assembled acceptance, accounting, restore/export, independent rebuild |
| G3 — BF3-023–030 | Controlled learning, Grok Build, production, Slack/Bot/SMS, local inference, architecture experiments, justified forge improvements |

After release: qualify Cursor through same adapter/isolation contract; record actual model family
(two CLI brands with one family do not satisfy gate); qualify Claude/team workflow; finish TUI on
shared API; qualify Grok Build separately from Bot/unidentified executable; run representative team
pilot retaining failures/costs/human handling. Preserve complete canonical backlog.

Defaults: one package/hub/database/allocator, one active managed coding account initially, one coherent
task per PR/internal checklist, private clones/no worktrees, bounded planning/writing/repair/checks,
unknown usage/effects visible, fixture grants never live, no human authority via agent prose, preserve
legacy/history. Order: queue → restore/unify controller → browser planning → isolation/Codex → verify/
publish → real recovery journey → provider expansion.
