# BulletFarm
## Final comparative adjudication and engineering build contract

**Decision baseline 3.0 · September 16, 2026**  
**For:** Jepson Taylor, Alton, engineering owners, reviewers and fresh build agents  
**Specified-design score:** **95.04 / 100**  
**Status:** Proposed build contract. Not an implemented system, code audit, security certification, reliability percentage or measured cost advantage.

> **People own the outcome. Agents do bounded work. Evidence controls delivery.**

**Decision:** Build one small Rust authority layer around replaceable execution, with a Vite/TypeScript/React workbench. Preserve personal and domain ownership under one allocator for overlapping managed work. Start with one useful, independently checked change; add team capabilities before scaling; let reusable lessons and upstream components earn adoption through evidence.

The final design combines the strongest mechanisms, not the union of every feature. The previous 94.78 draft contributes the clearest job/capacity and stage-specific verification rules. The 94.50 rebuild kit contributes concrete contract and test artifacts. The harness-informed plan contributes execution profiles, scoped learning and source-informed reuse. Alton contributes the durable-assets strategy, gate-first operating discipline and first-class human workflows.

**Reading order:** Part I records what each source actually proposes and how it scores. Parts II–IV are the single normative target to build. Part V contains the build backlog and acceptance scenarios. Appendices contain contract examples, reference database structure, preservation decisions and sources. An implementation agent must not merge competing source defaults independently.

**Scope of review:** Ten distinct substantive plans present in this conversation, including the five original alternatives, Alton's vision and four later complete syntheses/drafts. Two decision briefs and the available backlogs are evaluated separately for their intended artifact roles. Identical files and PDF/Markdown representations do not receive additional votes. Earlier V3/FORGE documents referenced within these sources are background, not separately reviewed full-plan entries here.

**Evidence labels:** [A]–[J] identify supplied plans; [K]–[L] identify briefs. Section references point to those source documents. [W1]–[W11] identify selected primary technical documentation and pinned upstream files revisited in this review. Statements marked **Decision** or **Refinement** are this review's recommendations, not claimed agreement by both authors. All source hashes and exact filenames are in `SOURCE_MANIFEST.json`.

**Authority of this package:** The numbered build contract and its version-3 schemas are one target. A schema validates shape, not permission. Reference SQL enforces selected structural constraints, not all controller predicates. A discovered mismatch is a specification defect; stop the affected capability and resolve it through review. Archived source plans are evidence, not competing active instructions.

# Part I — Compare the plans before choosing the implementation

## 1. Inventory, provenance and ranking

The review found a distinct document titled *Personal ownership. Shared evidence.*, self-scored 94.78, in addition to the *Own the outcome* 94.50 rebuild plan. They share a version label but are not identical. Both are scored. The two Alton attachments are byte-identical and count once. The generated early Verified Changes document is identical to input A and also counts once.

Scores below assess **fit and specificity for the requested individual-and-team engineering product**, not an author's general judgment or an upstream project's intrinsic quality. In particular, Alton's operating vision has a narrower implementation remit than the full build specifications. Its lower product-contract score does not diminish the strategic contributions adopted here.

| ID | Distinct plan | Historical self-score | Common-rubric score |
|---|---|---|---|
| A | Verified Changes | 90.58 | 90.34 |
| B | One Lead, Many Bounded Workers | 91.62 | 90.28 |
| C | The Engineering Foreman | 91.47 | 91.80 |
| D | Verified Delivery | 91.98 | 91.30 |
| E | BulletFarm Core | 89.00 | 91.26 |
| F | Alton: Shared vision / Nightshift | Not stated | 81.84 |
| G | Final adjudication baseline 1.0 | 93.92 | 93.86 |
| H | Personal ownership / Shared evidence, 2.0 draft | 94.78 | 94.40 |
| I | Own the outcome, rebuild baseline 2.0 | 94.50 | 94.34 |
| J | Harness-informed baseline 2.1 | 94.50 | 94.48 |

**Leading cluster:** J (94.48), H (94.40) and I (94.34). Their sub-point spread is not a meaningful performance separation. J is the richest execution/learning reference; H provides clearer lifecycle decisions; I provides a more directly inspectable rebuild package. The recommendation is a consolidation of those strengths with G's authority foundation—not a claim that J is empirically better.

All scores are fresh judgments under the same weights. Earlier self-scores are provenance, not inputs. Historical tests reported inside a document were not treated as live implementation evidence. The current source tree, account entitlements, installed harnesses and owned forge were not audited.

## 2. The common 0–100 rubric

For every category, rate specification quality from 0 to 100; multiply by its weight and divide by 100. The twelve weights match the earlier common rubric, so scope is not silently changed to manufacture an improvement.

**Anchors:** below 50 lacks an essential mechanism; 50–69 describes a plausible happy path with major gaps; 70–84 is useful but has consequential unresolved design choices; 85–94 is strong, bounded and implementable; 95–100 is unusually precise, internally consistent and testable for that category. A score of 100 would still not turn a document into operational evidence.

A category considers the stated mechanism, its failure boundaries and the build/test path. Missing detail is not assumed implemented, but a deliberately disabled capability is not treated as an unsafe active capability. More pages, more services, more agents and more claims of novelty earn no points.

| Category | Weight | Detailed criteria |
|---|---|---|
| MVP simplicity | 14 | Deployment footprint; first useful slice; reuse and exclusions. |
| Autonomy | 10 | Fresh execution; typed waits and permissions; model-independent stopping. |
| Plans and contracts | 10 | Requirement coverage; coherent decomposition; bounded critique and uncertainty. |
| Verification | 12 | Gate integrity and adequacy; exact subjects; integrated and mission acceptance. |
| Recovery | 10 | Durable dispatch; fencing and handoff; ambiguous effects and restore. |
| Team ownership | 10 | Personal/domain control; reservations and handover; fairness and visibility. |
| Economics and learning | 8 | Complete cost and budgets; comparable outcomes; controlled improvement. |
| Provider realism | 6 | Supported protocols; pinned capability evidence; lifecycle/auth limitations. |
| Security | 8 | Physical isolation; current authority; protected evidence and data audiences. |
| Operator experience | 4 | One usable surface; smallest actionable decision; truthful status and reports. |
| Production and forge | 2 | Protected integration; immutable artifacts; explicit release/rollback limits. |
| Build precision | 6 | One definite contract; dependency-complete backlog; schemas, fixtures and rebuilding. |

### 2.1 Original implementation alternatives

| Category | Weight | A | B | C | D | E |
|---|---|---|---|---|---|---|
| MVP simplicity | 14 | 90 | 93 | 94 | 90 | 88 |
| Autonomy | 10 | 94 | 92 | 94 | 94 | 93 |
| Plans and contracts | 10 | 91 | 92 | 93 | 93 | 92 |
| Verification | 12 | 92 | 93 | 93 | 94 | 94 |
| Recovery | 10 | 92 | 87 | 91 | 91 | 93 |
| Team ownership | 10 | 89 | 88 | 90 | 91 | 94 |
| Economics and learning | 8 | 89 | 89 | 90 | 90 | 90 |
| Provider realism | 6 | 86 | 91 | 89 | 88 | 89 |
| Security | 8 | 90 | 90 | 92 | 93 | 94 |
| Operator experience | 4 | 92 | 93 | 94 | 93 | 92 |
| Production and forge | 2 | 89 | 89 | 90 | 92 | 90 |
| Build precision | 6 | 86 | 82 | 87 | 84 | 82 |
| Total | 100 | 90.34 | 90.28 | 91.80 | 91.30 | 91.26 |

### 2.2 Alton and the later complete plans

| Category | Weight | F | G | H | I | J |
|---|---|---|---|---|---|---|
| MVP simplicity | 14 | 91 | 94 | 95 | 95 | 93 |
| Autonomy | 10 | 76 | 95 | 95 | 95 | 96 |
| Plans and contracts | 10 | 84 | 94 | 95 | 94 | 94 |
| Verification | 12 | 87 | 95 | 96 | 96 | 96 |
| Recovery | 10 | 68 | 94 | 95 | 93 | 94 |
| Team ownership | 10 | 86 | 95 | 96 | 95 | 95 |
| Economics and learning | 8 | 91 | 92 | 94 | 94 | 95 |
| Provider realism | 6 | 80 | 91 | 91 | 91 | 94 |
| Security | 8 | 75 | 94 | 94 | 94 | 95 |
| Operator experience | 4 | 82 | 95 | 96 | 96 | 96 |
| Production and forge | 2 | 79 | 92 | 93 | 92 | 92 |
| Build precision | 6 | 72 | 92 | 88 | 94 | 92 |
| Total | 100 | 81.84 | 93.86 | 94.40 | 94.34 | 94.48 |

### 2.3 Failures that a weighted average cannot excuse

The affected autonomous capability is not releasable when an unauthorized actor can spend or access a domain, a writer can forge trusted acceptance, a stale selection can issue a new privileged request, untrusted source executes with publication credentials, an unverified integration subject merges, or an unresolved non-idempotent effect is blindly duplicated. Source text and a high design score cannot waive these boundaries.

Do not confuse a safe block with completed work. Report both: the boundary held, and the requested outcome remains incomplete. Likewise, do not confuse ending permission with proving physical termination, or acknowledging a command with confirming its remote effect.

## 3. Feedback on every substantive plan

### A. Verified Changes — 90.34/100

**Source position.** A proposes four Rust crates, immutable Git contracts, a hub ledger, rich Codex App Server integration and task/PR separation. Its verified-checkpoint dependency mode explicitly handles sequential microtasks within one PR. [A §§4–7, 10, 14]

**Retain.** Detailed ambiguous-effect recovery; durable artifact handling; externally rotated restore authority; the insight that a task is not necessarily a PR. These are substantive reliability mechanisms, not decoration.

**Improve.** The intra-PR checkpoint scheduler is more machinery than the first useful release needs. The example multiplies attempts and repairs without one total invocation definition. Minimum budget admission should precede the first paid adapter path, rather than being split between early reservation language and a later budget ticket. The current-generation rule needs an explicit handoff so delivery does not depend on a permanently live author.

**Decision.** Keep its recovery contract; default to one coherent code task with internal checklist steps per PR. Do not adopt the extra checkpoint dependency mode for the MVP. Replace nested retry allowances with one explicitly authorized cumulative limit.

**Decisive test.** Finish a writer, expire its execution lease, then complete legitimate verification/publication under a current task grant; reject publication after that grant expires. A's existing recovery detail is valuable, but this phase transition must be explicit.

### B. One Lead, Many Bounded Workers — 90.28/100

**Source position.** B favors headless subprocesses, fake dependencies, environment preflight and two initially independent proposals followed by synthesis for nontrivial planning. It explicitly includes mission-level acceptance and evidence-producing investigations. [B §§3, 7–9, 11, 18]

**Retain.** Fake-first delivery; baseline reproducers; the shortest provider seam; separation of useful negative research from code shipping; total-cost comparisons against a strong baseline.

**Improve.** Independent double proposals should be selective, not compulsory overhead for every ordinary feature. Its combined Claude/Grok ticket hides two separate certification efforts. The final pilot depends on a production-handoff package even though team value can be proven without production authority. Scope reservations and post-author delivery deserve the same precision as execution leases.

**Decision.** Adopt B's practical inner-loop discipline. Use Fast/Standard/Deep planning, split adapter tickets, and make production a separate extension. The uploaded 28-item companion belongs to C; B's inline appendix remains B's actual build sequence.

**Decisive test.** A fake-backed deterministic task and a real supplied task both complete before the council or release integration exists. A missing tool blocks before avoidable model spend.

### C. The Engineering Foreman — 91.80/100

**Source position.** C selects one Rust package, one small workbench, a supplied-contract inner loop before polished planning, and a truthful PR-ready boundary when safe native integration is unavailable. [C §§4, 18, 20–21, 25–26]

**Retain.** This is the best of the five original product shapes: small controller, on-demand Foreman, narrow task contracts, clear draft-only planning, ordinary modules and no homebuilt merge fallback.

**Improve.** Explicitly retain a change reservation after the worker finishes. Add assembled-mission acceptance rather than leaving it implicit in component checks. Its companion places the live adapter before its budget package and the CLI/full command surface later than the first CLI demonstration; define fake-only scaffolding or move those prerequisites. Several broad work packages are not yet executable lighter-agent contracts.

**Decision.** Preserve the small product shape and phased introduction. Repair lifecycle and prerequisite gaps rather than add a universal workflow engine. Do not treat the small category differences versus D/E as decisive evidence.

**Decisive test.** A pending PR blocks conflicting new work after its author has exited, while unrelated tasks continue. The operator can explain that wait without a model or provider terminal.

### D. Verified Delivery — 91.30/100

**Source position.** D emphasizes a small ready frontier, delayed-heartbeat handling, output backpressure, explicit unknown effects, skipped/flaky check handling and existing human review. It specifies several optional rich and simple provider transports. [D §§4, 6, 8–11, 15, 24]

**Retain.** Its operational details prevent real failure modes: late renewals must not extend stale authority, missing checks are not passes, and a positive model review cannot substitute for required human approval.

**Improve.** Freeze one transport per admitted provider. Use stable public PR identity with immutable candidate history instead of leaving branch-per-attempt behavior to create PR churn. BF-012 promises a goal-to-work-orders demonstration before BF-014 planning; label the first gate supplied-contract based. BF-013 certifies Claude planning/review, not cross-provider coding takeover. Add the latter before claiming the requested team capability.

**Decision.** Adopt its ready-frontier and observability discipline. Consolidate provider and delivery choices. Keep monetary cost, human minutes and delay separate unless an explicit conversion model is selected.

**Decisive test.** Suspend a runner and deliver a late renewal; it cannot revive expired authority. Then prove actual implementation transfer between two model-provider families, not merely two text reviewers.

### E. BulletFarm Core — 91.26/100

**Source position.** E chooses PostgreSQL and makes two-user coordination central. It explicitly retains pending-change reservations through review/integration and forbids candidate-controlled Git configuration during privileged collection. [E §§2, 7.4, 9.1, 14]

**Retain.** Its pending-change ownership and safe import rules are among the strongest contributions in the original set. Enrollment, browser/session controls, true check producers and the distinction between current authority and a process's private files are concrete.

**Improve.** A team hub alone does not require PostgreSQL. Retain a working PostgreSQL deployment, but a clean single-writer API hub can use local SQLite. Clarify the first useful solo demonstration versus the two-person release requirement: the opening and later sequence use different emphases. Fourteen broad packages need tighter acceptance bindings before economical-agent dispatch. Minimum budget implementation must precede paid work.

**Decision.** Keep its ownership and import boundaries; do not inherit an additional database service or oversized first demonstration merely to signal enterprise readiness.

**Decisive test.** Import a candidate containing hostile Git configuration without executing it in the publisher, then show reservation ownership survives a crashed author and is released only after a reconciled disposition.

### F. Alton's Shared vision / Nightshift — 81.84/100

**Source position.** Alton's goal is unattended shipping plus controlled harness learning. People own domains; eventual factories dispatch independently by partition. Warm shifts execute batches, per-task commits preserve attribution, and block candidates integrate on a reporting cadence. File existence represents task state. [F §§1–12]

**Retain.** The strongest strategic contribution is durable assets versus disposable machinery. Gate adequacy comes first. Human takeover, respecify/decompose/do-it, named escalation owners, shared gate versions, fair-start experiments and an explicit architecture exit test should all survive. [F §§2, 10–13]

**Improve.** Unknown blockers must block rather than only log. Text changes cannot mint unlimited retries. A block-length lease must not release an unresolved change merely because the process disappeared. Separate advisory stall detection from mandatory hard-limit enforcement. A whole-PR revert can delete its own regression test. Its Phase 1 excludes comparison while the MVP checklist includes two harnesses and sampling. Federated global capacity and cross-domain changes need explicit authority mechanics; a site field is insufficient.

**Decision.** Adopt the operating philosophy, not all of its topology. The 81.84 score is for this requested complete product/build contract: it loses points for unspecified conversational/UI, detailed recovery and authority mechanisms, and conflicting MVP sequencing—not for putting people first. It does not imply Alton's existing code lacks controls; that code was not examined.

**Decisive test.** A failed task can be respecified or taken over in one action, retains prior cost and known work, and cannot bypass the same gate. Shipping proceeds while a separately funded challenger remains unable to publish.

### G. Final adjudication baseline 1.0 — 93.86/100

**Source position.** G consolidates the five original proposals into one Rust/SQLite controller and introduces a precise separation of execution lease, change reservation and candidate selection, with current task authority for post-author delivery. [G §§7–19]

**Retain.** This is the strongest authority and recovery foundation: permanent generations, exact evidence, externally mediated effects, no false exactly-once guarantee and explicit backup-restoration fencing. Its unified build graph resolves many earlier mismatches.

**Improve.** It does not yet operationalize Alton's gate-adequacy-first phase, human takeover/drain workflows or controlled shipping-versus-shadow learning as fully as later plans. Physical resource occupancy must be tracked separately from relinquished writing authority. Specify stages for checks that cannot run until a draft branch exists. Planning itself needs bounded jobs before there is a task graph.

**Decision.** Keep its safety properties, not every earlier phase label. Add the human, gate and job refinements from H/I, and the profile/learning controls from J.

**Decisive test.** Current grant revocation, selected-candidate change and an in-flight lost PR reply are raced deliberately. A historical authorized attempt cannot issue a new effect, and an already dispatched action is reconciled rather than falsely recalled.

### H. Personal ownership / Shared evidence, distinct 2.0 draft — 94.40/100

**Source position.** H presents five user-facing record concepts, one common run protocol for paid work, owner/domain consent, human takeover and return, stage-specific gates, truthful physical-capacity accounting and a cold-start build checkpoint. Its self-score is 94.78. [H §§3–16, 18–20]

**Retain.** H has the best explicit closure of several subtle boundaries: ending a writer does not free an unconfirmed process slot; draft publication can obtain CI-only evidence without pretending the PR is ready; inference, verification and human contribution do not share identical permissions; Git contract hashing is over exact bytes.

**Improve.** H's text names a contract directory, SQL, backlog, examples and checker, but only its full specification and START_HERE are available for this distinct draft in the mounted conversation/package inventory. Its build-precision score therefore does not credit uninspected sidecars. This is an artifact-availability limitation, not a claim those files never existed elsewhere. It also predates J's inspected harness/profile and learning-promotion detail.

**Decision.** Use H's lifecycle and stage definitions in the new normative core. Supply one actual matching schema/backlog package rather than combining H's prose with I's differently named schemas silently.

**Decisive test.** A fresh agent follows only the delivered version-3 kit. Every cited file exists; job purpose, profile, grant and gate stage agree across prose, examples, schemas and storage.

### I. Own the outcome, rebuild baseline 2.0 — 94.34/100

**Source position.** I keeps personal owner lanes under one allocator; distinguishes task/lease/reservation/attention windows; adds gate adequacy, human workflows, shadow evaluation and portable exports. Its kit supplies five schemas, examples, SQL, a backlog and an offline checker. [I §§1–22]

**Retain.** It turns the synthesis into inspectable build artifacts and explicitly handles planning before a task graph through mission-level model jobs. Human privacy, scoped domain ownership, cumulative retries, separate shipping/learning budgets and window-independent PR delivery are strong product decisions.

**Improve.** Its separate model-job and task-run shapes plus later profile bindings can be simplified in a clean build. The seal transition releases an execution slot without H's explicit physical-stop condition. Stage-specific draft publication versus CI-ready completion should be stated in the main transition table, not inferred from CI-only language. Add a documented migration instead of mixing this kit's wire shapes with H's prose.

**Decision.** Reuse its tangible rebuild discipline, examples and test obligations. Consolidate only the duplicated bookkeeping, not the distinct permissions of writers, planners and verifiers.

**Decisive test.** A planning job can be budgeted without any implementation task; a verifier can finish after an author exits; an unconfirmed author still consumes physical exposure; and a restored export cannot resurrect live grants.

### J. Harness-informed baseline 2.1 — 94.48/100

**Source position.** J preserves I's numbered body, adds a research decision layer and H1–H7 integration contracts, and uses an ExecutionBinding sidecar to avoid changing the five old schemas. It distinguishes Prime executor reuse, OpenJarvis component reuse and QM scope/authority patterns. [J R1–R7, H1–H7]

**Retain.** The model is no longer the sole experimental unit: harness, model, context, skills, memory, engine and environment are pinned together. Native refinement cannot alter shared authority or acceptance. Prime session-veto/child/cancellation cases and QM's human-only authority distinction are concrete, relevant additions.

