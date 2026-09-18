-- Append-only upgrade; v3 contracts and old evidence remain untouched.
CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY);
INSERT INTO schema_migrations VALUES(1),(2);
ALTER TABLE sessions ADD COLUMN revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN(0,1));
ALTER TABLE sessions ADD COLUMN expires_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- Old prototype credentials were exposed through anonymous owner access.
UPDATE sessions SET revoked=1;
ALTER TABLE grants ADD COLUMN fixture_only INTEGER NOT NULL DEFAULT 1 CHECK(fixture_only IN(0,1));
ALTER TABLE missions ADD COLUMN control_state TEXT NOT NULL DEFAULT 'active' CHECK(control_state IN('active','stopped','cancelled'));
-- Existing grants remain fixture-only; give the old fixture the new fixture planner capability.
UPDATE grants SET capability_json=json_set(capability_json,'$.plan',json('true')) WHERE id='demo-grant' AND fixture_only=1;
CREATE TABLE projects(
 id TEXT PRIMARY KEY, owner_id TEXT NOT NULL REFERENCES principals(id),
 name TEXT NOT NULL, path TEXT NOT NULL, base_oid TEXT NOT NULL,
 mode TEXT NOT NULL CHECK(mode IN('fixture','live')), last_opened TEXT NOT NULL,
 UNIQUE(owner_id,path)
);
INSERT INTO projects SELECT 'demo','owner-demo','Delivery ID demo','fixture://dedup','fixture-v1','fixture','2026-01-01T00:00:00Z' WHERE EXISTS(SELECT 1 FROM principals WHERE id='owner-demo');
CREATE TABLE drafts(
 id TEXT PRIMARY KEY, mission_id TEXT NOT NULL UNIQUE REFERENCES missions(id),
 owner_id TEXT NOT NULL REFERENCES principals(id), project_id TEXT NOT NULL REFERENCES projects(id),
 revision INTEGER NOT NULL CHECK(revision>0), goal TEXT NOT NULL,
 plan_json TEXT, accepted_revision INTEGER, status TEXT NOT NULL,
 CHECK(accepted_revision IS NULL OR accepted_revision=revision)
);
CREATE TABLE conversation(
 seq INTEGER PRIMARY KEY AUTOINCREMENT, draft_id TEXT NOT NULL REFERENCES drafts(id),
 role TEXT NOT NULL CHECK(role IN('owner','planner','controller')), body TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE fixture_faults(point TEXT PRIMARY KEY,remaining INTEGER NOT NULL);
CREATE TABLE launch_intents(
 job_id TEXT PRIMARY KEY REFERENCES jobs(id), generation INTEGER NOT NULL,
 incarnation TEXT NOT NULL UNIQUE, state TEXT NOT NULL CHECK(state IN('pending','dispatching','acknowledged','unknown','stopped')),
 workspace TEXT NOT NULL, checkpoint TEXT NOT NULL DEFAULT 'admitted'
);
CREATE TABLE task_projection(
 seq INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL UNIQUE REFERENCES tasks(id),
 owner_id TEXT NOT NULL REFERENCES principals(id), repo_id TEXT NOT NULL REFERENCES repositories(id),
 mission_id TEXT NOT NULL REFERENCES missions(id), title TEXT NOT NULL,
 phase TEXT NOT NULL, why TEXT NOT NULL, version INTEGER NOT NULL, pr_json TEXT
);
CREATE INDEX work_owner_cursor ON task_projection(owner_id,seq DESC);
CREATE INDEX contracts_mission ON contracts(mission_id,task_id,revision);
CREATE INDEX jobs_mission ON jobs(mission_id,lifecycle);
CREATE INDEX jobs_task ON jobs(task_id,purpose,assignment_generation);
CREATE INDEX evidence_candidate ON evidence(candidate_id,check_id);
CREATE INDEX effects_task ON effects(task_id,state);
CREATE INDEX drafts_owner ON drafts(owner_id,id);
CREATE INDEX conversation_draft ON conversation(draft_id,seq);
CREATE INDEX operations_actor ON operations(actor_id,id);
-- Preserve existing prototype tasks in the new projection, without certifying them.
INSERT INTO task_projection(task_id,owner_id,repo_id,mission_id,title,phase,why,version)
SELECT t.id,m.owner_id,c.repo_id,m.id,COALESCE(json_extract(c.raw_json,'$.title'),t.id),t.phase,
 'Legacy fixture evidence; requalification required',t.version
FROM tasks t JOIN contracts c ON c.task_id=t.id AND c.revision=t.active_revision JOIN missions m ON m.id=c.mission_id;
