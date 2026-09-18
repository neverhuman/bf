-- Only future records are constrained; prototype evidence stays historical.
CREATE TRIGGER evidence_binding BEFORE INSERT ON evidence BEGIN
 SELECT CASE WHEN NOT EXISTS(
  SELECT 1 FROM jobs j JOIN candidates c ON c.id=NEW.candidate_id JOIN principals p ON p.id=NEW.producer_id
  WHERE j.id=NEW.job_id AND j.purpose='verify' AND j.lifecycle='running'
  AND p.kind='verifier' AND p.active=1 AND p.id='verifier-fixture'
  AND j.task_id=c.task_id AND j.task_revision=c.task_revision
  AND j.input_digest=NEW.subject_digest
  AND json_extract(j.envelope_json,'$.candidate_id')=c.id
  AND json_extract(NEW.receipt_json,'$.generation')=j.assignment_generation
 ) THEN RAISE(ABORT,'invalid_evidence_binding') END;
END;
CREATE TRIGGER candidate_binding BEFORE INSERT ON candidates WHEN NEW.producer_job_id IS NOT NULL BEGIN
 SELECT CASE WHEN NOT EXISTS(
  SELECT 1 FROM jobs j JOIN artifacts a ON a.id=NEW.artifact_id
  WHERE j.id=NEW.producer_job_id AND j.task_id=NEW.task_id AND j.task_revision=NEW.task_revision
  AND j.purpose='implement' AND j.lifecycle='running' AND j.writer_authority='active' AND a.complete=1
 ) THEN RAISE(ABORT,'invalid_candidate_binding') END;
END;
CREATE TRIGGER candidates_no_delete BEFORE DELETE ON candidates BEGIN SELECT RAISE(ABORT,'immutable_candidate'); END;
INSERT INTO schema_migrations VALUES(3);