**Improve.** The growing research/body/addendum/sidecar precedence surface now costs implementer attention. A fresh build should use one purpose-tagged job contract with its profile identity directly bound, while preserving a documented import path for old envelopes. Reuse scores compare different named adoption routes; they are not an intrinsic harness leaderboard. The new default must include H's stage and physical-capacity refinements, which are not in I's copied core.

**Decision.** J is the best current execution/learning reference, not a complete replacement for H's lifecycle clarity. Keep Prime as a certification candidate, not a mandatory dependency before a conforming incumbent can ship. Keep OpenJarvis optional and QM a pattern/client, not three stacked controllers.

**Decisive test.** Profile replacement changes only future eligible jobs; a model cannot approve its own grant/profile promotion; a native daemon cannot outlive the bounded job unnoticed; and shadow output cannot become the shipping selection.

## 4. Briefs, backlogs and packaging: score the right object

The alignment brief [K] and harness decision brief [L] are not full implementation alternatives. Their five equally weighted review-brief criteria are source fidelity, decision clarity, distinction between agreement and recommendation, coverage at the stated level, and navigation/usability. K scores **93.0/100** (95/95/96/87/92); L scores **92.6/100** (94/94/95/87/93). K clearly separates central arbitration from eventual federation; L clearly separates executor reuse from platform adoption. Neither should be dispatched as a standalone engineering contract; both defer mechanisms to a longer specification.

The available backlogs use a separate equally weighted rubric: structure, dependency integrity, first-use prerequisite clarity, acceptance specificity and source traceability. These scores are artifact scores, not architecture scores.

| Artifact | Five category scores | Score | Specific feedback |
|---|---|---|---|
| A: 24 tasks / 12 groups | 95 / 94 / 78 / 84 / 89 | 88.0 | Rich fields and checkpoint modes; minimum budget and retry semantics need tightening. |
| C: 28 packages | 58 / 93 / 62 / 80 / 100 | 78.6 | Perfect ID/title pairing to C; prose-heavy and not fully bound executable tasks. |
| G: 24 core + 6 extensions | 94 / 96 / 87 / 88 / 87 | 90.4 | Coherent gates and check/fault references; actual owner, checks and repository bindings remain unresolved. |
| I: 22 core + 6 extensions | 88 / 94 / 88 / 79 / 87 | 87.2 | Useful concrete kit; broad single acceptance sentences leave work for task refinement. |
| J: I plus seven adoption amendments | 90 / 94 / 86 / 85 / 88 | 88.6 | Good preservation; sidecar adoption and legacy core need one definitive build graph. |
| H: referenced machine backlog | Not available for inspection | Not scored | Do not substitute I’s backlog solely because both documents say version 2.0. |

Static graph checks can detect missing IDs and cycles. They cannot prove that a first paid executor has budget controls merely because every dependency ID exists. The new backlog therefore ties each first exposed capability to the packages that implement its authority, isolation and evidence requirements. Larger infrastructure packages are build work packages; refine them against the actual repository before sending them to lighter models.

The current C companion is paired by exact ID/title matches to C, not by a filename guess. Generated duplicates and PDF renderings are representations, not extra design votes. Historical validation counts remain historical; this package reports its own executed checks separately.

## 5. Synthesis decisions: what survives and what does not

The strongest final plan is not “everything in J plus everything in H.” That would retain contradictory wire types and duplicated layers. It is one small target with a traceable decision for each conflict.

| Choice | Selected default | Reason and provenance |
|---|---|---|
| Ownership | People/domain lanes; one shared allocator | Keep F’s accountable owners and H’s consent; retain G’s shared authority. |
| Storage | SQLite on the clean-install hub; keep conforming existing PG | A–D/G–J simplicity; E’s existing PG can remain, but never two backends by default. |
| Contracts | Exact-byte immutable Git JSON; explicit runtime receipts | H clarity; reject F’s absent-file-as-done rule. |
| Job bookkeeping | One purpose-tagged machine-job envelope | H common run bookkeeping, I’s pre-plan jobs, J’s profile identity; distinct privileges remain. |
| Physical capacity | Release only on confirmed stop | H refinement; ending writer authority alone is insufficient. |
| Change ownership | Retain reservation through pending delivery | E/G foundation, preserved in later plans. |
| Checks | Prepublication, candidate, integration and release stages | H; no circular CI-before-PR dependency or falsely green draft. |
| Cadence | Continuous eligible PRs; windows govern attention/budgets | Retain F reports without forcing block-sized code. |
| Planning | Fast / Standard / Deep fixed flows | C/G–J; independent alternatives only when justified. |
| Reuse | Conforming incumbent; Prime RPC candidate; OJ optional subset; QM patterns | J research retained; no wholesale platform fork or forced harness rewrite. |
| Learning | Evidence → versioned proposal → held-out test → approved canary | F experimental separation plus J scoped promotion. |
| Build packaging | One version-3 contract, backlog and compatibility note | Preserve sources; remove competing active addenda. |

### 5.1 New closure requirements from this review

**N1 — Current prerequisite validity.** A historical merged receipt is necessary for a code dependency but may not remain sufficient after a known revert, API-breaking replacement or withdrawn evidence. Recheck exact current inputs and relevant protected preconditions; a merge ancestor alone does not prove the behavior still exists.

**N2 — Budget to reach the next honest boundary.** Before launching a writer, reserve the bounded mandatory verification/review allowance needed to reach the declared review-ready boundary, not only the next cheap model invocation. This is a configured conservative completion reserve, not a prediction that all future repair will fit.

**N3 — Every machine job has its own recovery identity.** Planners, reviewers and verifier processes require durable start/result, deadlines and stale-result handling too. Reuse job supervision; never borrow a live writer lease to authenticate a later verifier.

**N4 — Human authority cannot leak through the Foreman.** Record the initiating human and the currently authenticated execution actor separately. A model acting for an administrator is still a model-scoped capability, not an interactive administrator session.

**N5 — One rebuild contract.** All new envelopes are version 3. Legacy imports are explicit, paused and non-authorizing. The compiler should not need to understand a prose addendum to discover that an old example means something different.

These are reviewer refinements. H already supplies the stage/capacity distinctions; J already supplies profile and human-only capability principles. They are credited rather than represented as wholly new inventions.

# Part II — The final product and engineering contract

## 6. Product contract and binding non-goals

An owner submits an outcome through `bf run` or the same web command path. The Foreman proposes a bounded plan, a second eligible model challenges important assumptions, and the hub admits an immutable task frontier. Workers execute with fresh task contexts. Independent checks produce exact-subject evidence. A protected publisher creates reviewable changes and observes real integration. The owner can inspect, steer, pause, respecify or take over without opening a provider terminal for routine supported work.

The system is useful with one person and one agent. Team mode adds shared arbitration and visibility, not a different product. The Foreman is replaceable reasoning; current state, authority, budgets and completion remain deterministic even during a provider outage.

**Primary measures:** delivered parent requirements at their declared boundary; their later survival or rework; total attributable money; active human handling; and elapsed time. Keep these quantities separate. Agent count, output tokens, commits and PR counts are diagnostic only. A useful negative investigation is a different outcome from shipped code.

**Binding exclusions for the initial build:** no new Git object store, workflow DSL, general chat bus, vector database requirement, active-active hub, peer-to-peer queue, required Kubernetes/Redis/Kafka, autonomous policy self-modification, new deployment platform, compulsory warm shift, mandatory one-PR-per-block, unobserved recursive workers, full company-assistant suite or universal terminal scraper. Adding a feature requires a named failing acceptance case, an observed user need, or an explicitly funded experiment.

### 6.1 Delivered capabilities, not vague autonomy levels

`assist_only` allows bounded proposals under human supervision with visible missing acceptance. `verified_pr` allows independently checked review preparation while preserving human merge requirements. `policy_merge` additionally requires the exact forge integration, issuer and approval tests. `production_observed` requires the later deployment/observation adapter. Capabilities are admitted per repository/task class and profile, not universally across a whole company.

The first code mission defaults visibly to `pr_ready`. A requested `merged` or `production_observed` goal cannot silently downgrade. A dependency requiring merged code still waits for integration even when its predecessor's own mission requested only a PR. That wait is a genuine delivery dependency, not a native-agent pause to automate away.

### 6.2 Solo, team and external work

`bf serve --local` runs the same authority for private standalone repositories. A team-managed repository has exactly one configured managed authority. Personal runners connect outbound to it; an offline laptop does not create new team leases. Private experiments outside managed work are allowed, but their branch is observed/adopted before BulletFarm assumes responsibility for delivery.

Owner lanes contain policy, priority, eligible-profile preferences, budget and notification settings. They are not separate schedulers. The shared maintenance lane handles only explicitly delegated cross-domain or unowned work. Missing ownership goes to a named intake owner. CODEOWNERS is an input to ownership mapping, not automatically a spending or scheduling grant.

A multi-repository mission has ordered per-repository changes and an assembled compatibility manifest. It does not pretend one Git PR or one transaction atomically updates several repositories or services.

## 7. One small deployment

```text
Custom bf CLI / Vite + React workbench / later thin channels
                         |
                 Authenticated command API
                         |
                  One Rust hub process
        contracts | authority | scheduler | reconciliation
                         |
           One SQLite database + immutable artifacts
                         |
               Outbound-connected Linux runners
                         |
          Isolated model jobs / independent check jobs
                         |
                Exact candidates + evidence
                         |
              Credential-separated publisher
                         |
                Existing protected forge / CI
                         |
                Existing release system, later
```

Use one Cargo package with a library and `bf` binary. Roles are `serve`, `runner` and client commands. Serve compiled static web assets from the hub. The first certified worker platform is Linux; desktop clients can control remote Linux runners. Do not build identical macOS/Windows sandbox implementations for first value.

Suggested modules are `domain`, `store`, `auth`, `control`, `runner`, `adapters`, `evidence`, `forge`, `api` and `cli`. Planning, routing, reporting and learning proposals are submodules, not services. Use established Rust HTTP/async/serialization/CLI libraries and pin actual compatible dependencies during implementation; no invented current lockfile is provided.

The clean-install database is local-disk SQLite with one bounded DB command worker. Use foreign keys, short transactions, a busy timeout and durable acknowledged mutations. WAL's documented single-writer and same-host constraints require API access by remote runners, never direct network-filesystem access to the DB. Verify the linked runtime contains the documented WAL-reset fix before enabling WAL. [W3]

Preserve a working conforming PostgreSQL backend if replacing it would cost more than keeping it. Freeze that deployment decision once. The delivered structural reference is SQLite; it does not require a tested private implementation to migrate.

No SQL write transaction spans a Git subprocess, model request, network API call or compilation. Batch bounded progress updates; do not commit one transaction per streamed token. An OS instance lock prevents two hub processes on the same store. No high-availability or automatic failover is promised.

### 7.1 Minimum process separation

The hub never executes candidate code, builds, tests, repository hooks or unrestricted planning shells. Tool-using planners and reviewers are runner jobs too. Candidate execution has no hub, forge, trusted-check, signing or deployment credentials. A trusted reporter authenticates check results outside the candidate process. The publisher operates on validated objects without executing source.

Separate directories or a private clone are not an OS sandbox. Git documents administrative state shared by linked worktrees. [W6] Use a tested existing disposable Linux sandbox/VM boundary with a private writable home and Git state, restricted mounts/egress, process and resource limits, no host SSH agent, host container socket or adjacent owner workspace. A compromised host remains outside a same-host process-isolation guarantee.

### 7.2 Make lean and fast measurable

Use a declared benchmark host/build/storage profile with 10,000 historical tasks, four simulated runners, and an explicit mixed load such as 20 status reads plus 10 short control mutations per second. Proposed starting targets: status/why p95 under 100 ms, durable small command p95 under 250 ms, idle hub RSS under 150 MiB, and zero model calls for status. These are engineering targets, not observed results or universal service guarantees.

Report remote network, model startup, clone/snapshot preparation, inference, tool execution, verification and integration separately. Bound every queue. Protect heartbeat, cancellation and result commits from a slow UI or huge log stream. Optimize unnecessary model calls and repeated setup before porting a mature harness to Rust for a latency it does not dominate.

## 8. Contracts, identities and one owner for each fact

The visible nouns are **Mission, Task, Attempt, Candidate, Evidence, Decision and Owner**. Internally, a purpose-tagged **Job** supervises every machine execution; an implementation Attempt is the shipping-write specialization of a Job. This retains earlier terminology while avoiding duplicated planner, writer and verifier lifecycle code. Human contributions are explicit records, not fake processes with heartbeats.

| Fact | Authoritative source | Never infer it from |
|---|---|---|
| Agreed task/plan/decision bytes | Confirmed immutable Git object + exact-byte digest | Latest editable Markdown or a model summary |
| Current grant, selection, lease, budget, reservation | Hub state transaction | A pinned historical permission snapshot |
| Exact proposed code | Immutable commit/tree and validated provenance | A branch name alone |
| Required check outcome | Authenticated producer and exact check input manifest | Writer text or a same-named untrusted check |
| Actual PR/ref/merge | Canonical forge observation/readback | An accepted request or missing reply |
| Deployment and health | Existing release/monitoring receipts | Merge success or a job exit code |

### 8.1 Identity and numeric rules

Opaque globally unique IDs are identifiers, never authorization. Version-3 JSON uses lowercase snake_case enums. All JSON integers are within JavaScript's safe integer range; monetary micro-units are nonnegative checked integers with explicit currency. Rust uses checked arithmetic and rejects overflow. Decimal model settings use strings before adapter-native conversion. No float budget arithmetic.

Contract and profile digests are SHA-256 of **exact UTF-8 bytes**; reject duplicate JSON keys and invalid encoding before interpretation. Formatting changes alter the digest. The digest and enclosing Git commit are stored outside those bytes, avoiding self-reference. This deliberately selects H's simpler byte identity over I's semantic JSON canonicalization. Importers retain old digest algorithm labels; they must not relabel an old digest as the new algorithm.

Timestamp observations use UTC ISO-8601. Runner elapsed deadlines use a monotonic clock with conservative renewal timing. Hub-assigned committed event sequence is authoritative ordering; provider timestamps and local sequences are observations/deduplication aids.

### 8.2 Mission contract

A Mission identifies owner, allowed repositories/domains, outcome/non-goals, stable parent requirement IDs, declared delivery goal, planning mode, grant reference, budget accounts, assembled acceptance obligations and notification/window policy. Planning begins under the authorized mission before an implementation task exists. Unfunded planning is not an invisible Foreman exception to accounting.

### 8.3 Task contract

A Task identifies logical ID/revision/lineage/mission, one source repository, owner/domain, kind/class, objective/non-goals, parent requirement IDs, typed prerequisite revisions, write paths/interface keys, behavioral acceptance with trusted check references, gate/risk/routing references, granted allowance, stop conditions and context references. A code task's delivery goal is `pr_ready` or `merged`; deployment is the mission/release goal. Investigations end at `evidence_accepted`.

A proposed check ID can mean a trusted recipe that will test newly implemented behavior; it need not pretend a test already exists. Before live execution the recipe must actually resolve to protected runnable checks or a designated manual acceptance obligation. Schema-valid is not ready-to-run.

### 8.4 Scope and source inputs

A file path is exact; a directory ends in `/`. No implicit glob semantics. Reject absolute paths, `.`/`..`, backslashes, NUL/newline and ambiguous invalid paths. Compare path segments, not raw prefix strings. Renames require both old and new path permission. Symlinks, gitlinks/submodules, mode changes, generated code and policy files receive explicit rules.

Read paths aid retrieval but do not lock the whole repository. Write reservations coordinate final delivered changes; the sandbox confines transient operations. Always check the complete actual diff. An apparently low-risk task that touches authentication, CI, migrations or release infrastructure acquires the stricter required checks and decisions from the actual surface.

### 8.5 Execution profile

A profile is immutable configuration: harness/adapter/version, one transport, model/provider/settings, context strategy, prompts, skills, initial memory, engine endpoint reference, environment/hardware class and native-delegation ceilings. A job directly references its digest. There is no new v3 ExecutionBinding sidecar.

A profile is **not a grant**. Admission intersects current owner/data policy, capabilities, budget and sandbox certification. Endpoint and credential references resolve from protected settings; secrets never appear in the profile. Runtime observations must match the resolved admitted configuration. Human contributions do not require a fictitious model profile.

### 8.6 Candidate, evidence and receipt

A Candidate binds exact commit/tree/base, task contract, producing job or human contribution, actual changed paths and immutable artifact manifest. A versioned current Selection determines which candidate is eligible for new verification/delivery. It does not imply that any check passed.

Evidence binds the exact subject, check stage/recipe/fixture/environment, expected producer, suites discovered/executed/skipped, first outcome and artifacts. A CompletionReceipt records the declared boundary actually reached, supporting evidence and external observations, cost confidence and unresolved limitations. Subsequent regressions append new observations; they do not rewrite history to look successful.

## 9. Git intent and activation

Use one protected controller-written branch per repository, `refs/heads/bf/control`, containing mission files, immutable numbered plan/task revisions and adopted decisions. Ordinary code remains on ordinary PR branches. Metadata updates are forward-only and expected-old-ref checked; they do not trigger application deployment.

Git supports expected-old-value ref operations. [W4] This corrects the broad claim in Alton's text; it does not supply a transactional cross-machine budget/lease/effect system. BulletFarm uses Git for durable agreements and SQL for live promises.

Activation is a recoverable protocol: validate the full proposal; persist publication intent; publish/confirm exact Git objects; atomically import the complete validated graph and active revision; then admit jobs. A crash between Git and SQL leaves inert recoverable content, never half a runnable graph. Repeating the same publication/activation adopts the same identity.

Substantive revisions produce an explicit preserve/cancel/supersede mapping. Active work stays on its old contract only while still permitted; it never silently inherits changed scope or tests. A current revocation takes precedence over historical authorization. Immutable documents do not include their own enclosing commit hash.

Import existing `todo/*.md` preserving IDs, owners, domains, class, blockers, gate hints and source provenance. Unknown blockers are resolved against retained records or explicit external evidence; otherwise they block. Cancelled, removed or superseded tasks are not completed prerequisites. File deletion never sets `merged`. `AGENT_CHAT.md` may be a generated digest, not a coordination authority.

### 9.1 Current dependency rule

A code prerequisite requires its specified integrated receipt plus validity at the selected current source base. The hub checks current known invalidations, relevant interface/version preconditions and required sanity checks. A revert can preserve Git ancestry while removing behavior; ancestry alone is insufficient. Unknown required applicability blocks or creates a replan decision.

Do not claim a universal semantic-revert detector. Owner/forge observations and explicit preconditions catch defined invalidations; current integrated testing remains necessary. A downstream worker gets exact component revisions and assumptions. Significant relevant drift must be reviewed, not silently hidden behind `latest`.

### 9.2 Coherent granularity

Default one independently verifiable code task and its tests to one PR. Internal checklist steps may use sequential fresh contexts within the task's allowance but are not an intra-PR distributed task graph. Code dependencies wait for actual merge. No stacked-PR engine in the MVP.

Split unrelated behavior around stable interfaces; keep coupled changes together when splitting creates failing intermediates or repeated context costs. Hard uncertainty starts as a bounded investigation with hypothesis, baseline, frozen inputs, resource budget and stopping condition. A negative finding can satisfy the investigation. Preserve parent requirements so splitting never manufactures throughput.

## 10. Planning, fresh contexts and the Foreman

Fast mode uses one scoped proposal for a familiar narrow change. Standard uses A's proposal, B's critique against original requirements/source, and A's single revision. Deep uses two initially independent proposals, one synthesis, and at most one further targeted adjudication or an authorized discriminating experiment. Standard is three substantive calls, but its critic sees the draft; it is not blind independence.

Each material finding states affected requirement/source, severity, evidence, proposed correction and a distinguishing test. The adopted plan records addressed/rejected-with-reason/unresolved dispositions. Structural validation checks IDs, cycles, scope, authorization and acceptance mappings; it does not prove semantic completeness. Model agreement and scores cannot waive blocking obligations.

`bf plan` requests a draft only. `bf run` requests in-policy planning and implementation under a selected grant. Once a conforming exact plan is activated, runnable tasks start fresh execution automatically. Do not use native interactive plan mode to execute already approved work. No menu indexing, arrow-key automation or universal `yes`.

A new task, replacement attempt or provider handoff gets a new conversation and compact packet: objective, non-goals, exact source/dependencies, accepted decisions, behavioral criteria, trusted check descriptions, allowed scope, result schema, relevant failed attempt and remaining limits. Normal healthy tool loops within that attempt can retain context. Fresh does not mean amnesiac; it means carrying authoritative decisions rather than the whole debate.

