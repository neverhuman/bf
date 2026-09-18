import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type {Command, Detail, Draft, Operation, Project, WorkItem} from "./contracts";

function sessionToken() {
  const fragment = new URLSearchParams(location.hash.slice(1));
  const supplied = fragment.get("token");
  const project = fragment.get("project");
  if (project && project !== localStorage.getItem("bf.project")) {localStorage.setItem("bf.project",project);localStorage.removeItem("bf.draft");localStorage.removeItem("bf.goal");}
  if (supplied) { sessionStorage.setItem("bf.session", supplied); history.replaceState(null, "", location.pathname); }
  return supplied ?? sessionStorage.getItem("bf.session") ?? "";
}
const phaseLabel = (phase: string) => phase.replaceAll("_", " ");
export function App() {
  const [token] = useState(sessionToken);
  const [projects, setProjects] = useState<Project[]>([]);
  const [project, setProject] = useState(() => localStorage.getItem("bf.project") ?? "");
  const [drafts, setDrafts] = useState<Draft[]>([]);
  const [selected, setSelected] = useState(() => localStorage.getItem("bf.draft") ?? "");
  const [editRevision, setEditRevision] = useState<number | null>(() => Number(localStorage.getItem("bf.editRevision")) || null);
  const [goal, setGoal] = useState(() => localStorage.getItem("bf.goal") ?? "");
  const [items, setItems] = useState<WorkItem[]>([]);
  const [detail, setDetail] = useState<Detail | null>(null);
  const [message, setMessage] = useState(token ? "Connecting…" : "Run bf to open an authenticated workbench.");
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [pending, setPending] = useState<string | null>(() => sessionStorage.getItem("bf.pending"));
  const [diagnostic, setDiagnostic] = useState<unknown>(null);
  const [accountsOpen, setAccountsOpen] = useState(false);
  const [account, setAccount] = useState<Record<string, unknown>>({});
  const selectedRef = useRef(selected);
  const draft = drafts.find(d => d.id === selected);
  const projectInfo = projects.find(p => p.id === project);
  const mission = draft?.mission_id;
  const activeItems = useMemo(() => items.filter(i => !mission || i.mission_id === mission), [items, mission]);
  const editable = !draft || ["proposed", "blocked", "failed"].includes(draft.status);

  const api = useCallback(async <T,>(path: string, init: RequestInit = {}): Promise<T> => {
    const response = await fetch(path, {...init, headers: {"Authorization": `Bearer ${token}`, ...(init.body ? {"Content-Type": "application/json"} : {}), ...init.headers}});
    const value = await response.json();
    if (!response.ok) throw new Error(value.message ?? `Request failed (${response.status})`);
    return value as T;
  }, [token]);

  const refresh = useCallback(async () => {
    const [p, d, w] = await Promise.all([api<{items: Project[]}>("/v3/projects"), api<{items: Draft[]}>("/v3/drafts"), api<{items: WorkItem[]}>("/v3/work")]);
    setProjects(p.items); setDrafts(d.items); setItems(w.items); setConnected(true);
    setProject(previous => p.items.some(x => x.id === previous) ? previous : (p.items.find(x => x.mode === "live")?.id ?? p.items[0]?.id ?? ""));
    if (selectedRef.current) setDetail(await api<Detail>(`/v3/drafts/${encodeURIComponent(selectedRef.current)}`));
  }, [api]);

  useEffect(() => {selectedRef.current = selected; localStorage.setItem("bf.draft", selected);}, [selected]);
  useEffect(() => {localStorage.setItem("bf.goal", goal);}, [goal]);
  useEffect(() => {if (editRevision) localStorage.setItem("bf.editRevision",String(editRevision));else localStorage.removeItem("bf.editRevision");}, [editRevision]);
  useEffect(() => {localStorage.setItem("bf.project", project);}, [project]);
  useEffect(() => {
    if (!token) return;
    let cancelled = false;
    const controller = new AbortController();
    void refresh().then(() => setMessage("Connected. Status uses no model calls.")).catch(error => setMessage(String(error)));
    async function subscribe() {
      while (!cancelled) {
        try {
          const response = await fetch("/v3/events", {headers: {Authorization: `Bearer ${token}`}, signal: controller.signal});
          if (!response.ok || !response.body) throw new Error("Session unavailable. Run bf to reconnect.");
          const reader = response.body.getReader(); const decoder = new TextDecoder(); let buffer = ""; let lastCursor: number | null = null;
          while (!cancelled) {
            const {value, done} = await reader.read(); if (done) throw new Error("Connection interrupted. Reconnecting…");
            buffer += decoder.decode(value, {stream: true});
            let boundary: number;
            while ((boundary = buffer.indexOf("\n\n")) >= 0) {
              const frame = buffer.slice(0, boundary); buffer = buffer.slice(boundary + 2);
              if (frame.includes("event: disconnected")) throw new Error("Session expired or revoked. Run bf to reconnect.");
              const data = frame.split("\n").find(line => line.startsWith("data: "))?.slice(6);
              if (data) {const snapshot = JSON.parse(data) as {cursor: number};if (snapshot.cursor !== lastCursor) {await refresh();lastCursor = snapshot.cursor;}setConnected(true);}
            }
            if (buffer.length > 262144) throw new Error("Update exceeded the allowed size. Reconnecting…");
          }
        } catch (error) {
          if (cancelled) return;
          setConnected(false); setMessage(String(error));
          await new Promise(resolve => setTimeout(resolve, 1500));
          if (!cancelled) await refresh().catch(() => undefined);
        }
      }
    }
    void subscribe();
    return () => {cancelled = true; controller.abort();};
  }, [token, refresh]);
  useEffect(() => {
    if (selected && connected) void api<Detail>(`/v3/drafts/${encodeURIComponent(selected)}`).then(setDetail).catch(error => setMessage(String(error)));
  }, [selected, connected, draft?.status, items, api]);

  const submit = useCallback(async (raw: string) => {
    if (busy) return;
    setBusy(true); setPending(raw); sessionStorage.setItem("bf.pending", raw);
    try {
      const operation = await api<Operation>("/v3/commands", {method: "POST", body: raw});
      if (operation.result.revision) setEditRevision(operation.result.revision);
      setDiagnostic(operation); setPending(null); sessionStorage.removeItem("bf.pending");
      if (operation.result.draft_id) {setSelected(operation.result.draft_id);selectedRef.current = operation.result.draft_id;}
      setMessage(operation.result.status === "blocked" ? "Goal saved. Live planning needs a qualified runner, Codex connection and current grant. Operating HOLD remains active." : "Saved. Following durable work…");
      await refresh();
    } catch (error) {setMessage(String(error));} finally {setBusy(false);}
  }, [busy, api, refresh]);
  const command = useCallback((kind: Command["kind"], target: string | null, version: number | null, payload: Command["payload"] = {}) => {
    const body: Command = {schema_version: 3, command_id: crypto.randomUUID(), kind, target_id: target, expected_version: version, payload};
    void submit(JSON.stringify(body));
  }, [submit]);
  function review() {if (!goal.trim() || !project || !editable) return;command(draft ? "edit_draft" : "run", draft?.id ?? null, draft ? (editRevision ?? draft.revision) : null, {goal, project_id: project});}
  function selectDraft(value: Draft) {setEditRevision(value.revision);setSelected(value.id);setGoal(value.goal);setProject(value.project_id);setDetail(null);}
  function newGoal() {setEditRevision(null);setSelected("");setGoal("");setDetail(null);}
  function demo() {newGoal();setProject("demo");setGoal("Reject repeated delivery IDs");setMessage("Demo selected. Review its fixed fixture plan before starting.");}

  const workList = useMemo(() => <section aria-label="Work"><h2>Work</h2>{activeItems.length === 0 ? <p className="muted">Tasks appear after you accept a plan.</p> : activeItems.map(item => <article className="card" key={item.task_id}><h3>{item.title}</h3><p className="status">{phaseLabel(item.phase)}</p><p>{item.why}</p>{item.phase === "failed" && <button disabled={busy} onClick={() => command("retry_task",item.task_id,item.version)}>Retry work within plan limits</button>}{item.pr && (item.pr.fixture_only ? <p>Simulated draft #{item.pr.number}</p> : item.pr.url.startsWith("https://") && <a href={item.pr.url} target="_blank" rel="noreferrer">Open draft PR #{item.pr.number}</a>)}</article>)}</section>, [activeItems, busy, command]);

  return <main>
    <header className="row spread"><a className="brand" href="/">bf</a><span className={connected ? "status connected" : "status"}>{connected ? "Connected" : "Disconnected"}</span><button className="ghost" onClick={() => {setAccountsOpen(!accountsOpen);void api<{provider: Record<string, unknown>}>("/v3/doctor").then(d => setAccount(d.provider)).catch(error => setMessage(String(error)));}}>Accounts</button></header>
    <h1>What should happen?</h1>
    <p className="muted">Describe a goal, review its plan, then follow the changes and checks.</p>
    <label htmlFor="project">Project</label>
    <select id="project" value={project} disabled={!!draft} onChange={event => setProject(event.target.value)}><option value="" disabled>Select a project</option>{projects.map(p => <option key={p.id} value={p.id}>{p.name}{p.mode === "fixture" ? " · Demo" : ""}</option>)}</select>
    {projectInfo?.mode === "fixture" && <p className="notice">Demo: fixed fixture code and simulated pull requests. This does not execute your repository or use a model.</p>}
    <label htmlFor="goal">Goal</label>
    <textarea id="goal" value={goal} disabled={!editable} onChange={event => setGoal(event.target.value)} onKeyDown={event => {if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {event.preventDefault();review();}}} placeholder="Describe the behavior you want and how we can check it." />
    <div className="row"><button disabled={busy || !connected || !goal.trim() || !project || !editable} onClick={review}>{draft ? "Revise plan" : "Review plan"}</button><button className="ghost" onClick={newGoal}>New goal</button><button className="ghost" onClick={demo}>Try the demo</button></div>
    <p role="status" aria-live="polite" className="message">{message}</p>
    {pending && <div className="row"><button className="ghost" disabled={busy} onClick={() => void submit(pending)}>Retry same request</button><button className="ghost" disabled={busy} onClick={() => {setPending(null);sessionStorage.removeItem("bf.pending");}}>Dismiss request</button></div>}
    {accountsOpen && <section className="card" aria-label="Accounts"><h2>Codex subscription</h2><dl><dt>Installed</dt><dd>{String(account.installed ?? "Not yet probed")}</dd><dt>Authenticated</dt><dd>{phaseLabel(String(account.authenticated ?? "Not yet checked"))}</dd><dt>Expired</dt><dd>{String(account.expired ?? "Unknown")}</dd><dt>Qualified</dt><dd>No</dd></dl><p>Live connection is unavailable while runner and transport qualification are incomplete. Your goal and draft remain saved. No provider credentials are requested here.</p></section>}
    {draft && <section className="card" aria-label="Plan review"><div className="row spread"><h2>Plan · revision {draft.revision}</h2><span>{phaseLabel(draft.status)}</span></div>
      {draft.plan ? <><p>{draft.plan.objective}</p><h3>Acceptance</h3><ul>{draft.plan.acceptance.map(a => <li key={a.id}>{a.statement}</li>)}</ul><dl><dt>Scope</dt><dd>{draft.plan.scope.join(", ")}</dd><dt>Account / model</dt><dd>{draft.plan.account} · {draft.plan.model}</dd><dt>Limits</dt><dd>{draft.plan.limits.write_invocations} writing attempts · {draft.plan.limits.runtime_seconds} seconds</dd><dt>Checks</dt><dd>{draft.plan.checks.join(", ")}</dd></dl><details><summary>Advanced details</summary><p>Source base: <code>{draft.plan.base_oid}</code></p><p>Approval binds this exact revision. Scope, spending, permissions and destination changes require a new decision.</p></details><button disabled={busy || draft.status !== "proposed" || goal !== draft.goal} onClick={() => command("start_work", draft.id, draft.revision)}>Start work</button></> : <p>{draft.status === "planning" ? "A bounded planning job is preparing the proposal…" : "Live planning is blocked. Qualification and a current grant are required before a provider can run."}</p>}
      <div className="row controls"><button className="ghost" disabled={busy || draft.status === "cancelled"} onClick={() => command(draft.paused ? "resume" : "pause", draft.mission_id, draft.mission_version)}>{draft.paused ? "Resume dispatch" : "Pause dispatch"}</button><button className="ghost" disabled={busy || draft.status === "cancelled"} onClick={() => command("stop", draft.mission_id, draft.mission_version)}>Stop</button><button className="ghost" disabled={busy || draft.status === "cancelled"} onClick={() => command("cancel", draft.mission_id, draft.mission_version)}>Cancel goal</button></div>
    </section>}
    {workList}
    {detail && <section className="card" aria-label="Conversation and jobs"><h2>Conversation</h2>{detail.conversation.map(line => <p key={line.seq}><strong>{line.role}: </strong>{line.body}</p>)}<h3>Jobs</h3>{detail.jobs.map(job => <p key={job.id}>{job.purpose}: {phaseLabel(job.lifecycle)} · process {job.occupancy}</p>)}</section>}
    <section aria-label="Recent goals"><h2>Recent goals</h2><div className="list">{drafts.map(d => <button className="item" key={d.id} onClick={() => selectDraft(d)} aria-current={d.id === selected ? "true" : undefined}>{d.goal}<span className="muted">{phaseLabel(d.status)}</span></button>)}</div></section>
    <details><summary>Details and diagnostics</summary><button className="ghost" onClick={() => void refresh().catch(error => setMessage(String(error)))}>Reconnect snapshot</button><pre>{JSON.stringify(diagnostic ?? detail, null, 2)}</pre></details>
  </main>;
}
