PRAGMA foreign_keys = ON;
CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY);
INSERT INTO schema_migrations VALUES(1);
CREATE TABLE principals(
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('human','agent')),
  active INTEGER NOT NULL CHECK(active IN (0,1))
);
CREATE TABLE sessions(
  token TEXT PRIMARY KEY,
  principal_id TEXT NOT NULL REFERENCES principals(id),
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN (0,1))
);
CREATE TABLE commands(
  command_id TEXT PRIMARY KEY,
  actor_id TEXT NOT NULL REFERENCES principals(id),
  kind TEXT NOT NULL,
  body_sha TEXT NOT NULL,
  raw_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE TABLE operations(
  id TEXT PRIMARY KEY,
  command_id TEXT NOT NULL UNIQUE REFERENCES commands(command_id),
  actor_id TEXT NOT NULL REFERENCES principals(id),
  status TEXT NOT NULL,
  result_json TEXT NOT NULL
);
INSERT INTO principals VALUES('owner','human',1);