The Foreman runs on user input, planning, meaningful unresolved uncertainty or requested narration. Deterministic reads answer status, readiness, grants, cost and process health. Typed questions, findings and decisions route only to relevant jobs/owners. No broadcast transcript or permanent token-consuming supervisor is needed.

### 10.1 Delegated requests versus human authority

The initiating human, authenticated actor, and delegated capability are distinct. A model launched for an administrator cannot call human-only grant changes, impersonation, required human approvals or shared policy promotion. These require an authenticated interactive decision channel and current action/subject authorization. The source's identity string is not a role.

Within standing authority, the controller may automatically select a plan, launch a job, answer an already resolved technical question or retry an eligible transient failure. A new product choice, spending extension, disclosure boundary or protected action is a named decision; unrelated work continues. It is not acceptable to route through another provider to bypass a denial.

### 10.2 Scoped memory

Reusable notes and skills carry owner, source domain/repository, audience, applicability, provenance and version. A summary derived from restricted content retains its restrictions. Recheck current rights both when building context and when delivering a response. Shared reports are assembled from records permitted for their intended audience; model discretion is not a disclosure control.

Begin with path/symbol/test search and a small revision-keyed repository map. Reuse safe immutable context, not stale authorization. Working scratch memory can evolve inside one admitted job. Cross-task promotion follows the controlled learning path in §22.

## 11. Gate adequacy and admission before scale

Run Alton's gate-first diagnostic using existing authorized CI/scripts before building a fleet. Select approximately ten meaningful historical behavioral fixes where available, record selection criteria, keep the accepted regression oracle, establish a passing good baseline, and reintroduce the relevant defect without deleting its detection test. Classify detected, missed, invalid injection, flaky and not-applicable cases explicitly.

Add a no-op candidate for a task requiring new behavior, a plausible constant result where relevant, missing test discovery, skipped mandatory cases and attempts to weaken acceptance. A whole-PR revert that removes the oracle is not a valid detection test. Ten cases diagnose coverage; they do not certify rare-failure reliability.

Gate integrity asks whether the author can replace or impersonate a check. Gate adequacy asks whether the check detects failures relevant to the intended class. Candidate verification asks whether this exact subject passed. All three remain separate.

Admission records specify task classes, risk ceilings, profile/gate revisions, manual review obligations, permitted delivery boundary, residual gaps and approving principal. A legacy repository may allow supervised proposals while autonomous merge remains disabled. Improve weak checks rather than broaden a green badge. Documentation or dependency changes are not universally harmless classes.

The first live task needs a meaningful independent acceptance path. Missing infrastructure/configuration is a typed onboarding block, not permission to fabricate a pass. First-use controls apply before their corresponding capability is enabled; a fake-only prototype can exist earlier without pretending it is a certified live worker.

# Part III — The execution and delivery contract

## 12. One machine-job protocol, with different permissions

Use one durable `Job` envelope for `plan`, `implement`, `investigate`, `review`, `verify`, and `shadow` purposes. A machine job is not necessarily a model call: a verifier may be an ordinary test process. Planning is mission-bound before any task graph exists. Implementation is bound to an activated task revision. Review and verification bind an exact candidate and selection version. Shadow work binds an evaluation cohort and cannot select a shipping candidate.

The job carries `schema_version`, ID, purpose, lane, executor kind, mission, optional task/revision, exact input digest, optional source base/candidate, model profile when applicable, current grant, budget hold, assigned runner, dispatch generation, authority epoch and deadline. A write-enabled implementation additionally carries the task's permanent writer generation. The two generations solve different problems: a task writer is superseded across repairs; an individual job delivery is superseded across runner assignments.

`lane` is `shipping`, `shadow`, or `wildcard`. These are policy tags over the same machinery, not separately deployed systems. Ordinary read-only planning/review is in the shipping workflow but does not acquire the implementation writer lease. The `verify` executor is a registered trusted check runner, not a model impersonating one. A human contribution records an exact submitted commit and its owner without inventing a provider session or requiring the person to send a heartbeat.

### 12.1 Small adapter contract

```text
probe(profile) -> CapabilityReceipt
start_fresh(job) -> RunHandle
read_events(handle) -> bounded normalized events
cancel(handle) -> requested | confirmed_stopped | unconfirmed
collect_result(handle) -> proposed_artifact | blocked | failed | interrupted
```

The adapter normalizes behavior; it does not schedule another owner's work, grant permissions, decide task acceptance, or publish to the forge. Optional input/permission callbacks are advertised explicitly. A profile needing an unavailable capability is ineligible. Never emulate missing authority semantics with terminal keystrokes.

Every normalized event names job ID, assignment generation and producer-local sequence. The hub derives the authenticated runner and global event sequence. Preserve bounded raw protocol diagnostics separately. Result frames are proposals until the server validates their role, subject and current authority. A process exit, `success: true`, or a final answer cannot set `merged`, `verified`, or `observed_healthy`.

### 12.1a Exact result payload

`schemas/job_result.schema.json` is the model/adapter result shape, registered as `result-v3`. It contains schema version, job ID, assignment generation, producer sequence, outcome, an optional proposed candidate (commit/tree/base and issued artifact ID), artifact references, optional typed blocker and a usage-artifact reference. Allowed outcomes are `candidate_proposed`, `artifact_proposed`, `blocked`, `failed`, and `interrupted`. A candidate outcome requires its candidate object; other outcomes do not carry one. A blocked outcome requires a code and explanation. No authoritative PASS, MERGED, actor role or grant can be supplied.

The hub also validates purpose: a planner returns a plan artifact, a reviewer returns a finding artifact, and an implementer may propose code. A verifier reports through `receipt-v3`, the Receipt schema, under its distinct enrolled role. Meter artifacts are authenticated observations whose basis is settled by the budget controller; a missing usage artifact is unknown, not zero. Transport success and schema validity do not grant the payload authority.

### 12.2 Three separate state dimensions

| Dimension | Values / interpretation |
|---|---|
| Job lifecycle | `prepared`, `starting`, `running`, `stopping`, `succeeded`, `failed`, `interrupted`, `unknown` |
| Logical writing authority | `none`, `active`, `sealed`, `revoked`; only one active shipping writer per task |
| Physical occupancy | `allocated`, `running`, `stopped`, `unconfirmed`; an unconfirmed process is not free capacity |

A job may have a sealed candidate while its process cleanup is still unconfirmed. Independent checking can continue on the immutable candidate, but its old runner slot is not counted as definitely available. This adopts H's explicit resource-accounting refinement. Author independence does not mean resource accounting can forget the author.

All machine jobs, including planners, reviewers and verifiers, have a durable start intent, exact runner incarnation, deadline, bounded output, terminal receipt and recovery path. Their late results may be retained as observations but cannot replace a newer job's authoritative result. The trusted verifier result requires the expected verifier identity and current job generation; it never borrows the author lease.

### 12.3 Task state and holds

```text
Code task:
proposed -> ready -> working -> checking -> review_ready
         -> integrating -> merged

Investigation:
proposed -> ready -> working -> checking -> evidence_accepted

Terminal alternatives: failed, cancelled, superseded
```

A task has an orthogonal `hold_reason` with owner and intended next action. `auth_required`, `dependency_invalid`, `budget_unavailable`, `review_required`, `resource_conflict`, `check_missing`, and `outcome_unknown` are different holds. Holding a task does not erase its candidate, PR or evidence. A failed attempt does not automatically fail the task; another attempt needs a current authorized retry decision.

Publication is a separate observed attribute: no PR, draft/open PR, closed, or merged. A draft PR may exist while the task is still `checking` because a CI-only check needs that remote object. `review_ready` means the declared automated preparation and required automated review are complete; existing human review can still be pending. `merge_eligible` is a derived predicate, never a model-authored status.

The mission's declared boundary remains explicit: `pr_ready`, `merged`, `production_observed`, or `evidence_accepted`. A goal cannot silently downgrade. A task can satisfy a PR-ready mission while its code dependency still waits for actual merge. Several completed tasks do not satisfy the mission until its required outcome checks do.


## 13. Identity, current grants and human-only decisions

A consequential request has three different identities: the authenticated actor, the initiating human, and the delegated capability. Record all three where applicable. A model acting on behalf of an administrator is still a model actor with a bounded capability; it is not an interactive administrator authorized to approve its own requests. This makes QM's deliberately human-only operations enforceable rather than merely absent from a tool menu. [W9]

Reuse organization identity in team mode. For a private pilot, an administrator can issue one-time enrollment material exchanged for a revocable, repository-scoped runner identity. Local bootstrap uses a private loopback/socket endpoint and a generated local secret, not an unauthenticated network listener. `bf init` creates configuration; it does not create remote rights, funding or provider consent.

Initial roles are administrator, engineer and observer, narrowed by repository membership and domain grants. The actor type is independently `human`, `agent`, `runner`, `verifier`, or `publisher`. All reads, SSE subscriptions, artifact transfers, commands and operation-status responses apply scope checks. Do not expose inaccessible object existence through differentiated error details or unfiltered global counts.

A grant identifies issuer, subject, permitted actions, repositories/domains, relevant data classification, expiry, revocation and allowance references. It is resolved from protected state, not trusted because a task or profile names it. Approval binds the exact action and subject, selection version, required policy, environment where relevant, actor and expiry. Changing the subject requires a new applicable approval.

### 13.1 Capability classes

| Class | Examples | Authorized path |
|---|---|---|
| Read and propose | Inspect permitted status; propose plan, finding or task amendment | Human or scoped agent capability |
| Execute within existing grant | Start an accepted task, ordinary in-sandbox tools, bounded repair | Deterministic controller under standing development authority |
| Change authority | Grant/role changes, spending extensions, required human approval, protected policy promotion | Designated authenticated human/administrator action |
| Report trusted facts | Verifier result, authenticated forge observation, publisher receipt | Exact enrolled producer and job/effect identity |

A normal authorized plan-to-code transition is automatic. This does not require pretending every request is harmless. Unknown tools, wider source scope, new data destinations and credential demands are denied or become a specific decision while unrelated tasks continue. Provider refusals and authentication barriers are not instructions to rotate identities or switch providers to evade restrictions.

Current revocations apply at admission and before new privileged dispatch. A pinned old policy is historical evidence, not immunity from present restrictions. A newly stricter check requirement must be reconciled before delivery. A newly broader policy does not silently widen a running task's scope.

Browser sessions need authenticated TLS, secure cookies and CSRF/origin protections. CLI and runner credentials stay outside source repositories and candidate sandboxes. Source text, issues, logs and imported Markdown are untrusted content, even when they contain JSON resembling an administrative command.


## 14. Claims, leases, scheduling and physical capacity

### 14.1 Atomic admission

The scheduler is deterministic Rust code. In one short transaction: verify the active task revision and expected version; required dependency conditions and known invalidations; current grants and risk; eligible profile/runner; path/interface reservations; owner/team WIP and physical capacity; implementation allowance; and all applicable budget holds. Acquire the complete incompatible reservation set or none. Increment the task writer generation, create the job, append the event and persisted launch intent, and commit before starting anything.

No transaction spans a model call, Git process, compilation or network request. An initial per-repository serialized admission check is sufficient for path/interface conflicts. It is a short database operation, not an eight-hour database lock.

Scopes are explicit repository-relative file names or directory prefixes. Match path components, not arbitrary string prefixes. Reject traversal, absolute paths, NULs and unsafe encoding. Renames require both source and destination permission. Gitlinks, symlinks, executable-mode changes, protected instructions, CI files and lockfiles get explicit risk/scope treatment. These claims coordinate delivered changes; the OS sandbox separately confines temporary execution.

### 14.2 Writer, reservation and candidate handoff

The execution lease identifies authority epoch, task/revision, job, assignment generation, writer generation, runner and deadline. A starting profile can use a ten-second heartbeat and sixty-second server lease with a conservative local stop margin. Those are tunable defaults, not a timing proof. Use monotonic elapsed time and acknowledged renewals; delayed replies cannot resurrect an expired generation. Test suspend/resume and clock changes.

At result collection, freeze writes or obtain an immutable consistent snapshot inside the untrusted execution boundary. Verify current job authority, full actual diff scope and artifact completeness. Transactionally record the exact candidate, advance its selection version, seal the author's writing authority, retain the task-owned change reservation and create an independent verification job intent.

A sealed job can replay the identical result idempotently but cannot submit changed code. A repair requires a new generation and new private workspace. Before admitting the repair, withdraw the old candidate's publication eligibility and reconcile any incompatible effect already in flight. Keep previous code and failures for diagnosis.

A reservation remains while the change is checking, under review or pending integration. Release or transfer only after actual merge, explicit authorized withdrawal/supersession, or reconciled abandonment. An old open PR does not vanish from conflict consideration merely because a timeout elapsed. Age creates an accountable decision; it is not permission to forget work.

### 14.3 Logical freedom is not physical freedom

After authority is sealed or revoked, request and observe cleanup of the entire registered process tree/container. Reclaim physical capacity only on a trusted stopped receipt or independently established destruction of the recorded sandbox. Unreachable occupancy remains reserved on that runner. Another runner may receive a successor if current authority, separate workspace, known cost exposure and configured team limits permit it; the dashboard still reports the unconfirmed process.

A stale process can remain physically alive after a partition. The enforceable product boundary is that it cannot submit a current candidate, access its successor's writable checkout, or publish privileged changes. Instantaneous global termination is not promised. Provider work already accepted remotely can continue billing after a local stop; this is retained exposure, not hidden free capacity.

### 14.4 Fair scheduling and completion priority

Persist a fair rotation cursor across eligible owners/domains. Within a lane, order by explicit priority, dependency-unblocking value, age and stable ID. Do not let an agent calculate that its work deserves everyone else's budget or that a reserved interface is probably free.

Initial tunable limits are four team writers, two per owner, two verifier slots on suitably sized infrastructure, one native integration path per repository and eight pending managed PRs per repository. Prepare roughly two useful waves of executable tasks. These are conservative starting values, not throughput claims.

Prefer finishing verification, review and integration when downstream queues fill. Protect interactive capacity and explicitly allocated owner budgets. Owner preference chooses among eligible execution profiles; idle capacity is not permission to use another person's credentials. Observe external PRs through authenticated webhooks and periodic readback. Unregistered unpushed edits remain outside coverage.


## 15. Stage-specific verification and protected delivery

This sequence adopts H's explicit stage distinction and preserves G/I/J's exact-subject evidence. It avoids the circular instruction “all remote checks must pass before creating the remote object that runs them.”

| Stage | Required state before advancing | What it does not claim |
|---|---|---|
| Prepublication | Current grant/selection; safe immutable import; actual scope; configured feasible local checks | Remote CI or merge completed |
| Draft publication | Persisted bounded branch/PR effect; exact remote head observed | Task is review-ready merely because a PR exists |
| Candidate / PR checks | Protected CI-only checks, complete required suites, independent risk-appropriate review | Human approvals or final integration satisfied |
| Review-ready | Declared automated preparation obligations complete; unresolved human obligations visible | Automatic merge authority |
| Integration | Native protected current combined subject, correct issuers and required reviews | Production deployment |
| Mission / release | Assembled outcome and requested artifact/environment observation | Universal correctness beyond those checks |

The gate manifest records stage, location (`local`, `ci`, or `both`), protected recipe digest, required suite identities, execution environment and expected producer. Reuse the same definitions where feasible, not a hand-maintained imitation of CI. CI-only obligations remain pending; do not put production credentials on a laptop to make local checks a literal superset.

### 15.1 Independent verification jobs

A verifier checks out the exact candidate in a clean constrained job. Its authoritative recipe and necessary acceptance fixtures are outside the writer's control. Candidate-authored tests are useful additional evidence, not the only oracle. Candidate code and tests execute without the trusted reporter's credential. This follows the same untrusted-execution boundary documented for privileged CI. [W11] The reporter authenticates job identity, exact subject, exit/results, suite discovery and artifact completeness before recording trusted evidence.

Evidence binds task/contract digest, candidate commit/tree, selection version, relevant base or integration subject, check/fixture/environment/toolchain identities, actual suites, result, producer and artifacts. For non-code investigations, bind the declared hypothesis and evidence artifact instead of inventing a Git candidate. A source signature authenticates a producer; it does not prove test adequacy.

Changed candidate content invalidates relevant candidate evidence. Changed gate, environment, selection or integration inputs require the applicable recheck. Preserve earlier evidence as historical, not currently applicable. Initially rerun required profiles rather than build a general proof-reuse system.

Missing test discovery, required skips, truncated results and wrong issuers cannot pass. Keep first results and all reruns. Flaky quarantine requires protected policy, an owner and replacement coverage; do not sample until green or let the author decide that its own failure is harmless.

### 15.2 Review and risk

A reviewer gets the original requirements, exact diff, relevant source and authenticated checks in a fresh bounded context. It does not primarily judge the author's success narrative. Findings identify affected clauses/locations and a concrete test or concern. A model review is an advisory artifact whose required workflow obligation must be satisfied; it is not a trusted test or a substitute for required human/Code Owner review.

Mechanical low-risk changes may omit an extra model review under protected policy. Behavioral changes require an independent review. Authentication, permissions, destructive data operations, public compatibility contracts and release infrastructure raise minimum risk from actual changed surfaces, regardless of a task's friendly label. Unanimous models cannot lower that floor.

### 15.3 Stable PRs and integration

Use one stable current PR per coherent task revision, with exact candidate history underneath. Repairs normally add forward commits from the observed remote head. A local rebase does not authorize force-pushing remote history. Unexpected human branch changes or a human-closed PR become explicit reconciliation decisions; do not overwrite or reopen them automatically.

Task-to-candidate-to-merge attribution is mandatory. One universal commit count or no-squash rule is not. Preserve original tested objects and record how the certified merge method maps to final content and lineage; the queue commit need not equal a final squash commit.

Use the existing protected native merge queue or an already implemented equivalent that passes the same behavioral checks. Required checks must test the current composition and intended issuer. GitHub's queue uses combined merge groups; required Actions checks need the `merge_group` event. [W5] A last-minute local rebase is helpful but does not close the race if main moves again. If the actual forge cannot enforce the required subject/protection, autonomous merge stays disabled and the product states PR-ready/manual integration.

Do not build a new refinery as the MVP fallback. Bisection is a later diagnostic: a red combination can arise from interacting changes rather than one independently bad branch. Any reduced candidate set must be reverified.

### 15.4 Dependency invalidation and assembled outcomes

An authentic old merge receipt remains historical after a known revert. It does not alone satisfy a present prerequisite. On known prerequisite withdrawal, rollback or breaking interface change, mark affected dependents held and re-evaluate their explicit preconditions on the current exact base. Preserve the old record; do not rewrite history.

Git ancestry alone cannot detect a behavioral revert: the original merge can remain an ancestor. Use known invalidation events and task-specific precondition checks, with actual combined verification as the final boundary. This is a bounded defense, not a claim to infer every semantic incompatibility.

A mission's final acceptance runs on the assembled result. Cross-repository missions identify each exact component and compatibility scenario; there is no claimed atomic multi-repository merge or deployment. A numerical research task needs its frozen data, tolerances, resource limits and independent comparison; it does not become correct because one example ran.


## 16. External effects, storage and recovery

The database and forge do not share a transaction. Each consequential remote write has a stable logical key, immutable request digest, exact subject, current authority reference, expected remote state and an effect record:

```text
pending -> dispatching -> confirmed
                     -> outcome_unknown -> confirmed | not_applied | blocked
```

The local transaction that revalidates present authority and marks `dispatching` is the dispatch-authorization point. Cancellation ordered before it prevents dispatch. Cancellation ordered after it treats the effect as potentially in flight, even when the network has not acknowledged anything. A best-effort abort is not proof the remote action did not happen.

Before privileged dispatch, require the current publisher/epoch, live task/domain grant, current revision and selection, applicable stage evidence, designated approvals, retained reservation, expected remote refs and no unresolved conflicting effect. Ended author leases are not required; expired current grants still block. Serialize effects affecting one logical PR/change.

### 16.1 Ambiguous responses

On a lost PR creation reply, read back the exact branch/head and stable task/revision marker before attempting another creation. Adopt an existing matching PR. A quick “not found” can remain inconclusive while a request is delayed or visibility lags. Retry only when target semantics or reconciliation establish safety; otherwise keep `outcome_unknown` and a named operator action.

The same rule applies to merging, check publication and release triggers. A local idempotency key does not magically make a remote non-idempotent API exactly-once. Already dispatched effects may complete after cancellation; record actual observations and perform only authorized follow-up actions.

### 16.2 All-job restart behavior

