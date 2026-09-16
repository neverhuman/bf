-- BulletFarm v3 structural reference. Not the implemented controller.
-- Application transactions MUST additionally enforce current grants, scope overlap,
-- all budget caps, dependency validity, stage obligations and trusted producer roles.
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
 expires_at TEXT NOT NULL, revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN (0,1)),
 invocation_allowance INTEGER NOT NULL CHECK(invocation_allowance>=0), used_invocations INTEGER NOT NULL DEFAULT 0 CHECK(used_invocations>=0),
 capability_json TEXT NOT NULL
);
CREATE TABLE budget_accounts(
 id TEXT PRIMARY KEY, currency TEXT NOT NULL CHECK(length(currency)=3), limit_micros INTEGER NOT NULL CHECK(limit_micros>=0),
 actual_micros INTEGER NOT NULL DEFAULT 0 CHECK(actual_micros>=0)
);
CREATE TABLE runners(
 id TEXT PRIMARY KEY, principal_id TEXT NOT NULL REFERENCES principals(id), incarnation_id TEXT NOT NULL,
 enabled INTEGER NOT NULL CHECK(enabled IN (0,1)), capacity INTEGER NOT NULL CHECK(capacity>=0)
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
 active INTEGER NOT NULL CHECK(active IN (0,1)), owner_id TEXT NOT NULL REFERENCES principals(id)
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
 byte_count INTEGER NOT NULL CHECK(byte_count>=0), complete INTEGER NOT NULL CHECK(complete IN (0,1)), metadata_json TEXT NOT NULL
);
CREATE TABLE evidence(
 id TEXT PRIMARY KEY, job_id TEXT NOT NULL REFERENCES jobs(id), candidate_id TEXT REFERENCES candidates(id),
 subject_digest TEXT NOT NULL, producer_id TEXT NOT NULL REFERENCES principals(id), check_id TEXT NOT NULL,
 result TEXT NOT NULL CHECK(result IN ('pass','fail','incomplete','not_applicable')), receipt_json TEXT NOT NULL
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
 evidence_id TEXT, reason TEXT NOT NULL, active INTEGER NOT NULL CHECK(active IN (0,1))
);
CREATE TABLE sessions(
 token TEXT PRIMARY KEY, principal_id TEXT NOT NULL REFERENCES principals(id), created_at TEXT NOT NULL
);
CREATE TABLE operations(
 id TEXT PRIMARY KEY, actor_id TEXT NOT NULL REFERENCES principals(id), command_id TEXT NOT NULL,
 status TEXT NOT NULL, result_json TEXT NOT NULL
);
CREATE TRIGGER contracts_no_update BEFORE UPDATE ON contracts BEGIN SELECT RAISE(ABORT,'immutable_contract'); END;
CREATE TRIGGER contracts_no_delete BEFORE DELETE ON contracts BEGIN SELECT RAISE(ABORT,'immutable_contract'); END;
CREATE TRIGGER profiles_no_update BEFORE UPDATE ON profiles BEGIN SELECT RAISE(ABORT,'immutable_profile'); END;
CREATE TRIGGER candidates_no_update BEFORE UPDATE ON candidates BEGIN SELECT RAISE(ABORT,'immutable_candidate'); END;
CREATE TRIGGER evidence_no_update BEFORE UPDATE ON evidence BEGIN SELECT RAISE(ABORT,'immutable_evidence'); END;
CREATE TRIGGER evidence_no_delete BEFORE DELETE ON evidence BEGIN SELECT RAISE(ABORT,'immutable_evidence'); END;
CREATE TRIGGER command_identity_immutable BEFORE UPDATE OF request_digest,payload_json,actor_id,command_id ON commands BEGIN SELECT RAISE(ABORT,'immutable_command_identity'); END;
CREATE TRIGGER effect_identity_immutable BEFORE UPDATE OF logical_key,request_digest,payload_json,task_id,candidate_id,selection_version,grant_id,publisher_id ON effects BEGIN SELECT RAISE(ABORT,'immutable_effect_identity'); END;