At normal hub restart, acquire the single-instance lock, inspect retained authority and unsettled effects, reconnect/inspect runner incarnations, fence expired work and reconcile before conflicting writes. Do not restart every task or every check blindly. A verifier crash gets a new check-job assignment; a late old result cannot supersede the new job's selected result.

The runner records exact sandbox/process identity before acknowledging execution. Duplicate start deliveries return the existing job. An ambiguous spawn interval yields inspection/quarantine, not an uninformed second accepted process. A reused PID or pathname is not job identity.

### 16.3 Disaster restoration

Restore paused. Quiesce the old publisher or revoke its remote credentials; rotate authority through a protected mechanism outside the restored database; invalidate old runner sessions; validate retained artifacts; reconcile all unsettled external effects and open PRs; then authorize new work. A new random epoch inside a rolled-back database alone cannot stop an old host that still possesses a usable forge token.

Manual restoration steps are acceptable for this single-hub release when the procedure is explicit and rehearsed. Do not claim active-active failover, zero downtime or recovery of changes never durably captured.

### 16.4 Artifacts and safe cleanup

Write bounded artifacts to issued temporary locations, validate size/hash/path, durably finalize, then commit complete metadata. Missing or truncated referenced artifacts are unavailable, not green evidence. Upload receipts derive the producer's authority; worker-provided URLs are not fetch instructions for the privileged hub.

Back up a consistent database snapshot plus the artifacts required by retained evidence. Retention/cleanup requires exact job/workspace identity, known disposition, policy and absence of active references. Age alone is not deletion permission. Preserve the failed candidate/checkpoint before destructive reset where capture was possible.

Logs and remote Markdown/HTML are escaped; terminal control sequences are sanitized. Bound requests, events, frames, artifacts and browser buffers. Optional progress deltas may be dropped with explicit counters; control/results must remain durable or fail explicitly. Slow UI clients cannot stall lease renewal, cancellation or evidence recording.


## 17. Human takeover, respecification and attention

Work belongs to people. Adopt Alton's takeover, escalation drain and ad-hoc entry modes, with H's explicit return-to-agents path. The person shares the acceptance obligations, not a requirement to imitate an unattended eight-hour process.

`bf take <task>` creates a durable operation under the person's current domain authority. Fence the old writer, withdraw pending publication eligibility, retain the change reservation, request a bounded checkpoint/stop and reconcile in-flight effects. Offer the latest known exact checkpoint or a clean base in a different human clone. Never transfer a potentially still-written directory.

An unreachable runner may have unsaved changes that were never captured. Show that limitation rather than claiming lossless recovery. A person can work provisionally in a separate clone while an old remote effect remains unknown, but no conflicting publication or unsafe reservation release may occur until that effect is reconciled.

`bf submit-human <task> --commit <oid>` adopts exact submitted content under current scope and runs the same independent checks. `bf return <task> --checkpoint <artifact>` records the human's checkpoint, decisions and unresolved failures and admits a new bounded agent job when authorized. Neither command requires access to the person's home directory or private chat history.

### 17.1 Escalation drain

Each escalation contains owner, age, requirement, exact failed criteria, attempted routes, cumulative costs/confidence, latest salvageable work and the smallest next decision. The owner may respecify, decompose, implement, reassign, grant a justified extension or withdraw. Group repeated infrastructure problems rather than demanding one response per affected worker.

A new contract revision may display a zero revision-local attempt count. It receives additional allowance only through an explicit authorized extension. Lifetime lineage, costs, failure evidence and permanent writer generations never reset. A whitespace edit, task split or model-written new ID cannot create free attempts. Children inherit parent requirement accounting and require explicit allocations.

Ad-hoc human work can be registered before entering managed delivery. Late registration supplies attribution, not retroactive permission. Urgent independent work can take its own protected PR rather than wait for a report window. Existing organizational emergency procedures remain explicit external governance, not an invented automation bypass.

### 17.2 Attention windows and reports

A window names an admission interval, spending allocation, capacity profile, report cutoff and optional bounded drain deadline. At its boundary, produce an honest snapshot. Do not discard a useful patch to make the report tidy or release a pending PR reservation because the clock changed.

Already admitted work continues only within its existing runtime, grant and drain allowance. Further calls require admission to applicable budget windows. Attribute costs to both consuming windows and original parent outcomes without charging twice. Advisory stall detection may suggest review; hard cancellation, lease loss and declared limits are enforced by deterministic components.

Reports work without a model: completed parent outcomes, review-ready and merged changes separately, missing checks, failures, pending reservations, decisions/owners, known/estimated/unknown spend, unconfirmed processes and exact next actions. Optional narration uses that frozen report and cannot invent counts. Respect quiet hours for informational notices; separately configured urgent incidents use the designated escalation path.

Human active-time records are voluntary/self-reported or explicitly approved measurements, labelled by source. Unknown effort is not zero. Do not infer employee quality from raw throughput or require keystroke surveillance. Team members see authorized work facts; raw traces may have narrower audiences.


## 18. Economics: reserve enough to finish checking

### 18.1 Measure outcomes, not token mix

Choose eligible profiles by data permission, current authority, needed tools, risk, context, owner preference, profile health and capability before price. Then choose an explicit task-class route: economical for calibrated bounded work; standard for moderate coupling; frontier for deep uncertainty or sensitive diagnosis. No model family is permanently writer or judge. No required 70–90% inexpensive-call quota overrides quality.

Report accepted surviving parent outcomes, total monetary/compute cost, active human handling and elapsed time separately. A blended internal business metric must disclose its conversion assumptions. Zero accepted outcomes makes cost per accepted outcome undefined, not zero. Token/PR counts cannot inflate delivered requirements.

### 18.2 Completion reserve — new explicit admission rule

A cheap implementation call is not the whole job of delivering a change. Before a write attempt, reserve its admitted execution exposure **and** the configured minimum mandatory verification/review envelope for the resulting candidate. Maintain this completion reserve at task/mission scope. The writer may not consume it as extra coding budget.

On author sealing, transfer or attach the held completion allocation to the required check/review jobs without creating another bill. On failed verification, retain actual costs and recompute remaining allowance before repair. A task that cannot fund its required completion path is held before further writing; an authorized owner can extend the allowance or reduce scope through a valid new contract.

This is an admission policy, not a promise every unknown defect can be solved with a fixed amount. Plans may be wrong, checks may fail repeatedly and provider accounting may be incomplete. Expose those limits. The rule prevents avoidable unchecked-output accumulation caused by reserving only the next model call.

### 18.3 Ledger and nested usage

Store integer monetary micro-units and explicit currency. Apply limits at organization/project, owner/domain, mission, original-work lineage, window and actual upstream resource pool. Multiple holds constrain the same spending; they are not multiple expenses. Settle each unique meter delta once. If a root reports inclusive child usage, do not add the child's usage again; record the reporting basis and use disjoint leaf totals or a single inclusive total.

Include planning, critiques, failed work, repairs, reviews, native children, provider tools and attributable compute. Report actual, estimated and unknown use separately. Missing telemetry retains conservative exposure; stopped processes are not evidence of zero cost. Actual overruns must be recorded in full even when they exceed a limit; freeze new admission where needed rather than rejecting the historical bill to keep a fake balanced budget.

Strict invoice caps require enforceable upstream/proxy request controls. A local timeout can stop new work without stopping already accepted remote billing. Profiles with less precise metering must show residual exposure and remain ineligible where stricter control is required.

### 18.4 Bounded retries

Default automatic allowance: three write-enabled invocations per original task, including initial implementation, repair and escalation. Internal ordinary tool loops are inside one invocation but remain under time/tool/spend limits. Asking for new code changes after a failed result consumes another invocation even within the same session. A fresh context does not reset the allowance.

Allow at most two transport retries where the previous outcome is known and the operation is retry-safe; account for native retries. Unknown external operations are not retry-safe by default. Repeated identical failure fingerprints prompt diagnosis/replanning instead of automatic repetition. A read-only format repair must not secretly become an uncounted code-writing repair.

Reserve interactive headroom separately from autonomous allocations. Different API keys can help identity and attribution but do not establish independent upstream quota pools. Unmanaged external sessions make capacity partially observed; display that limitation rather than claiming omniscient remaining quota.


# Part IV — Harnesses, people and improvement

## 19. Upstream resources: use the right layer

J's resource review is retained as a **selected-source review**, not a benchmark of the user's codebase. The pinned source snapshots remain useful for reproducibility. They are not necessarily the newest release or the binary installed on a runner. This review revisited selected protocol/security/inference interfaces and current official platform documentation; it did not install or benchmark the three upstream products.

| Resource | Source-supported contribution | Final use | Not adopted as default |
|---|---|---|---|
| Prime Agent | Programmatic persistent execution and structured RPC; detailed session and child lifecycle | First new executor to certify at the existing seam | Global task allocator, shared cross-owner daemon, native PASS as verification |
| OpenJarvis | Real Rust engine/core boundary alongside a broader application | Optional narrow inference-component/local-route spike | Entire training, desktop, tracing and scheduler stack |
| QM | Personal/shared scope and deliberate human-only authority decisions | Scope, audience and capability patterns; optional client when already operated | Broad company-assistant/keychain/browser platform as the engineering core |

The prior review prioritized **reuse routes**, not entire projects, at Prime 86.05, OpenJarvis 84.00 and QM patterns 83.35. Those historical route scores are not comparable to this document's ten full-plan scores. We retain the route decisions but do not claim a fresh universal project ranking from limited selected-file inspection. [J R1–R6]

### 19.1 One executor first, not one vendor forever

Preserve the first already conforming incumbent. Prime is a serious first certification candidate, including an existing Alton setup. The clean-room fallback is Codex noninteractive execution. A second genuinely different coding-provider family, typically Claude against that fallback, is required for the team gate. Two harness labels using the same underlying provider are not evidence of cross-provider coding takeover.

Codex's current documentation supports `codex exec --json` and schema-constrained final output. Start a new invocation for a new task, rather than resuming the planner. Configure the least permitted execution scope explicitly and isolate model credentials from repository-controlled processes. App Server can replace this transport when a required capability justifies it; do not support two transports merely to preserve every old option. [W1]

Claude's current headless documentation supports `-p` and structured output. Ordinary headless startup can load repository hooks/MCP configuration without an interactive trust dialog; bare mode changes discovery and authentication. Select and test one profile rather than paste a universal flag set onto all accounts. Native shutdown behavior is observed, then reinforced by the recorded sandbox boundary. [W2]

Grok Build is an additional certified machine interface, not Grok Bot and not an arbitrary similarly named community CLI. Its documented auto-approval mode does not itself bypass TUI plan review. Keep planning in BulletFarm and use one supported headless/ACP route when this adapter is enabled. [W7]

### 19.2 Prime RPC: exact certification obligations

Use the pinned RPC interface as the candidate adapter. Frame JSONL on LF bytes with bounded buffering and explicit request IDs. A prompt acknowledgment establishes admission, not completion. A `new_session` response can report outer success while its inner `cancelled` flag says no switch happened; verify actual new session identity and absence of unauthorized old context. [W8]

Place each managed root's daemon/session/kernel/child state in a dedicated job namespace. Do not use a shared user-wide daemon as the authority or cleanup boundary. A client disconnect is not a stopped job. Do not run a global shutdown that can terminate another owner's sessions. Confirm full process/sandbox termination or report unconfirmed occupancy.

Native children are disabled for the initial shipping profile unless their count, depth, model eligibility, usage and writable behavior are enforceable. A prompt asking a child to be read-only is not confinement. Children may inspect immutable snapshots or produce private scratch artifacts; one current writer controls the selected candidate tree. Nested accounting must distinguish inclusive from exclusive root totals.

The previous selected RLM documentation describes child spawn as admission and subsequent answers as messages/files; treat both as observations, not trusted task completion. Record actual child model/configuration rather than assume the parent's model is always fixed for descendants. Native schedules, cross-root messages and automatic continuation cannot outlive the hub grant or allocate shared team work. [J R4, H3]

Allow persistent working variables during a bounded attempt. Shared refinement is not automatic: a reusable skill or profile follows the reviewed promotion path in §22. A profile that cannot meet its declared cancellation, credential or child-write boundary is pilot-only, read-only, or ineligible. Do not weaken the overall product guarantee to make an adapter badge green.

### 19.3 OpenJarvis: prove the subset before adopting it

The prior pinned inspection found Rust workspace code and a React/Vite frontend alongside Python distribution paths. Do not dismiss it as Python-only or infer that the entire application is a single Rust service. The revisited `InferenceEngine` trait is inference generation/stream/discovery, not the complete coding-worker or task-ownership protocol. [J R3; W10]

Evaluate engine/core reuse with a minimal consumer: dependency/build footprint, explicit model endpoint, stream/error handling, bounded blocking calls and timeout/cancellation behavior. An aborted local future does not prove the inference server stopped GPU work. If an existing certified harness already calls the local endpoint directly, do not add a redundant forwarding service.

A local profile records model artifact/version, quantization when known, engine, hardware class, context/output limits, warm/cold state and concurrency. Quality and total accepted-outcome cost must be measured. Local execution is not automatically competent, free or faster. Mutable trace outcomes may support analysis but cannot replace the immutable acceptance ledger. No training pipeline or learned router is an MVP prerequisite.

### 19.4 QM patterns and explicit fork criteria

Adopt source/audience labels, current membership checks, separate personal and shared context, and human-only privilege-changing operations. Sharing a summarized private lesson still shares information derived from that source; default to the original restrictions unless an authorized declassification path says otherwise. Construct shared reports from records already accessible to the intended audience. Model discretion is not an output disclosure boundary. [W9; J R2, H5]

A currently operated QM instance may call BulletFarm through a scoped client capability. It cannot become another owner of task state or approve its own elevated requests. The first install does not deploy QM.

**No wholesale fork is recommended now.** Fork a specific upstream only after a failing required conformance test identifies a necessary patch and a bounded maintenance owner. Record exact source revision, scope of the patch, applicable license notices and upgrade tests. A complete TypeScript/Python-to-Rust conversion is a rewrite, not a low-cost fork. No remote repository fork, branch, deployment or source mutation was performed during this review.

The earlier YC video review used an automatic transcript mirror. Here it remains historical motivation for harness/context evaluation, not newly verified audiovisual evidence or a quantitative performance source. The engineering decisions rest on explicit requirements and inspected interfaces, not transferred benchmark multipliers.


## 20. One interface and one command API

### 20.1 CLI contract

These are required target commands for the implementation, not executables supplied by this document package.

```text
bf init
bf serve --local
bf connect <team-endpoint>
bf runner enroll / bf runner start
bf doctor
bf submit <task.json>
bf plan "Explore ..."
bf run "Implement ..." --goal pr_ready
bf chat <mission>
bf status --mine / --team
bf why <task>
bf take <task>
bf submit-human <task> --commit <oid>
bf return <task> --checkpoint <artifact-id>
bf revise <task> --file <task.json>
bf decide <decision> --option <id>
bf pause <mission>
bf stop <mission>
bf cancel <task-or-mission>
bf report --window <id>
bf export --out <directory>
```

`plan` never starts implementation. `run` uses a selected standing grant; it cannot manufacture one. `pause` stops new dispatch and new privileged sends, allowing already authorized local jobs to drain within their existing limits. `stop` additionally requests active termination; resume is a separate current-authority decision. `cancel` revokes further work/delivery for the target and reconciles effects already dispatched. None promises a remote undo. `doctor`, status and explanations work without model calls.

### 20.2 Command envelope

All machine-facing new contracts use `schema_version: 3`. A command has `command_id`, `kind`, `target_id`, `expected_version`, and a kind-specific payload. For creation, target and expected version are null; updates require a target and positive expected version. The authenticated actor is not a payload field. Use one command endpoint, so CLI, web and later channels do not implement separate business rules.

| Command kind | Required payload / guard |
|---|---|
| `create_mission` | Proposed owner/domain, exact requirement artifact, goal and grant reference; current caller may request them |
| `activate_plan` | Exact published plan artifact, Git identity and digest; valid graph and current grant |
| `pause`, `stop`, `cancel` | Optional reason; expected current target version |
| `take` | `checkpoint_preference` of last verified or last durable; current human owner/delegate |
| `submit_human` | Exact candidate artifact and contract digest; current human contribution authority |
| `return` | Exact checkpoint artifact and handoff artifact; current owner and remaining allowance |
| `revise` | New contract artifact/revision and old digest; no implicit allowance increase |
| `resolve_decision` | Option and exact subject digest; designated actor class, not just matching human owner ID |
| `grant_allowance` | Grant reference, positive additional invocations or spend, currency and reason; human-only authorized action |
| `request_merge` | Candidate, selection version, observed head/base; actual native protection and current approvals |

`202 Accepted` means a durable operation record exists, not that external work finished. Replaying the same actor/key/body returns that operation. Reusing the key for different content returns conflict. Recheck current read authorization before returning a historical replay response; idempotency does not leak a record after access was revoked.

Errors include `INVALID_CONTRACT`, `UNKNOWN_DEPENDENCY`, `DEPENDENCY_INVALID`, `UNKNOWN_PROFILE`, `CHECK_MISSING`, `STALE_VERSION`, `STALE_GENERATION`, `STALE_SELECTION`, `POLICY_DENIED`, `AUTH_REQUIRED`, `BUDGET_UNAVAILABLE`, `RESOURCE_CONFLICT`, `OUTCOME_UNKNOWN` and `STORAGE_UNAVAILABLE`. Return the smallest useful authorized explanation and a correlation ID. Do not retry every error identically.

### 20.3 Endpoints and producer binding

```text
POST /v3/commands
GET  /v3/operations/{id}
GET  /v3/missions/{id}
GET  /v3/tasks/{id}
GET  /v3/tasks/{id}/why
GET  /v3/work?owner=&domain=&phase=
GET  /v3/events?after={cursor}
POST /v3/runners/enroll
POST /v3/runners/{id}/poll
POST /v3/jobs/{id}/heartbeat
POST /v3/jobs/{id}/events
POST /v3/jobs/{id}/result
POST /v3/artifacts/uploads
GET  /v3/artifacts/{issued-id}
POST /v3/hooks/forge
```

Job endpoints validate enrolled producer, assignment generation, epoch and purpose. A verifier result is accepted only for the expected check job and subject; ordinary runner credentials cannot mint verifier evidence. Grant changes and human decisions may share the command transport, but their authorization requires the distinct human path. There is no public arbitrary-shell or arbitrary-SQL endpoint.

SSE uses committed sequence cursors and a permission-filtered snapshot on compaction/reconnect. An expired or unauthorized subscription ends or refreshes appropriately. Provider-local timestamps are observations, not global ordering authority. Artifact content hashes establish identity, not permission; every issued artifact ID is scope-authorized.

### 20.4 Web workbench

One React workbench has Mine, Domain and Team filters; a compact Foreman input; a work table; and a task/mission drawer. Show outcomes delivered, decisions needing the viewer, review-ready changes, active work, holds/unknowns, and spend/exposure. Logs are drill-down content, not the product home screen.

Every decision card states the exact question, why it matters, recommended action, alternatives, owner, affected version, relevant evidence and remaining allowance. Every significant state has a source and observation time. Avoid synthetic completion percentages derived from tokens or task count. Disabled capabilities and stale observations are explicit.

The same page must remain useful when all models fail: inspect state, determine ownership, see pending operations and request pause/stop. Accessibility includes keyboard navigation, readable contrast, semantic tables and no status conveyed by color alone. Usability tests ask an engineer to locate the next actionable blocker without opening raw vendor logs.


## 21. Production, channel boundaries and the owned forge

The initial team product can end at protected PR-ready or merged work. Production is a separately admitted mission goal with an existing release adapter. Build one immutable artifact from the admitted source, verify staging, authorize the exact artifact/environment/configuration, promote that same artifact, and observe deployed identity and health for the declared interval.

A successful trigger is not deployment; deployment is not completed healthy observation. Keep signing and production credentials outside coding jobs. Reversible application rollout can use a tested standing rollback policy. Irreversible data migrations require an explicit forward/restore strategy and owner; binary rollback does not restore data. Multi-repository release compatibility is a manifest of exact component versions, not an atomicity claim.

Slack is the first optional channel. Verify its real transport signatures and replay behavior, map workspace/user identity to BulletFarm rights, and bind decisions to current exact subjects. Grok Bot is a potential thin client through a capability-tested path, not the Grok Build executor and not a new scheduler. SMS starts with opt-in notifications and authenticated decision links; a phone number and a generic yes do not authorize high-impact actions. These channels cannot introduce a second task database or broader authority than the same user has in the web client.

The owned GitHub-compatible forge is an advantage at the enforcement boundary. Initially use existing APIs and protections. Later select one measured improvement: native task/candidate/evidence metadata; visible registered change intent; or server-enforced current subject/fence at publication or integration. Source access and endpoint parity do not establish those semantics; test the actual implementation.

Preserve ordinary Git/PR interoperability. No BulletGit replacement, new object store, custom merge queue, full GitHub UI clone or new deployment engine is a prerequisite. Unsupported automatic merge remains accurately disabled rather than hidden behind a supposed safe client-side workaround.


## 22. Learning that cannot authorize itself

### 22.1 Shipping, shadow and wildcard

Shipping has one selected current candidate and task-owned conflict reservation. A shadow evaluation runs the same frozen task or cohort in a private workspace with an experiment budget, but no shipping claim, candidate selection, forge token or release authority. Shadow code is not automatically the winner because it finished first. Adoption requires a current authorized shipping candidate and fresh applicable checks.

Wildcard work may test a genuinely different architecture, not merely a different adapter. Wrap it in an authorized mission, resource/data boundary and evidence import path. It may break contingent batching or decomposition conventions, but not attribution, protected acceptance or authority. Unconfigured experiment spending is zero, not an implicit percentage of the team's money.

### 22.2 Evaluation contract

Before a comparison, declare the task class, hypothesis, profiles, quality floor, material cost/latency objective, selection policy, frozen task/base/gates/environment, initial learned state, evaluator, stopping/precision plan and promotion criteria. Hold batch position and warm state constant when comparing harnesses alone, or explicitly label them as part of the treatment.

Use a controlled logical evaluation pool. Domain-selected production rows remain operational observations unless their assignment satisfies the comparison design. One physical server alone does not remove selection bias. Record task family, owner/site, gate revision, model/harness/profile, resource contention and integration order. Failures remain in the denominator.

Report complete cost, accepted behavior, repair burden, human handling and latency. A shadow that never deployed has no measured production survival; simulation against a common integration snapshot is a different metric. Smaller diffs and model grades are secondary diagnostics. Different frontier models can share errors. A short sample is a pilot, not a universal ranking; choose sample size and stopping rules for the claimed effect and uncertainty.

### 22.3 Three levels of learning permission

**Task-local adaptation:** permitted working notes, variables and temporary helper code may change inside the existing job budget and sandbox. They remain untrusted scratch. Record initial/final state where it matters. They cannot alter the task contract, protected checks, grant or shared skill store.

**Reusable proposal:** create a concise lesson with symptom, applicable class/revision, evidence, suggested prompt/skill/profile change, known counterexamples and a test of benefit. Use normal tasks/decisions/artifacts; no permanent meta-agent is required. A failed attempt is not a universal rule that an approach never works.

**Shared promotion:** review a versioned candidate profile/skill, run fixed negative and held-out cases, obtain designated current approval, canary on new jobs, observe outcomes, then promote or roll back. Do not change live profiles or acceptance tests to manufacture a win. Executable skill code and expanded egress follow normal security/code review. A private lesson retains its audience restrictions.

A learned route can be considered later after the explicit routing table and measurement are reliable. No bandit, training cluster or automatic policy evolution enters the MVP merely because an upstream supports it.

### 22.4 Replacement is a successful outcome

The named service owner periodically reviews maintenance burden, gate adequacy, human handling, accepted-outcome cost and whether a simpler system now preserves the durable assets. Test a replacement using fixed cases and a portable export. Loyalty to the current dispatcher is not the objective. A review cadence is an operational requirement to configure; no actual calendar event has been created by this document.


## 23. Operations, migration and remaining risk

### 23.1 Onboarding and upgrades

Inspect the actual repository, gates, current implementation, ownership, data policy, isolation and forge before replacing code. Preserve existing conforming behavior. Resolve the minimum first-task check contract and actual owner/grant/budget. Missing gates produce a coverage-improvement task or a narrower assisted capability—not a fake pass.

Pin the actual Rust/TypeScript toolchains, lockfiles, harness executable digests and effective profiles during the build. A source snapshot does not identify the installed binary. Canary upgrades against protocol, permission, cancellation, accounting and negative fixtures; do not change semantics in the middle of a job. Roll back by selecting a previously certified version for new jobs, not editing an active profile.

Watch unknown-effect age, physical unconfirmed occupancy, mandatory-check gaps, review pressure, dependency invalidations, scope contention, actual/estimated cost and escalation age. Increasing writer count is not the default answer to blocked review or verification.

### 23.2 Portable exports and version migration

`bf export` produces a versioned manifest, immutable contracts/decisions/profiles, permitted gate references, ownership history, outcome/cost JSONL, evidence manifests and retained artifacts. Include exact hashes and explicit missing/withheld records. Never export usable secrets or reactivate historical grants/leases as live authority.

V3 imports A–J content only through a named source-format adapter. Preserve raw source bytes and original digest semantics. Map old Attempt and mission ModelJob records to purpose-tagged Job history. For J, resolve its ExecutionBinding into the new direct profile reference after verifying the old relationship. Do not reinterpret an old semantic JSON digest as an exact-byte digest, or silently call a v1 payload v3.

Import begins paused/proposed with new authority. Stop the old overlapping allocator, reconcile existing PRs, resolve owners and missing dependencies, issue new grants and activate only after validation. Maintain migration tooling, not two indefinitely competing runtime contract systems. Existing external human-controlled work may continue outside coverage, with clear observation labels.

### 23.3 What remains uncertain

Task decomposition is still a reasoning problem. Gate adequacy is finite and workload-specific. Provider credential separation may be insufficient for a sensitive profile. Scope claims cannot detect every semantic conflict. The single hub can be unavailable. Native forge behavior may fail conformance. Human registration or review burden can outweigh benefits. Economical routes can be more expensive after repairs.

The response to each is explicit: combine work, improve or narrow gates, disable unsuitable profiles, use current integration checks, recover paused authority, retain protected manual integration, simplify human workflows and disable losing routes. Do not address every weakness with another permanent agent or service.

The design score does not resolve these uncertainties. Actual code, fault receipts, clean-room rebuilding and representative outcomes determine release readiness.


# Part V — Build, test and score

## 24. One ordered implementation backlog

This release contains **22 core work packages and eight deferred work packages**. The package count is an implementation organization, not a promise that every ticket fits one economical-agent invocation. Split a package only after mapping it onto the actual code, retaining its acceptance and prerequisite obligations. The new BF3 identifiers explicitly supersede competing BF/BF2 sequences; old task IDs remain in source and migration maps, not as a second live backlog.

| Gate | Target | Evidence required |
|---|---|---|
| G0 | Gate diagnostic and deterministic substrate | Contracts, identity, budgets, Git activation and fake jobs work without paid providers |
| G1 | One person's useful delivery loop | One real supplied task reaches independently checked draft PR; interruption and replay preserve truthful state |
| G2 | Individual-and-team MVP | Two owners/runners/coding-provider families; Foreman, takeover, fair claims, web/CLI, exact integration, accounting and restore |
| G3 | Measured extensions | Each optional adapter, learning, release, channel, component or topology change proves its own value and boundary |

The initial fake-backed implementation must compile and run without provider keys or remote permissions. An existing authorized development workflow builds the substrate; BulletFarm does not need to exist before it can build itself. Live capabilities require actual repository mapping, owners, grants, budgets, toolchain pins and conformance receipts. Null backlog bindings are unresolved, not unlimited authority.

### 24.1 Build-agent procedure

Read the final normative specification, `START_HERE.md`, v3 schemas and examples. Select one dependency-ready BF3 package. Map its module scope onto existing source, preserving conforming code. Write the relevant failing fixture first. Implement only its acceptance obligations. Run the project's actual checks, capture exit status and evidence, and obtain independent review for protected changes. Update `BUILD_CHECKPOINT.json` with exact spec/source identities, completed packages, checks, known gaps and the next eligible package.

Do not reopen the database, allocator, scope, digest, grant, candidate or delivery choices silently. An actual contradiction is a specification defect to record and resolve. Ordinary internal helper names are implementation details; changing permission or completion semantics is not.

The future implementation must provide `scripts/check` covering Rust formatting/lint/tests, schema fixtures and web typecheck/test/build, using pinned lockfiles. It must provide `bf demo --fixture basic` and `bf demo --fixture interrupted_publish` with fakes, no network calls and deterministic receipts. A fake provider returning a predetermined patch tests orchestration, not model quality.

A fresh-agent reconstruction test receives this kit and an empty source tree, not earlier conversations or the existing implementation. The independent reviewer tests contract behavior and records every assumption the builder had to guess. This reconstruction is required future evidence; generating the kit does not perform it.


### 24.2 Work-package map

| ID | Gate | Deliverable | Depends on |
|---|---|---|---|
| BF3-001 | G0 | Inspect existing code and gate adequacy | — |
| BF3-002 | G0 | Define v3 contracts, profiles and pure validators | BF3-001 |
| BF3-003 | G0 | Persist state, jobs, commands and effect identity | BF3-002 |
| BF3-004 | G0 | Authenticate principals and expose the minimal CLI | BF3-003 |
| BF3-005 | G0 | Reserve execution and completion budgets | BF3-003, BF3-004 |
| BF3-006 | G0 | Publish and activate Git contracts with dependency validity | BF3-002, BF3-003, BF3-004 |
| BF3-007 | G0 | Build the adapter seam and deterministic fakes | BF3-002 |
| BF3-008 | G1 | Isolate workspaces and track physical execution | BF3-004, BF3-007 |
| BF3-009 | G1 | Implement transactional claims and all-job recovery | BF3-003, BF3-004, BF3-005, BF3-006, BF3-007, BF3-008 |
| BF3-010 | G1 | Certify one incumbent executor and the clean-room fallback | BF3-007, BF3-008, BF3-009 |
| BF3-011 | G1 | Seal candidates and run stage-specific independent checks | BF3-006, BF3-008, BF3-009 |
| BF3-012 | G1 | Publish and reconcile a stable draft PR | BF3-004, BF3-009, BF3-011 |
| BF3-013 | G1 | Demonstrate the first useful developer loop | BF3-010, BF3-011, BF3-012 |
| BF3-014 | G2 | Add owner lanes, fairness and external-work awareness | BF3-009, BF3-012, BF3-013 |
| BF3-015 | G2 | Implement human takeover, return and respecification | BF3-011, BF3-012, BF3-014 |
| BF3-016 | G2 | Certify a second coding-provider family | BF3-010, BF3-013 |
| BF3-017 | G2 | Build Foreman planning and scoped context | BF3-006, BF3-013, BF3-014, BF3-016 |
| BF3-018 | G2 | Ship the single React workbench and shared API | BF3-004, BF3-014, BF3-015, BF3-017 |
| BF3-019 | G2 | Certify native integration and assembled outcomes | BF3-011, BF3-012, BF3-014, BF3-016 |
| BF3-020 | G2 | Report complete cost, windows and calibrated routes | BF3-005, BF3-014, BF3-016 |
| BF3-021 | G2 | Rehearse restore, retention, migration and export | BF3-003, BF3-009, BF3-011, BF3-012, BF3-015, BF3-020 |
| BF3-022 | G2 | Run team release, fault and independent rebuild gates | BF3-015, BF3-017, BF3-018, BF3-019, BF3-020, BF3-021 |
| BF3-023 | G3 | Controlled shadow evaluations and shared profile promotion | BF3-022 |
| BF3-024 | G3 | Add the Grok Build executor | BF3-016, BF3-022 |
| BF3-025 | G3 | Connect one existing production pipeline | BF3-019, BF3-022 |
| BF3-026 | G3 | Add signed Slack thin-client commands | BF3-018, BF3-022 |
| BF3-027 | G3 | Validate Grok Bot and SMS client capabilities | BF3-026 |
| BF3-028 | G3 | Evaluate the OpenJarvis engine/core boundary | BF3-013, BF3-020 |
| BF3-029 | G3 | Test architectural replacement or federation need | BF3-021, BF3-022, BF3-023 |
| BF3-030 | G3 | Implement one justified forge-native improvement | BF3-019, BF3-022 |


### 24.3 Detailed package acceptance

Each item below is a build obligation, not a claim that the code or check currently exists.


#### BF3-001 — Inspect existing code and gate adequacy

**Gate:** G0. **Prerequisites:** None.

**Outcome:** Freeze a source/owner/deployment decision and first meaningful independent check.

**Proposed source scope:** `docs/onboarding/`, `fixtures/gates/`.

1. **BF3-001-AC01** — Known-good fixture passes and injected behavioral defect fails without removing its oracle.
2. **BF3-001-AC02** — Missing ownership, protection, credentials or checks remain unresolved; no live spending or mutation.
3. **BF3-001-AC03** — Record which existing modules can be reused and which task classes remain assisted-only.

**Scenario links:** AT-032


#### BF3-002 — Define v3 contracts, profiles and pure validators

**Gate:** G0. **Prerequisites:** BF3-001.

**Outcome:** Implement the specified wire shapes and semantic validation.

**Proposed source scope:** `src/domain/`, `schemas/`, `tests/contracts/`.

1. **BF3-002-AC01** — Reject unknown schema versions, duplicate keys, missing IDs, cycles and unresolved mandatory checks.
2. **BF3-002-AC02** — Task, planning, verifier and shadow job variants obey their role-specific requirements.
3. **BF3-002-AC03** — Exact-byte digests and migration provenance cannot be silently confused with old canonical digests.

**Scenario links:** AT-001, AT-002, HF01


#### BF3-003 — Persist state, jobs, commands and effect identity

**Gate:** G0. **Prerequisites:** BF3-002.

**Outcome:** Create one transactional state/event/outbox command kernel.

**Proposed source scope:** `src/store/`, `migrations/`, `tests/store/`.

1. **BF3-003-AC01** — A failed transition rolls back state and audit/outbox together.
2. **BF3-003-AC02** — Same actor/key/body returns one durable operation; changed body conflicts.
3. **BF3-003-AC03** — Immutable contracts/candidates/evidence and job/selection relationships enforce the structural reference.

**Scenario links:** AT-003, AT-004, HF16


#### BF3-004 — Authenticate principals and expose the minimal CLI

**Gate:** G0. **Prerequisites:** BF3-003.

**Outcome:** Give users model-independent scoped commands before enabling live execution.

**Proposed source scope:** `src/auth/`, `src/api/commands.rs`, `src/cli/`, `tests/auth/`.

1. **BF3-004-AC01** — An agent initiated by an admin cannot exercise interactive human-only authority.
2. **BF3-004-AC02** — Repository event/artifact reads and historical operation replays recheck current access.
3. **BF3-004-AC03** — Local bootstrap is private; runner enrollment is revocable; body owner fields never authenticate.

**Scenario links:** AT-005, AT-006, HF17, CF06


#### BF3-005 — Reserve execution and completion budgets

**Gate:** G0. **Prerequisites:** BF3-003, BF3-004.

**Outcome:** Admit jobs only with bounded allowance and a mandatory completion reserve.

**Proposed source scope:** `src/control/budget.rs`, `tests/budget/`.

1. **BF3-005-AC01** — Concurrent admission conserves all applicable account holds without double-counting hierarchical limits.
2. **BF3-005-AC02** — Writer use cannot consume the reserved verifier/reviewer envelope.
3. **BF3-005-AC03** — Unknown usage stays exposed; actual overruns are recorded and further admission held.

**Scenario links:** AT-007, AT-008, AT-009, CF05


#### BF3-006 — Publish and activate Git contracts with dependency validity

**Gate:** G0. **Prerequisites:** BF3-002, BF3-003, BF3-004.

**Outcome:** Activate exactly one confirmed complete plan graph.

**Proposed source scope:** `src/control/plans.rs`, `src/forge/intent.rs`, `tests/activation/`.

1. **BF3-006-AC01** — Crash between Git and SQL leaves inert metadata, not a runnable partial graph.
2. **BF3-006-AC02** — Unknown, cancelled or known-reverted prerequisites do not satisfy readiness.
3. **BF3-006-AC03** — Metadata-only refs use expected-old forward updates and do not trigger code deployment.

**Scenario links:** AT-010, AT-011


#### BF3-007 — Build the adapter seam and deterministic fakes

**Gate:** G0. **Prerequisites:** BF3-002.

**Outcome:** Implement bounded machine-job framing and fake provider/forge behavior.

**Proposed source scope:** `src/adapters/`, `fixtures/protocol/`, `tests/adapters/`.

1. **BF3-007-AC01** — Malformed or truncated final frames cannot become success.
2. **BF3-007-AC02** — Admission, execution end, candidate output and physical cleanup are distinct events.
3. **BF3-007-AC03** — All fixture paths run without model credentials or network access.

**Scenario links:** AT-012


#### BF3-008 — Isolate workspaces and track physical execution

**Gate:** G1. **Prerequisites:** BF3-004, BF3-007.

**Outcome:** Use private writable Git state inside a constrained Linux execution boundary.

**Proposed source scope:** `src/runner/`, `tests/isolation/`.

1. **BF3-008-AC01** — Jobs cannot read/write other owners, host credentials, hub state or privileged sockets.
2. **BF3-008-AC02** — Cleanup uses exact sandbox incarnation; unconfirmed processes remain occupied.
3. **BF3-008-AC03** — Native daemons and children cannot escape the registered job lifetime or shared write restrictions.

**Scenario links:** AT-013, AT-014, HF06, HF09, CF01


#### BF3-009 — Implement transactional claims and all-job recovery

**Gate:** G1. **Prerequisites:** BF3-003, BF3-004, BF3-005, BF3-006, BF3-007, BF3-008.

**Outcome:** Own one shipping writer while independently supervising every machine job.

**Proposed source scope:** `src/control/scheduler.rs`, `src/control/jobs.rs`, `tests/ownership/`.

1. **BF3-009-AC01** — Acquire all conflicting resources or none; a replay cannot spawn a second accepted incarnation.
2. **BF3-009-AC02** — Author sealing/revocation does not release pending-change ownership or unconfirmed physical capacity.
3. **BF3-009-AC03** — Late planner/reviewer/verifier results cannot replace newer job assignments.

**Scenario links:** AT-016, AT-017, AT-018, AT-019, AT-020, AT-022, HF07, HF08, CF03


#### BF3-010 — Certify one incumbent executor and the clean-room fallback

**Gate:** G1. **Prerequisites:** BF3-007, BF3-008, BF3-009.

**Outcome:** Certify one live executor, using Prime RPC or an existing conforming incumbent; Codex headless is the reference fallback.

**Proposed source scope:** `src/adapters/selected.rs`, `fixtures/providers/`, `tests/provider_conformance/`.

1. **BF3-010-AC01** — Only one selected transport is required; no TUI/menu automation.
2. **BF3-010-AC02** — Fresh session identity is proved, including a Prime session-veto response where applicable.
3. **BF3-010-AC03** — Cancellation, auth/quota, nested usage and required credential boundaries pass the selected profile fixtures.

**Scenario links:** AT-026, HF02, HF03, HF04, HF05


#### BF3-011 — Seal candidates and run stage-specific independent checks

**Gate:** G1. **Prerequisites:** BF3-006, BF3-008, BF3-009.

**Outcome:** Create exact candidates and trusted check jobs that outlive author authority.

**Proposed source scope:** `src/evidence/`, `tests/verification/`.

1. **BF3-011-AC01** — Author PASS, missing discovery, wrong producer or changed subject cannot satisfy checks.
2. **BF3-011-AC02** — Verification runs under its own identity/deadline and survives author exit.
3. **BF3-011-AC03** — Prepublication and CI-only obligations are distinguished so publication is neither blocked circularly nor called fully verified too early.

**Scenario links:** AT-021, AT-030, AT-031, AT-033, AT-034, AT-044, HF11, CF02


#### BF3-012 — Publish and reconcile a stable draft PR

**Gate:** G1. **Prerequisites:** BF3-004, BF3-009, BF3-011.

**Outcome:** Perform credential-separated candidate import and recoverable PR publication.

**Proposed source scope:** `src/forge/`, `tests/effects/`.

1. **BF3-012-AC01** — Candidate config/hooks/helpers cannot execute inside the privileged importer.
2. **BF3-012-AC02** — A lost create response adopts the matching remote PR or remains unknown before retry.
3. **BF3-012-AC03** — Current selection/grant and remote head are required; human edits/closure are not overwritten.

**Scenario links:** AT-015, AT-023, AT-024, AT-035, AT-036, AT-037


#### BF3-013 — Demonstrate the first useful developer loop

**Gate:** G1. **Prerequisites:** BF3-010, BF3-011, BF3-012.

**Outcome:** A supplied task yields one independently checked draft PR and a truthful report.

**Proposed source scope:** `tests/e2e/`, `src/cli/demo.rs`, `docs/first_win.md`.

1. **BF3-013-AC01** — Fixture demo requires no keys and clearly labels its fake executor/forge.
2. **BF3-013-AC02** — Live demonstration records exact versions, checks, costs and unresolved limits rather than inheriting fixture certification.
3. **BF3-013-AC03** — Author can stop while verification/delivery continue; replay and interruption do not duplicate logical publication.

**Scenario links:** Package-specific acceptance above; the integrated release gate adds cross-cutting cases.


#### BF3-014 — Add owner lanes, fairness and external-work awareness

**Gate:** G2. **Prerequisites:** BF3-009, BF3-012, BF3-013.

**Outcome:** Coordinate two owners without taking their product authority.

**Proposed source scope:** `src/control/owners.rs`, `src/forge/observations.rs`, `tests/team/`.

1. **BF3-014-AC01** — Cross-domain work requires applicable grants; shared lane cannot steal owned tasks.
2. **BF3-014-AC02** — Overlap waits with an accountable reason while disjoint work proceeds.
3. **BF3-014-AC03** — External PR observations reconcile; unregistered unpushed work is explicitly outside coverage.

**Scenario links:** AT-040


#### BF3-015 — Implement human takeover, return and respecification

**Gate:** G2. **Prerequisites:** BF3-011, BF3-012, BF3-014.

**Outcome:** Transfer known work into separate human/agent workspaces without hiding history.

**Proposed source scope:** `src/control/human.rs`, `src/cli/human.rs`, `tests/human/`.

1. **BF3-015-AC01** — Old writer is fenced; latest known checkpoint is provided and uncaptured work is disclosed.
2. **BF3-015-AC02** — Human submission and return follow identical independent checks.
3. **BF3-015-AC03** — Respecification needs explicit added allowance; lifetime generations, costs and parent requirements remain.

**Scenario links:** AT-038, AT-039


#### BF3-016 — Certify a second coding-provider family

**Gate:** G2. **Prerequisites:** BF3-010, BF3-013.

**Outcome:** Prove actual cross-provider implementation, not two planning outputs.

**Proposed source scope:** `src/adapters/second.rs`, `tests/provider_conformance/`.

1. **BF3-016-AC01** — The same task can restart from an exact checkpoint with a fresh eligible second-provider context.
2. **BF3-016-AC02** — Data policy, startup hooks, permission handling, cancellation and accounting remain explicit.
3. **BF3-016-AC03** — A second harness name using the same provider alone does not satisfy this gate.

**Scenario links:** AT-027


#### BF3-017 — Build Foreman planning and scoped context

**Gate:** G2. **Prerequisites:** BF3-006, BF3-013, BF3-014, BF3-016.

**Outcome:** Convert authorized mission intent into a bounded, accepted task graph and fresh executions.

**Proposed source scope:** `src/control/foreman.rs`, `src/control/context.rs`, `tests/planning/`.

1. **BF3-017-AC01** — Planning before a task exists uses a mission job and the same spending controls.
2. **BF3-017-AC02** — Fast/Standard/Deep flows are bounded and every mandatory requirement maps to acceptance.
3. **BF3-017-AC03** — Private memory cannot enter a wider audience automatically; agents cannot self-approve escalations.

**Scenario links:** AT-025, HF12


#### BF3-018 — Ship the single React workbench and shared API

**Gate:** G2. **Prerequisites:** BF3-004, BF3-014, BF3-015, BF3-017.

**Outcome:** Expose Mine/Domain/Team, actionable decisions and evidence without native terminals.

**Proposed source scope:** `web/`, `src/api/`, `tests/ui/`.

1. **BF3-018-AC01** — UI/CLI agree on committed state; stale commands and unauthorized reads fail server-side.
2. **BF3-018-AC02** — Slow clients and large output cannot stall control/results; reconnect repairs state.
3. **BF3-018-AC03** — Keyboard-accessible status and why remain useful without any model provider.

**Scenario links:** HF19


#### BF3-019 — Certify native integration and assembled outcomes

**Gate:** G2. **Prerequisites:** BF3-011, BF3-012, BF3-014, BF3-016.

**Outcome:** Use the actual protected integration subject and final mission checks.

**Proposed source scope:** `src/forge/integration.rs`, `src/evidence/mission.rs`, `tests/integration/`.

1. **BF3-019-AC01** — Wrong issuer, moved target or incompatible individually-green changes cannot use stale evidence.
2. **BF3-019-AC02** — Unsupported merge capabilities stay disabled; no homemade queue is introduced.
3. **BF3-019-AC03** — Known reverted prerequisites hold dependents even when original commits remain ancestors.

**Scenario links:** AT-041, AT-042, CF04


#### BF3-020 — Report complete cost, windows and calibrated routes

**Gate:** G2. **Prerequisites:** BF3-005, BF3-014, BF3-016.

**Outcome:** Make accepted-outcome cost and attention visible without a model polling loop.

**Proposed source scope:** `src/control/routing.rs`, `src/control/report.rs`, `tests/accounting/`.

1. **BF3-020-AC01** — Inclusive root/child meters are not double-charged and missing use is never zero.
2. **BF3-020-AC02** — Window end preserves useful work within the existing bounded drain authority.
3. **BF3-020-AC03** — Review/verification pressure reduces new writing; routing uses eligible calibrated profiles rather than a cheap-call quota.

**Scenario links:** AT-028, AT-029, AT-043, AT-047, HF10, CF08


#### BF3-021 — Rehearse restore, retention, migration and export

**Gate:** G2. **Prerequisites:** BF3-003, BF3-009, BF3-011, BF3-012, BF3-015, BF3-020.

**Outcome:** Recover durable work without reviving stale authority.

**Proposed source scope:** `src/store/recovery.rs`, `src/control/export.rs`, `tests/restore/`.

1. **BF3-021-AC01** — Restore starts paused with old publisher quiescence/revocation and external authority rotation.
2. **BF3-021-AC02** — Missing artifacts stay unavailable; cleanup cannot erase live or unrelated work.
3. **BF3-021-AC03** — Old contracts/profile bindings import with provenance but no live grants/leases; export is portable.

**Scenario links:** AT-045, AT-046, HF20, CF07


#### BF3-022 — Run team release, fault and independent rebuild gates

**Gate:** G2. **Prerequisites:** BF3-015, BF3-017, BF3-018, BF3-019, BF3-020, BF3-021.

**Outcome:** Prove the intended two-person product and document what remains unproven.

**Proposed source scope:** `tests/acceptance/`, `docs/pilot/`, `scripts/check`.

1. **BF3-022-AC01** — Two owners/runners/provider families exercise overlap, takeover, long checks, expired grants and interruption.
2. **BF3-022-AC02** — A fresh agent reconstructs the fake-backed reference from this package alone and records every missing assumption.
3. **BF3-022-AC03** — Representative pilot reports failures, complete cost/human handling and uncertain observations; no paper score substitutes for runtime evidence.

**Scenario links:** AT-052


#### BF3-023 — Controlled shadow evaluations and shared profile promotion

**Gate:** G3. **Prerequisites:** BF3-022.

**Outcome:** Compare immutable execution treatments without blocking or taking over shipping.

**Proposed source scope:** `src/control/evaluations.rs`, `tests/evaluations/`.

1. **BF3-023-AC01** — No shadow selection, shipping claim or publication authority.
2. **BF3-023-AC02** — Fixed or explicitly adaptive initial state, gate/holdout and stopping criteria remain declared.
3. **BF3-023-AC03** — Shared lessons require current owner approval, canary and rollback; running jobs remain pinned.

**Scenario links:** AT-048, AT-049, HF13, HF14, HF18


#### BF3-024 — Add the Grok Build executor

**Gate:** G3. **Prerequisites:** BF3-016, BF3-022.

**Outcome:** Add one official machine transport only after the common boundary works.

**Proposed source scope:** `src/adapters/grok.rs`, `tests/provider_conformance/grok/`.

1. **BF3-024-AC01** — Freshness, permission, cancel, auth/quota and output tests pass.
2. **BF3-024-AC02** — Grok Bot is not treated as the same executor.
3. **BF3-024-AC03** — No provider-specific scheduler or menu automation is added.

**Scenario links:** Package-specific acceptance above; the integrated release gate adds cross-cutting cases.


#### BF3-025 — Connect one existing production pipeline

**Gate:** G3. **Prerequisites:** BF3-019, BF3-022.

**Outcome:** Promote and observe one exact artifact through existing governance.

**Proposed source scope:** `src/forge/release.rs`, `tests/release/`.

1. **BF3-025-AC01** — Artifact/environment substitution invalidates approval.
2. **BF3-025-AC02** — No coding job gains signing or production credentials.
3. **BF3-025-AC03** — Observed healthy is distinct from trigger/deployment; irreversible data recovery remains explicit.

**Scenario links:** AT-050


#### BF3-026 — Add signed Slack thin-client commands

**Gate:** G3. **Prerequisites:** BF3-018, BF3-022.

**Outcome:** Provide scoped submission/status/decision links through the same controller.

**Proposed source scope:** `src/api/slack.rs`, `tests/channels/slack/`.

1. **BF3-026-AC01** — Verify signatures/replay and mapped actor identity.
2. **BF3-026-AC02** — No Slack-specific queue or approval bypass.
3. **BF3-026-AC03** — Sensitive required approvals use the designated human path and exact subject.

**Scenario links:** AT-051


#### BF3-027 — Validate Grok Bot and SMS client capabilities

**Gate:** G3. **Prerequisites:** BF3-026.

**Outcome:** Establish the real supported integration before enabling narrow clients.

**Proposed source scope:** `docs/channels/`, `tests/channels/`.

1. **BF3-027-AC01** — Unsupported APIs remain unsupported, not silently browser-automated.
2. **BF3-027-AC02** — SMS defaults to opt-in notifications/authenticated links, not generic yes approvals.
3. **BF3-027-AC03** — Same command and disclosure restrictions as CLI/web.

**Scenario links:** Package-specific acceptance above; the integrated release gate adds cross-cutting cases.


#### BF3-028 — Evaluate the OpenJarvis engine/core boundary

**Gate:** G3. **Prerequisites:** BF3-013, BF3-020.

**Outcome:** Measure whether a minimal inference subset is cheaper than the existing adapter path.

**Proposed source scope:** `spikes/openjarvis/`, `docs/reuse/`.

1. **BF3-028-AC01** — Compile/dependency/cancel/stream evidence is collected in the actual environment.
2. **BF3-028-AC02** — No broad Python/desktop/training/scheduler dependency is adopted implicitly.
3. **BF3-028-AC03** — Local quality, continued work after timeout and total cost are measured before routing promotion.

**Scenario links:** HF15


#### BF3-029 — Test architectural replacement or federation need

**Gate:** G3. **Prerequisites:** BF3-021, BF3-022, BF3-023.

**Outcome:** Run a bounded investigation, not a mandatory multi-hub rewrite.

**Proposed source scope:** `docs/architecture_trials/`, `tests/import_export/`.

1. **BF3-029-AC01** — Choose one explicit hypothesis and experiment budget.
2. **BF3-029-AC02** — Disjoint ownership does not imply atomic cross-repo behavior or global quota enforcement.
3. **BF3-029-AC03** — A candidate architecture retains authority/evidence and exports; absent value ends the experiment.

**Scenario links:** Package-specific acceptance above; the integrated release gate adds cross-cutting cases.


#### BF3-030 — Implement one justified forge-native improvement

**Gate:** G3. **Prerequisites:** BF3-019, BF3-022.

**Outcome:** Close one measured metadata, ownership or exact-subject enforcement gap.

**Proposed source scope:** `src/forge/extensions/`, `tests/forge_extensions/`.

1. **BF3-030-AC01** — One targeted change with maintainer and conformance receipts.
2. **BF3-030-AC02** — Ordinary Git/PR compatibility and canonical merge authority are preserved.
3. **BF3-030-AC03** — No new Git engine or full-platform rewrite follows solely from source access.

**Scenario links:** Package-specific acceptance above; the integrated release gate adds cross-cutting cases.


## 25. Runtime acceptance inventory

The inventory retains **52 earlier AT obligations and 20 harness-specific HF obligations**, with explicit v3 wording/mapping corrections, and adds **eight CF closure scenarios**. That is **80 required future runtime scenarios**, not 80 tests passed by this document. Repetition across cases is intentional when one observable boundary has several distinct failure modes.

AT-022 now distinguishes logical author exit from physical occupancy. HF02 references the direct v3 profile identity rather than the retired binding sidecar. The source versions of all 72 retained obligations are archived. BF3 mappings replace earlier work-package identifiers; preserved IDs are traceability, not a second execution graph.

Cases apply before their associated capability is exposed. A fake-only implementation can exist before live-provider certification, but later tests cannot retroactively excuse unsafe first use. Each test receipt records exact versions, fixture, input, observed failure/pass, producer and artifacts. Repeated success on a finite fixture does not establish impossibility of all future failures.


### 25.1 Core and human workflow

| ID | Injection / condition | Required outcome | Package |
|---|---|---|---|
| AT-001 | Unknown dependency | Missing ID or deleted task file does not establish completion. | BF3-002 |
| AT-002 | Graph cycle | Cyclic requirements reject plan activation. | BF3-002 |
| AT-003 | Same command replay | Same key/body returns same operation. | BF3-003 |
| AT-004 | Changed command body | Same key with changed payload rejects without mutation. | BF3-003 |
| AT-005 | Unauthorized actor | Observer cannot spend; runner cannot impersonate owner or verifier. | BF3-004 |
| AT-006 | Cross-project read | Unauthorized event/artifact reads fail even for known hashes. | BF3-004 |
| AT-007 | Budget race | Concurrent admissions preserve bounded allocations. | BF3-005 |
| AT-008 | Unknown usage | Unreported cost stays reserved/uncertain and is not zero. | BF3-005 |
| AT-009 | Expired grant | Old policy does not override revocation or expiry. | BF3-005 |
| AT-010 | Activation crash | Published inactive Git revision cannot dispatch half a graph. | BF3-006 |
| AT-011 | Task deletion | Deleting a source task cannot set MERGED or unblock a dependent. | BF3-006 |
| AT-012 | Malformed provider output | Truncated frame/final result cannot become accepted work. | BF3-007 |
| AT-013 | Private workspace | Two attempts and human clone cannot modify each other. | BF3-008 |
| AT-014 | Secret boundary | Generated code cannot read forge/reporting/release or host credentials. | BF3-008 |
| AT-015 | Malicious import | Git hooks/helpers/filters and traversal cannot execute in privileged publisher. | BF3-012 |
| AT-016 | Claim race | Exactly one current shipping writer admitted. | BF3-009 |
| AT-017 | Partial resource claim | Conflict acquires none of the requested incompatible set. | BF3-009 |
| AT-018 | Duplicate spawn | Replay launches one accepted incarnation or quarantines ambiguity. | BF3-009 |
| AT-019 | Stale generation | Old worker result/heartbeat cannot advance current state. | BF3-009 |
| AT-020 | Suspended runner | Late heartbeat response does not incorrectly extend authority. | BF3-009 |
| AT-021 | Author completed | Verifier/delivery continue after author lease ends. | BF3-011 |
| AT-022 | Pending PR ownership | Ending author authority retains the pending-change reservation; physical slot is free only after confirmed cleanup. | BF3-009 |
| AT-023 | Changed selection | Old green candidate cannot publish after replacement. | BF3-012 |
| AT-024 | Current grant at delivery | Long CI does not extend expired publication authority. | BF3-012 |
| AT-025 | Routine plan handoff | Implementation has fresh session; no native plan menu. | BF3-017 |
| AT-026 | Forbidden operation | No-prompt profile does not allow wider scope/egress. | BF3-010 |
| AT-027 | Provider auth/quota | Park or authorized backoff/fallback; no identity evasion. | BF3-016 |
| AT-028 | Silent healthy tool | Quiet approved build is not killed solely for silence. | BF3-020 |
| AT-029 | Actual hard limit | Budget/lease/cancel deadline is enforced despite advisory watcher. | BF3-020 |
| AT-030 | False author PASS | Writer summary cannot create trusted verification. | BF3-011 |
| AT-031 | Wrong evidence issuer | Same-name check from wrong identity is rejected. | BF3-011 |
| AT-032 | Gate negative case | Trusted oracle retained; known-good passes and injected behavior fails. | BF3-001 |
| AT-033 | No-op candidate | Unchanged bug does not pass solely on old broad green suite. | BF3-011 |
| AT-034 | Disabled test discovery | Missing/skipped required tests do not equal pass. | BF3-011 |
| AT-035 | Lost PR response | Reconcile one remote PR before retry; not-found may remain unknown. | BF3-012 |
| AT-036 | Cancel while effect in flight | New dispatch blocked; old outcome reconciled, not declared undone. | BF3-012 |
| AT-037 | Human changes PR | Do not overwrite, force-push or reopen a human-closed PR. | BF3-012 |
| AT-038 | Takeover checkpoint | New human clone uses known exact checkpoint; missing unsaved work is disclosed. | BF3-015 |
| AT-039 | Respecification allowance | Authorized new revision may progress; total money/invocations preserved. | BF3-015 |
| AT-040 | Owner partition | Shared lane needs explicit grant before owned cross-domain work. | BF3-014 |
| AT-041 | Integration drift | Moving target requires current combined-subject verification. | BF3-019 |
| AT-042 | Assembled behavior | All task checks green cannot override failed mission scenario. | BF3-019 |
| AT-043 | Window boundary | Report cutoff preserves work; hard deadline uses normal checkpoint/stop. | BF3-020 |
| AT-044 | Partial artifact | Incomplete bytes never satisfy complete evidence. | BF3-011 |
| AT-045 | Restore rollback | Old publisher quiesced/revoked and external authority rotated before new work. | BF3-021 |
| AT-046 | Export/import | Definitions and evidence survive; live grants/leases do not reactivate. | BF3-021 |
| AT-047 | Human baseline bias | Unknown handling time and task-selection confounding explicitly labeled. | BF3-020 |
| AT-048 | Shadow isolation | Shadow candidate cannot acquire shipping selection or effect. | BF3-023 |
| AT-049 | Shadow survival | Unshipped shadow has no production-survival observation. | BF3-023 |
| AT-050 | Release substitution | Different artifact/environment invalidates old approval. | BF3-025 |
| AT-051 | Channel replay | Authenticated sender plus exact action/revision; duplicate messages do not duplicate commands. | BF3-026 |
| AT-052 | Blind reconstruction | Fresh agent uses kit alone; missing architectural decisions recorded. | BF3-022 |


### 25.2 Harness, profiles and scope

| ID | Injection / condition | Required outcome | Package |
|---|---|---|---|
| HF01 | Unknown or unadmitted profile | Reject launch; no default provider silently substituted. | BF3-002 |
| HF02 | Effective model differs from v3 job profile | Block or admit an explicitly authorized new profile. Do not relabel a live job; old binding import preserves provenance. | BF3-010 |
| HF03 | RPC session switch returns success with cancelled=true | Fresh-context requirement remains unsatisfied; no implementation starts on inherited context. | BF3-010 |
| HF04 | RPC prompt accepted then execution fails | Acknowledge admission only; later failure is not successful task completion. | BF3-010 |
| HF05 | Spawn returns handle without child result | No result or acceptance inferred from handle admission. | BF3-010 |
| HF06 | Native child switches to ineligible provider | Request denied or lane fails certification; parent data policy is not widened. | BF3-008 |
| HF07 | Child or daemon remains after client exits | Root sandbox remains supervised; deadline stops/fences all managed descendants. | BF3-009 |
| HF08 | Native session auto-resumes after lease loss | No current result or effect can be accepted; no access to another task namespace. | BF3-009 |
| HF09 | Children share writable candidate checkout | Shipping delegation disabled unless one-writer boundary is actually enforced. | BF3-008 |
| HF10 | Native child/refinement usage arrives late | Attribute to original work and profile; uncertain exposure is not released as zero. | BF3-020 |
| HF11 | Attempt-local lesson changes verifier or grant | Reject authority change; preserve it as an untrusted proposal for review. | BF3-011 |
| HF12 | Private memory proposed for team output | Current source and destination permissions enforced; no automatic scope promotion. | BF3-017 |
| HF13 | A profile change wins by changing acceptance tests | Evaluation invalid; trusted gate/holdout remains fixed. | BF3-023 |
| HF14 | Online refinement treated as frozen evaluation | Reject cohort equivalence or label adaptation treatment and initial/delta state. | BF3-023 |
| HF15 | Local engine timeout with continued server work | Record unknown/continuing resource use and block unsafe budget release. | BF3-028 |
| HF16 | Mutable trace outcome overwrites old result | It cannot modify authoritative immutable evidence or effect records. | BF3-003 |
| HF17 | Agent uses channel endpoint to self-approve | Human-only decision capability is rejected for agent principal. | BF3-004 |
| HF18 | Roll back a bad profile while attempts run | Only future admission selection changes; running subjects and accounting stay pinned. | BF3-023 |
| HF19 | Slow UI plus profile-event flood | Bound buffering; status, cancellation, durable results and leases retain priority. | BF3-018 |
| HF20 | Import profile export with old grants | Restore definitions/history only; new paused authority and fresh admission required. | BF3-021 |


### 25.3 Explicit closure scenarios

| ID | Injection / condition | Required outcome | Package |
|---|---|---|---|
| CF01 | Sealed writer still physically alive | Independent checking proceeds on sealed bytes; old runner occupancy stays allocated/unconfirmed until termination is proved. | BF3-008 |
| CF02 | CI-only check requires a draft PR | Prepublication gates permit exact draft publication; task stays checking until trusted CI result, without circular waits or premature review-ready. | BF3-011 |
| CF03 | Verifier restart and stale result race | Reassigned verification has a new job generation; an old result cannot overwrite current evidence selection, and re-execution is accounted. | BF3-009 |
| CF04 | Prerequisite merged then known-reverted | Historical receipt remains; dependent is held or fails precondition even though the old merge remains in ancestry. | BF3-019 |
| CF05 | Writer tries to exhaust verification money | Mandatory completion reserve cannot fund extra writing; check/review jobs can use its transferred hold without double billing. | BF3-005 |
| CF06 | Agent initiated by an administrator approves itself | Initiating owner is not authenticated human action; agent capability cannot change grants or satisfy required human approval. | BF3-004 |
| CF07 | Old digest or binding silently relabeled v3 | Import retains source bytes and digest scheme, validates mapping, creates new definitions/proposed history and no active authority. | BF3-021 |
| CF08 | Root-inclusive and child usage reported together | Reporting basis chooses inclusive root or disjoint leaves; same consumption is neither charged twice nor omitted. | BF3-020 |


### 25.4 Pilot and go/no-go

Run an initial functional pilot of approximately thirty representative tasks across two owners and two coding-provider families, including failures, high-risk cases that should stop, long checking, dependency invalidation and takeover. Freeze task requirements and acceptance before comparison. Compare the current manual workflow, a well-configured strong single-provider workflow, and BulletFarm; an all-strong BulletFarm control helps separate orchestration from economical routing.

Keep failures and exclusions visible. Record delivered parent requirements, agreed observation horizon, defects/rework, total cost/confidence, active human handling, latency and queue delays. Domain-selected production rows are not a randomized experiment. A small pilot demonstrates usefulness; sample/precision requirements for rare failures need a separate declared design.

The product gate is zero routine vendor-terminal intervention on certified in-policy paths; applicable authority/evidence/fault checks passed; truthful failed/unknown state; useful two-person coordination and takeover; and a justified quality/cost/attention result for promoted task classes. Cost-reduction percentages are measured outcomes, not promises in this document.

Non-compensable failures include unauthorized spend or cross-domain action, writer-forged checks, wrong-subject integration, privileged untrusted execution, newly dispatched stale-authority writes, unsafe duplicate effects after unresolved timeouts, and silent loss of acknowledged state. Any one blocks the affected capability regardless of its weighted score.


## 26. Final score and remaining deductions

The final contract scores **95.04/100**, rounded to **95.0**, under the same twelve-category rubric used for every full source plan. It is **0.56 points above J's 94.48** in this review. That small difference expresses a preference for clearer closure—not statistical superiority, measured reliability or a forecast of success.

It is stronger because it combines H's stage-specific checks and physical-capacity distinction with J's execution profiles and guarded learning, while producing one prospective v3 contract rather than another layer of amendments. Current prerequisite validity, completion funding, all-job recovery and actor-versus-initiator separation are made explicit. The implementation remains one small controller and existing execution/CI systems.


| Dimension | Weight | Score | Points | Why / remaining deduction |
|---|---:|---:|---:|---|
| MVP simplicity | 14 | 94 | 13.16 | One package, backend and allocator; early vertical slice and narrow optional adoption. The first trustworthy runner, publisher and identity setup still require real engineering. |
| Autonomy | 10 | 96 | 9.60 | Fresh authorized execution, typed stops, no terminal macros, model-independent control. Authentication and genuinely new product/permission decisions cannot disappear. |
| Plans and contracts | 10 | 95 | 9.50 | Coherent task contracts, explicit prerequisites, rolling plans and bounded critique. Meaningful decomposition and complete assumptions remain reasoning problems. |
| Verification | 12 | 96 | 11.52 | Gate adequacy, stage-specific checks, trusted exact subjects and assembled outcomes. No finite suite or reviewer guarantees semantic correctness. |
| Recovery | 10 | 95 | 9.50 | All-job recovery, sealed-author handoff, physical occupancy, unknown effects and restoration. Remote APIs remain non-atomic; the single hub is not highly available. |
| Team ownership | 10 | 95 | 9.50 | Owner/domain consent, retained reservations, human takeover and fair progress. Unregistered work and undeclared semantic conflicts remain partially visible. |
| Economics and learning | 8 | 95 | 7.60 | Completion reserve, full usage basis, bounded retries and controlled profile evaluation. Savings, quota fidelity and sufficient experimental volume are unmeasured. |
| Provider realism | 6 | 94 | 5.64 | Concrete certified transports and selected pinned upstream interfaces. Actual installed versions, credentials and native child behavior still need tests. |
| Security | 8 | 95 | 7.60 | Explicit actor classes, current grants, protected importer/reporter and scoped data. The actual operating-system and provider credential boundaries are not audited here. |
| Operator experience | 4 | 96 | 3.84 | One workbench, actionable decisions, deterministic reports and human return path. Real usability and attention reduction must be observed. |
| Production and forge | 2 | 94 | 1.88 | Current protected integration and same-artifact release, no new refinery. Actual private-forge semantics and service-specific recovery remain unproven. |
| Build precision | 6 | 95 | 5.70 | One contract, schemas/examples, reference SQL, ordered backlog and migration/rebuild tests. Independent reconstruction and complete runtime implementation have not occurred. |
| **Total** | **100** | | **95.04** | **Specified-design fit only.** |


**Not scored:** current implementation completeness, runtime uptime, measured savings, actual sandbox security, installed-provider conformance, forge parity, production readiness or the independent rebuild. Static schema/SQL/document checks cannot establish these. The validation report identifies exactly what was run.

The remaining evidence should come from an actual first task, a stopped author with continuing legitimate delivery, current-grant rejection after long CI, two-owner overlap/disjoint work, scoped human takeover, actual provider coding handoff, known-revert handling, completion-reserve behavior, physical cleanup and protected integration. More agents, more pages or another source reference do not substitute for those demonstrations.

## 27. Final build directive

**Build a personal engineering workbench with shared accountability.** People own purpose and decisions. Replaceable workers perform bounded work. The controller owns current permissions, assignment and recovery. Independent evidence governs acceptance. The forge and release systems remain authorities for what actually happened outside the controller.

Preserve useful existing source, tasks and history. Do not build a new Git implementation, a fleet of autonomous bosses or a generic workflow platform. Certify one useful executor first. Add team operation before scaling. Learn by controlled comparison, and make useful lessons pass a promotion gate instead of rewriting their own rules.

The decisive demonstration is one real change: a person delegates it; a fresh worker proposes exact code; its writing authority ends; independent checking continues; physical occupancy remains honest; one PR survives a lost response; current grants still apply; the pending change stays reserved; a human can take over; and another owner's disjoint task continues without a provider-terminal approval click.


# Appendix A — Wire contracts and worked examples

The seven schema files are strict v3 shape contracts. Their examples use synthetic IDs and hashes; none supplies an actual repository object, grant or budget. The following dictionaries and examples make the public contract readable without earlier chats. Semantic validation and authority checks remain mandatory.

| Contract | Mandatory groups | Additional rules |
|---|---|---|
| Task | Identity/revision/lineage; mission/repository/owner/domain; intent/class; dependencies; write/resources; acceptance; gate/risk/route/grant; limits/context/stop | Code tasks need writable scope and code delivery goal; investigations produce evidence; check references resolve |
| Profile | Harness source/binary/adapter/transport; provider/model/settings; context/prompt/skills/memory; engine/environment/hardware; delegation/refinement | Immutable definition; no grant; secrets/endpoints resolved only by protected config |
| Job | Purpose/lane/executor; mission and optional task; exact inputs/candidate/selection; profile; grant/hold; runner/generations/epoch; limits/deadline/result shape | Purpose-specific nullability and role constraints; planning precedes tasks; check jobs are not model writers |
| Receipt | Stage; mission/task/job; subject/contract/candidate/selection; check/recipe/environment; producer/results/suites/artifacts/time/limits | Trusted reporter resolves all candidate/remote identities; a JSON claim is not authenticated proof |
| Command | ID/kind; target and expected version; strict kind-specific payload | Creation uses null target/version; actor from authentication; human-only capabilities remain distinct |
| Job result | Job/assignment/sequence; proposed outcome/candidate/artifacts; typed blocker; usage reference | Purpose and authenticated producer verified independently; no worker-supplied acceptance |
| Build checkpoint | Spec/source identities; completed/next packages; commands/exit/artifacts; gaps; certified capabilities | Only actual evidence marks complete; no runtime certification inferred from a package build |

Every digest is SHA-256 over the exact registered UTF-8 artifact bytes unless the retained source explicitly says otherwise. Reject duplicate keys and unsupported schema versions. Numeric money is integer micro-units; settings needing exact decimals encode strings and are adapter-allowlisted. Object IDs retain their repository format; a synthetic example SHA is not fetched or treated as authentic.

The following is the complete example task, execution profile, implementation job, verification receipt and human-take command. Read `planning_job.json` and `verification_job.json` for the exact role variants; both use the same Job schema. `build_checkpoint.json` illustrates an empty honest checkpoint.


### Task example

```json
{
  "schema_version": 3,
  "id": "T-001",
  "revision": 1,
  "mission_id": "M-001",
  "lineage_id": "L-001",
  "repo_id": "repo-demo",
  "owner_id": "owner-demo",
  "domain_id": "core",
  "kind": "implementation",
  "class": "bounded_bug",
  "title": "Reject repeated delivery IDs",
  "objective": "First delivery is accepted; a repeat is rejected without a second effect.",
  "non_goals": [
    "No deployment or credential changes."
  ],
  "requirement_ids": [
    "REQ-01"
  ],
  "depends_on": [],
  "write_paths": [
    "src/dedup.rs"
  ],
  "resource_keys": [
    "dedup-contract"
  ],
  "acceptance": [
    {
      "id": "AC-01",
      "statement": "First then repeated delivery yields true then false.",
      "check_ids": [
        "dedup-repeat"
      ],
      "manual_owner_id": null
    }
  ],
  "gate_profile_id": "demo-protected",
  "risk": "standard",
  "route_profile_id": "demo-route",
  "grant_id": "demo-grant",
  "delivery_goal": "pr_ready",
  "limits": {
    "write_invocations": 3,
    "runtime_seconds": 600
  },
  "context_refs": [
    "demo-baseline"
  ],
  "stop_conditions": [
    "Protected check definition is missing."
  ]
}
```


### Profile example

```json
{
  "schema_version": 3,
  "id": "profile-fake",
  "revision": 1,
  "harness": {
    "name": "fake",
    "source_revision": "fixture-v1",
    "binary_digest": "1111111111111111111111111111111111111111111111111111111111111111",
    "adapter_revision": "fake-v1",
    "transport": "jsonl"
  },
  "provider_id": "fake",
  "model_id": "deterministic-patch",
  "settings": {},
  "context_strategy": "compact-v1",
  "prompt_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "skills_digest": "3333333333333333333333333333333333333333333333333333333333333333",
  "initial_memory_digest": "4444444444444444444444444444444444444444444444444444444444444444",
  "engine_ref": "engine-fake",
  "environment_digest": "5555555555555555555555555555555555555555555555555555555555555555",
  "hardware_class": "fixture",
  "delegation": {
    "mode": "disabled",
    "max_depth": 0,
    "max_children": 0,
    "usage_basis": "inclusive_root"
  },
  "refinement": "frozen"
}
```


### Job example

```json
{
  "schema_version": 3,
  "id": "J-001",
  "purpose": "implement",
  "lane": "shipping",
  "executor_kind": "model",
  "mission_id": "M-001",
  "task_id": "T-001",
  "task_revision": 1,
  "contract_digest": "d89846c7df91a485f975ffea92efb4ed21800fa706374d4a2eeb8e9b0108abba",
  "input_digest": "6666666666666666666666666666666666666666666666666666666666666666",
  "source_base_oid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "candidate_id": null,
  "selection_version": null,
  "profile_digest": "9a9ed8b442025e0ebec55f83d91a176cc1d6cffc98309e3b49614a3479e716a5",
  "evaluation_id": null,
  "grant_id": "demo-grant",
  "budget_hold_id": "BH-001",
  "runner_id": "runner-fixture",
  "assignment_generation": 1,
  "writer_generation": 1,
  "authority_epoch": "epoch-fixture",
  "limits": {
    "runtime_seconds": 600,
    "max_output_bytes": 1048576,
    "max_tool_calls": 100
  },
  "deadline_at": "2026-09-16T23:00:00Z",
  "result_schema_id": "result-v3"
}
```


### Receipt example

```json
{
  "schema_version": 3,
  "id": "EV-001",
  "stage": "candidate",
  "mission_id": "M-001",
  "task_id": "T-001",
  "job_id": "J-verify",
  "subject_digest": "7777777777777777777777777777777777777777777777777777777777777777",
  "contract_digest": "d89846c7df91a485f975ffea92efb4ed21800fa706374d4a2eeb8e9b0108abba",
  "candidate_id": "C-001",
  "selection_version": 1,
  "check_id": "dedup-repeat",
  "check_recipe_digest": "8888888888888888888888888888888888888888888888888888888888888888",
  "environment_digest": "5555555555555555555555555555555555555555555555555555555555555555",
  "producer_id": "verifier-fixture",
  "result": "pass",
  "suites": [
    {
      "name": "protected_dedup",
      "executed": 2,
      "failed": 0,
      "skipped": 0
    }
  ],
  "artifact_ids": [
    "ART-test-output"
  ],
  "observed_at": "2026-09-16T20:00:00Z",
  "limitations": [
    "Synthetic example, not an executed test receipt."
  ]
}
```


### Job result example

```json
{
  "schema_version": 3,
  "job_id": "J-001",
  "assignment_generation": 1,
  "producer_sequence": 8,
  "outcome": "candidate_proposed",
  "candidate": {
    "commit_oid": "cccccccccccccccccccccccccccccccccccccccc",
    "tree_oid": "dddddddddddddddddddddddddddddddddddddddd",
    "base_oid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "artifact_id": "ART-candidate"
  },
  "artifact_ids": [
    "ART-candidate"
  ],
  "blocker": null,
  "usage_artifact_id": null
}
```

### Command example

```json
{
  "schema_version": 3,
  "command_id": "CMD-001",
  "kind": "take",
  "target_id": "T-001",
  "expected_version": 3,
  "payload": {
    "checkpoint_preference": "last_durable"
  }
}
```


# Appendix B — Structural SQL reference

`reference/001_core.sql` initializes the structural reference in SQLite. This is a concrete schema baseline, not an implemented scheduler. It deliberately names which constraints stay in controller transactions. Permissions, semantic overlap, all budget caps, current dependencies, job-role authentication and exact evidence completeness cannot be inferred by foreign keys alone.

The table inventory covers principals/membership, ownership, missions/contracts/tasks, jobs and their separate occupancy, current grants, profiles, pending reservations, candidates/selections, artifacts/evidence, completion/budget holds, usage, commands/events/effects, decisions, observations and known prerequisite invalidations. These are tables in one database, not independently deployed services.

Run the supplied validator for actual in-memory structural tests. Configure production SQLite WAL and synchronization only after checking the linked runtime and filesystem. The structural smoke test does not exercise WAL, power loss, a real process race, or external publication.


```sql
-- BulletFarm v3 structural reference. Not the implemented controller.
-- Application transactions MUST additionally enforce current grants, scope overlap,
-- all budget caps, dependency validity, stage obligations and trusted producer roles.
-- In-memory validation of this file is not WAL or filesystem durability certification.
PRAGMA foreign_keys = ON;
CREATE TABLE schema_meta(version INTEGER PRIMARY KEY CHECK(version=3));
INSERT INTO schema_meta VALUES(3);
CREATE TABLE principals(
 id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('human','agent','runner','verifier','publisher')),
 active INTEGER NOT NULL CHECK(active IN (0,1))
);
CREATE TABLE repositories(id TEXT PRIMARY KEY, canonical_forge TEXT NOT NULL, policy_json TEXT NOT NULL);
CREATE TABLE memberships(
 principal_id TEXT NOT NULL REFERENCES principals(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
 role TEXT NOT NULL CHECK(role IN ('administrator','engineer','observer')),
 PRIMARY KEY(principal_id,repo_id)
);
CREATE TABLE domains(id TEXT PRIMARY KEY, owner_id TEXT NOT NULL REFERENCES principals(id), policy_json TEXT NOT NULL);
CREATE TABLE missions(
 id TEXT PRIMARY KEY, owner_id TEXT NOT NULL REFERENCES principals(id), domain_id TEXT NOT NULL REFERENCES domains(id),
 goal TEXT NOT NULL CHECK(goal IN ('pr_ready','merged','production_observed','evidence_accepted')),
 version INTEGER NOT NULL CHECK(version>0), paused INTEGER NOT NULL DEFAULT 0 CHECK(paused IN(0,1)),
 contract_json TEXT NOT NULL
);
CREATE TABLE contracts(
 task_id TEXT NOT NULL, revision INTEGER NOT NULL CHECK(revision>0), mission_id TEXT NOT NULL REFERENCES missions(id),
 repo_id TEXT NOT NULL REFERENCES repositories(id), digest TEXT NOT NULL CHECK(length(digest)=64),
 digest_scheme TEXT NOT NULL CHECK(digest_scheme='sha256_exact_utf8_v3'), git_oid TEXT NOT NULL,
 raw_json BLOB NOT NULL, PRIMARY KEY(task_id,revision), UNIQUE(task_id,revision,digest)
);
CREATE TABLE tasks(
 id TEXT PRIMARY KEY, active_revision INTEGER NOT NULL, version INTEGER NOT NULL CHECK(version>0),
 phase TEXT NOT NULL CHECK(phase IN ('proposed','ready','working','checking','review_ready','integrating','merged','evidence_accepted','failed','cancelled','superseded')),
 hold_reason TEXT, writer_generation INTEGER NOT NULL DEFAULT 0 CHECK(writer_generation>=0),
 lineage_id TEXT NOT NULL, lifetime_invocations INTEGER NOT NULL DEFAULT 0 CHECK(lifetime_invocations>=0),
 FOREIGN KEY(id,active_revision) REFERENCES contracts(task_id,revision)
);
CREATE TABLE dependencies(
 task_id TEXT NOT NULL, revision INTEGER NOT NULL, predecessor_id TEXT NOT NULL, predecessor_revision INTEGER NOT NULL,
 condition TEXT NOT NULL CHECK(condition IN ('merged','evidence_accepted')), preconditions_json TEXT NOT NULL,
 PRIMARY KEY(task_id,revision,predecessor_id,predecessor_revision),
 FOREIGN KEY(task_id,revision) REFERENCES contracts(task_id,revision),
 FOREIGN KEY(predecessor_id,predecessor_revision) REFERENCES contracts(task_id,revision),
 CHECK(task_id<>predecessor_id)
);
CREATE TABLE profiles(digest TEXT PRIMARY KEY CHECK(length(digest)=64), raw_json BLOB NOT NULL);
CREATE TABLE grants(
 id TEXT PRIMARY KEY, issuer_id TEXT NOT NULL REFERENCES principals(id), subject_id TEXT NOT NULL REFERENCES principals(id),
 expires_at TEXT NOT NULL, revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN(0,1)),
 invocation_allowance INTEGER NOT NULL CHECK(invocation_allowance>=0), used_invocations INTEGER NOT NULL DEFAULT 0 CHECK(used_invocations>=0),
 capability_json TEXT NOT NULL
);
CREATE TABLE budget_accounts(
 id TEXT PRIMARY KEY, currency TEXT NOT NULL CHECK(length(currency)=3), limit_micros INTEGER NOT NULL CHECK(limit_micros>=0),
 actual_micros INTEGER NOT NULL DEFAULT 0 CHECK(actual_micros>=0)
 -- Actual spending may exceed a limit: record it honestly; controller blocks new admission.
);
CREATE TABLE runners(
 id TEXT PRIMARY KEY, principal_id TEXT NOT NULL REFERENCES principals(id), incarnation_id TEXT NOT NULL,
 enabled INTEGER NOT NULL CHECK(enabled IN(0,1)), capacity INTEGER NOT NULL CHECK(capacity>=0)
);
CREATE TABLE jobs(
 id TEXT PRIMARY KEY, mission_id TEXT NOT NULL REFERENCES missions(id), task_id TEXT, task_revision INTEGER,
 purpose TEXT NOT NULL CHECK(purpose IN ('plan','implement','investigate','review','verify','shadow')),
 lane TEXT NOT NULL CHECK(lane IN ('shipping','shadow','wildcard')),
 executor_kind TEXT NOT NULL CHECK(executor_kind IN ('model','check')),
 profile_digest TEXT REFERENCES profiles(digest), runner_id TEXT NOT NULL REFERENCES runners(id),
 assignment_generation INTEGER NOT NULL CHECK(assignment_generation>0), writer_generation INTEGER,
 authority_epoch TEXT NOT NULL, grant_id TEXT NOT NULL REFERENCES grants(id), deadline_at TEXT NOT NULL,
 lifecycle TEXT NOT NULL CHECK(lifecycle IN ('prepared','starting','running','stopping','succeeded','failed','interrupted','unknown')),
 writer_authority TEXT NOT NULL CHECK(writer_authority IN ('none','active','sealed','revoked')),
 occupancy TEXT NOT NULL CHECK(occupancy IN ('allocated','running','stopped','unconfirmed')),
 input_digest TEXT NOT NULL, envelope_json TEXT NOT NULL,
 FOREIGN KEY(task_id,task_revision) REFERENCES contracts(task_id,revision),
 CHECK((task_id IS NULL AND task_revision IS NULL) OR (task_id IS NOT NULL AND task_revision IS NOT NULL)),
 CHECK((executor_kind='model' AND profile_digest IS NOT NULL) OR (executor_kind='check' AND profile_digest IS NULL)),
 CHECK(purpose<>'plan' OR (task_id IS NULL AND writer_authority='none' AND executor_kind='model')),
 CHECK(purpose<>'verify' OR (executor_kind='check' AND writer_authority='none')),
 CHECK(purpose<>'shadow' OR (lane='shadow' AND writer_authority='none')),
 CHECK(writer_authority='none' OR (purpose='implement' AND lane='shipping' AND task_id IS NOT NULL AND writer_generation>0)),
 UNIQUE(task_id,writer_generation)
);
CREATE UNIQUE INDEX one_current_writer ON jobs(task_id) WHERE writer_authority='active';
CREATE TABLE change_reservations(
 id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
 resource_kind TEXT NOT NULL CHECK(resource_kind IN ('path','interface')), resource_key TEXT NOT NULL,
 active INTEGER NOT NULL CHECK(active IN(0,1)), owner_id TEXT NOT NULL REFERENCES principals(id)
 -- Prefix overlap and all-or-none acquisition are controller predicates, not this index.
);
CREATE INDEX active_resources ON change_reservations(repo_id,active,resource_kind,resource_key);
CREATE TABLE human_contributions(
 id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id), owner_id TEXT NOT NULL REFERENCES principals(id),
 source_oid TEXT NOT NULL, input_artifact_id TEXT NOT NULL, provenance_json TEXT NOT NULL
);
CREATE TABLE candidates(
 id TEXT PRIMARY KEY, task_id TEXT NOT NULL, task_revision INTEGER NOT NULL, producer_job_id TEXT REFERENCES jobs(id),
 human_contribution_id TEXT REFERENCES human_contributions(id), commit_oid TEXT NOT NULL, tree_oid TEXT NOT NULL,
 base_oid TEXT NOT NULL, artifact_id TEXT NOT NULL, provenance_json TEXT NOT NULL,
 FOREIGN KEY(task_id,task_revision) REFERENCES contracts(task_id,revision), UNIQUE(id,task_id),
 CHECK((producer_job_id IS NOT NULL AND human_contribution_id IS NULL) OR (producer_job_id IS NULL AND human_contribution_id IS NOT NULL))
);
CREATE TABLE selections(
 task_id TEXT PRIMARY KEY REFERENCES tasks(id), version INTEGER NOT NULL CHECK(version>0), candidate_id TEXT,
 FOREIGN KEY(candidate_id,task_id) REFERENCES candidates(id,task_id)
);
CREATE TABLE artifacts(
 id TEXT PRIMARY KEY, repo_id TEXT NOT NULL REFERENCES repositories(id), digest TEXT NOT NULL CHECK(length(digest)=64),
 byte_count INTEGER NOT NULL CHECK(byte_count>=0), complete INTEGER NOT NULL CHECK(complete IN(0,1)), metadata_json TEXT NOT NULL
);
CREATE TABLE evidence(
 id TEXT PRIMARY KEY, job_id TEXT NOT NULL REFERENCES jobs(id), candidate_id TEXT REFERENCES candidates(id),
 subject_digest TEXT NOT NULL, producer_id TEXT NOT NULL REFERENCES principals(id), check_id TEXT NOT NULL,
 result TEXT NOT NULL CHECK(result IN ('pass','fail','incomplete','not_applicable')), receipt_json TEXT NOT NULL
 -- Producer role, purpose, generation, actual subject and complete artifacts are verified by the controller.
);
CREATE TABLE completion_holds(
 id TEXT PRIMARY KEY, task_id TEXT NOT NULL REFERENCES tasks(id), amount_micros INTEGER NOT NULL CHECK(amount_micros>=0),
 remaining_micros INTEGER NOT NULL CHECK(remaining_micros>=0), currency TEXT NOT NULL, state TEXT NOT NULL CHECK(state IN('held','assigned','settled','released'))
);
CREATE TABLE budget_holds(
 id TEXT PRIMARY KEY, job_id TEXT REFERENCES jobs(id), account_id TEXT NOT NULL REFERENCES budget_accounts(id),
 completion_hold_id TEXT REFERENCES completion_holds(id), amount_micros INTEGER NOT NULL CHECK(amount_micros>=0),
 state TEXT NOT NULL CHECK(state IN('held','settled','released','uncertain'))
);
CREATE TABLE usage_observations(
 id TEXT PRIMARY KEY, job_id TEXT NOT NULL REFERENCES jobs(id), meter_id TEXT NOT NULL, meter_sequence INTEGER NOT NULL CHECK(meter_sequence>=0),
 basis TEXT NOT NULL CHECK(basis IN ('delta','cumulative_inclusive','disjoint_leaf','unknown')),
 amount_micros INTEGER CHECK(amount_micros>=0), currency TEXT NOT NULL, confidence TEXT NOT NULL CHECK(confidence IN('reported','estimated','unknown')),
 UNIQUE(job_id,meter_id,meter_sequence), CHECK((confidence='unknown' AND amount_micros IS NULL) OR (confidence<>'unknown' AND amount_micros IS NOT NULL))
);
CREATE TABLE commands(
 actor_id TEXT NOT NULL REFERENCES principals(id), command_id TEXT NOT NULL, request_digest TEXT NOT NULL,
 operation_id TEXT NOT NULL UNIQUE, payload_json TEXT NOT NULL, PRIMARY KEY(actor_id,command_id)
);
CREATE TABLE events(
 seq INTEGER PRIMARY KEY AUTOINCREMENT, scope_repo_id TEXT REFERENCES repositories(id), kind TEXT NOT NULL,
 producer_id TEXT NOT NULL REFERENCES principals(id), producer_sequence INTEGER, job_id TEXT REFERENCES jobs(id),
 payload_json TEXT NOT NULL, UNIQUE(producer_id,job_id,producer_sequence)
);
CREATE TABLE effects(
 id TEXT PRIMARY KEY, logical_key TEXT NOT NULL UNIQUE, request_digest TEXT NOT NULL,
 task_id TEXT REFERENCES tasks(id), candidate_id TEXT REFERENCES candidates(id), selection_version INTEGER,
 grant_id TEXT NOT NULL REFERENCES grants(id), publisher_id TEXT NOT NULL REFERENCES principals(id),
 state TEXT NOT NULL CHECK(state IN ('pending','dispatching','confirmed','outcome_unknown','not_applied','blocked')),
 payload_json TEXT NOT NULL, receipt_json TEXT
);
CREATE TABLE decisions(
 id TEXT PRIMARY KEY, owner_id TEXT NOT NULL REFERENCES principals(id), required_actor_kind TEXT NOT NULL CHECK(required_actor_kind IN('human','agent')),
 subject_digest TEXT NOT NULL, version INTEGER NOT NULL CHECK(version>0), status TEXT NOT NULL CHECK(status IN('open','resolved','expired','superseded')),
 payload_json TEXT NOT NULL
);
CREATE TABLE observations(
 id TEXT PRIMARY KEY, repo_id TEXT NOT NULL REFERENCES repositories(id), source TEXT NOT NULL, external_id TEXT NOT NULL,
 observed_at TEXT NOT NULL, payload_json TEXT NOT NULL
);
CREATE TABLE dependency_invalidations(
 id TEXT PRIMARY KEY, predecessor_id TEXT NOT NULL REFERENCES tasks(id), observed_at TEXT NOT NULL,
 evidence_id TEXT, reason TEXT NOT NULL, active INTEGER NOT NULL CHECK(active IN(0,1))
);
CREATE TRIGGER contracts_no_update BEFORE UPDATE ON contracts BEGIN SELECT RAISE(ABORT,'immutable_contract'); END;
CREATE TRIGGER contracts_no_delete BEFORE DELETE ON contracts BEGIN SELECT RAISE(ABORT,'immutable_contract'); END;
CREATE TRIGGER profiles_no_update BEFORE UPDATE ON profiles BEGIN SELECT RAISE(ABORT,'immutable_profile'); END;
CREATE TRIGGER candidates_no_update BEFORE UPDATE ON candidates BEGIN SELECT RAISE(ABORT,'immutable_candidate'); END;
CREATE TRIGGER evidence_no_update BEFORE UPDATE ON evidence BEGIN SELECT RAISE(ABORT,'immutable_evidence'); END;
CREATE TRIGGER evidence_no_delete BEFORE DELETE ON evidence BEGIN SELECT RAISE(ABORT,'immutable_evidence'); END;
CREATE TRIGGER command_identity_immutable BEFORE UPDATE OF request_digest,payload_json,actor_id,command_id ON commands BEGIN SELECT RAISE(ABORT,'immutable_command_identity'); END;
CREATE TRIGGER effect_identity_immutable BEFORE UPDATE OF logical_key,request_digest,payload_json,task_id,candidate_id,selection_version,grant_id,publisher_id ON effects BEGIN SELECT RAISE(ABORT,'immutable_effect_identity'); END;
```


# Appendix C — Preservation and deliberate changes

Preservation means each important requirement has an explicit destination or rejection reason. It does not mean every old table, schema version, task ID and optional subsystem must be implemented. The archive preserves positions that this final contract does not adopt.

| Requirement / idea | Source | Final section | Scenario | Disposition |
|---|---|---|---|---|
| Personal/domain ownership; no task stealing | F, H, I, J | 6, 13, 14 | AT-040 | Retained; federation deferred explicitly |
| Custom CLI/web and one Foreman | A–E, G–J | 10, 20 | AT-025 | Retained; no provider terminal dependency |
| Small Git-backed contracts with acceptance | A–J | 8–11 | AT-001, AT-002 | Retained; code/tests remain coherent |
| Fresh approved-plan execution | A–E, G–J | 10, 12, 19 | AT-025, HF03 | Retained; actual session identity checked |
| Bounded cross-frontier critique | A–E, G–J | 10 | AT-025 | Retained; opinions not proof |
| No unknown-as-done file state | A–E, G–J; departure from F | 9 | AT-001, AT-011 | Typed dependency receipts replace file absence |
| Short lease, persistent change reservation | E, G–J | 14 | AT-021, AT-022 | Retained |
| Physical occupancy after logical sealing | H | 12, 14 | CF01 | Adopted explicitly |
| Verifier independent of author lifetime | G–J | 12, 15 | AT-021, CF03 | Retained and extended to every job |
| Current grants and exact selected candidate | G–J | 13–16 | AT-023, AT-024 | Retained |
| Human-only authority changes | J and QM | 13, 20 | HF17, CF06 | Distinct actor capability, not only user ID |
| Durable start/effects before action | A–E, G–J | 14, 16 | AT-018, AT-035 | Retained |
| Unknown outcomes and safe restoration | A–E, G–J | 16, 23 | AT-036, AT-045 | Retained; no exactly-once claim |
| Gate integrity and adequacy | F, H–J; integrity throughout | 11, 15 | AT-030, AT-032, AT-033 | Retained, distinct obligations |
| Stage-specific CI and PR readiness | H; location distinction I/J | 15 | CF02 | Adopted without circular publication |
| Protected exact combined integration | A–J | 15, 21 | AT-041 | Native authority; no custom queue default |
| Assembled mission acceptance | B, E, G–J | 15, 21 | AT-042 | Retained |
| Current prerequisite validity after known reverts | Earlier dependency intent; new explicit closure | 9, 15 | CF04 | Added bounded checks, no universal detector |
| Full costs and no retry-history erasure | A–E, G–J; respecify intent F | 17, 18 | AT-039, CF08 | Retained |
| Reserve mandatory checking/review cost | Budget principles throughout; new explicit closure | 18 | CF05 | Added completion allocation |
| Human takeover/respecify/ad-hoc | F, H–J | 17 | AT-038, AT-039 | Retained; separate checkout and return |
| Blocks as attention/budget windows | F operating concern; H–J resolution | 17, 18 | AT-043 | Retained, no forced block PR |
| Prime/recursive persistent execution | F, J | 19 | HF03–HF10 | Certify contained executor, not allocator |
| OpenJarvis Rust/local inference reuse | J | 19 | HF15 | Optional measured subset, not full fork |
| QM scoped memory/shared contexts | J | 10, 13, 19 | HF12, HF17 | Retained; source restrictions follow summaries |
| Execution profiles, not just model names | J | 8, 19, 22 | HF01, HF02 | Direct v3 job reference; old binding imported |
| Shipping versus shadow experiments | F, H–J | 22 | AT-048, AT-049 | No shadow publication authority |
| Guarded learning and rollback | J | 22 | HF11, HF13, HF18 | Retained; fixed gate and held-out test |
| Portable assets and architecture exit | F, H–J | 22, 23 | AT-046, HF20 | Retained; import paused without old grants |
| Production and channel stages | A–J | 21 | AT-050, AT-051 | Retained as explicit extensions |
| Fresh-agent rebuild and build checkpoint | H, I, J | 24, 25 | AT-052 | Retained; never asserted complete here |


# Appendix D — Sources and review limits

## D1. Supplied full plans

These are the primary basis for comparison. Source positions are not silently rewritten into the recommendation. The exact byte-identical duplicates are recorded in the manifest. Sections cited as A–J in the adjudication refer to these archived documents, not to a model's recollection of an earlier summary.


| ID | Exact supplied filename | SHA-256 prefix |
|---|---|---|
| A | `BULLETFARM_MVP_VISION_AND_ENGINEERING_SPEC(1)(5).md` | `83052159020242a2` |
| B | `BulletFarm_Vision_Engineering_Spec(5).md` | `3e932410e7040b8a` |
| C | `BULLETFARM_MVP_VISION_AND_ENGINEERING_SPEC(6).md` | `9b85b0d47cb8d12b` |
| D | `BULLETFARM_MVP_VISION_ENGINEERING_SPEC_2026-09-15(5).md` | `a4e23e26203b8455` |
| E | `BULLETFARM_CORE_MVP_ENGINEERING_SPEC(5).md` | `685110e6a9a5af76` |
| F | `Pasted markdown(20260916-143852).md` | `a74141c4da8f5fcf` |
| G | `BULLETFARM_FINAL_ADJUDICATION_AND_SPEC.md` | `fc21afad68ebb238` |
| H | `BULLETFARM_FINAL_SPEC.md` | `413de0ba6dcf9b76` |
| I | `BULLETFARM_FINAL_VISION_AND_ENGINEERING_SPEC.md` | `3cd7c85683174ed8` |
| J | `BULLETFARM_HARNESS_INFORMED_FINAL_SPEC.md` | `fb859ac742d3da1e` |


K is the four-page alignment/recommendation brief; L is the harness decision brief. Their Markdown representations are archived. Available machine backlogs for A, C, G and I were inspected; J adds mapped adoption amendments to I's graph. H's guide names additional sidecars, but this review only obtained its distinct specification and guide, not a separately verified H machine kit. Missing sidecars are not assumed absent from every other location.

The original A–E proposals and later syntheses are related artifacts, not independent empirical experiments. Their repeated agreement does not establish correctness. Prior full V3/FORGE documents mentioned inside the supplied plans and other tools listed in Alton's landscape were not separately re-audited or rescored as complete current architectures.

## D2. Selected external verification

The following primary documentation was revisited on September 16, 2026. It supports interface/platform facts only. It does not certify installed versions, the user's forge, contractual entitlements or runtime performance. Provider flags and support evolve; admission uses pinned actual version tests.


**[W1] [OpenAI — Codex non-interactive execution](https://developers.openai.com/codex/noninteractive/).** Structured JSONL/final schemas and credential boundaries.


**[W2] [Anthropic — Run Claude Code programmatically](https://code.claude.com/docs/en/headless).** Headless startup, explicit configuration and authentication distinctions.


**[W3] [SQLite — Write-Ahead Logging](https://sqlite.org/wal.html).** Local/same-host and single-writer constraints; durability and documented WAL fix.


**[W4] [Git — update-ref](https://git-scm.com/docs/git-update-ref).** Expected-old-value reference updates; not a complete task-claim system.


**[W5] [GitHub — Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue).** Current combined integration subjects and merge_group checks.


**[W6] [Git — worktree](https://git-scm.com/docs/git-worktree).** Shared administrative state; directories are not independent sandboxes.


**[W7] [xAI — Plan Mode](https://docs.x.ai/build/features/plan-mode).** TUI plan review is not removed merely by automatic tool approval.


**[W8] [Prime Agent — pinned RPC contract](https://github.com/PrimeIntellect-ai/prime-agent/blob/66abc2a604fc42a220292a1ca4cf33ee60cb5733/packages/coding-agent/docs/rpc.md).** Source lines 1–170 revisited: admission, abort, framing and session-switch cancellation.


**[W9] [QM — pinned security policy](https://github.com/yc-software/qm/blob/ec86d8b603a6cc07b21aa4d55359b88c425b61ef/SECURITY.md).** Source lines 100–145 revisited: deliberate human-only actions and stated boundary limitations.


**[W10] [OpenJarvis — pinned inference trait](https://github.com/open-jarvis/OpenJarvis/blob/a6b22bac1791feebf4688789d1aacb70bf0dd8fd/rust/crates/openjarvis-engine/src/traits.rs).** Source lines 1–48 revisited: inference interface is not the full coding-worker contract.


**[W11] [GitHub — Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use).** Untrusted source execution must not inherit privileged CI credentials.


Other pinned manifests, Prime architecture/RLM/refinement details, OpenJarvis frontend/trace-store information and QM stack descriptions are retained as selected-source findings of J, with its source references preserved in the archive. They are not presented as a new full-source audit. The YC video and automatic transcript mirror remain historical context in J; no audiovisual review was performed here. No source code from those upstream projects or font files is redistributed by this package.

## D3. Actual validation scope

`validate_package.py` tests package/schema/example relationships, scoring arithmetic, dependency graphs, source hashes and selected in-memory SQL constraints. It does not execute the 80 product scenarios, compile an orchestrator, run a provider, test a real sandbox, open a PR, fork a repository, or deploy anything. The generated report gives the actual passed/failed checks and environment.

**Offline validation executed for this package: 90 checks passed, zero failed.** These include seven schema definitions, nine examples, negative shapes, score arithmetic, input hashes, available machine-backlog graphs and selected SQLite constraints. Cargo and rustc are unavailable in the generation environment, so no Rust compilation is claimed.

The full PDF is rendered from the same Markdown source and visually inspected. Rendering success and document integrity are not operational readiness. The required next evidence is an implemented, independently tested first loop and the subsequent two-owner release gate.
