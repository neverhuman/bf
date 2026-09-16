import { useMemo, useState } from "react";

type WorkItem = {
  task_id: string;
  title: string;
  phase: string;
  why: string;
  pr?: { number?: number; url?: string } | null;
};

export function App() {
  const [goal, setGoal] = useState("Reject repeated delivery IDs");
  const [items, setItems] = useState<WorkItem[]>([]);
  const [open, setOpen] = useState<WorkItem | null>(null);
  const [log, setLog] = useState("Models are optional. Status is not.");
  const [busy, setBusy] = useState(false);

  const headline = useMemo(
    () => (items[0]?.phase === "review_ready" ? "A draft PR is waiting." : "What should happen?"),
    [items],
  );

  async function refresh() {
    const r = await fetch("/v3/work");
    const data = await r.json();
    setItems(data.items ?? []);
  }

  async function runDemo() {
    setBusy(true);
    setLog("Running the fake loop…");
    try {
      const r = await fetch("/v3/demo/basic", { method: "POST" });
      const data = await r.json();
      setLog(JSON.stringify(data, null, 2));
      await refresh();
    } catch (err) {
      setLog(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function runGoal() {
    setBusy(true);
    try {
      const r = await fetch("/v3/commands", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          schema_version: 3,
          command_id: crypto.randomUUID(),
          kind: "run",
          target_id: null,
          expected_version: null,
          payload: { goal },
        }),
      });
      setLog(await r.text());
      await refresh();
    } finally {
      setBusy(false);
    }
  }

  return (
    <main>
      <p className="kicker">ask · inspect · steer · take over</p>
      <h1>{headline}</h1>
      <textarea
        aria-label="Goal"
        value={goal}
        onChange={(e) => setGoal(e.target.value)}
        placeholder="Fix the flaky test in foo"
      />
      <div className="row">
        <button disabled={busy} onClick={runDemo}>
          Try the demo
        </button>
        <button className="ghost" disabled={busy} onClick={runGoal}>
          Save goal
        </button>
        <button className="ghost" disabled={busy} onClick={refresh}>
          Refresh
        </button>
      </div>
      <div className="list">
        {items.map((item) => (
          <button
            key={item.task_id}
            className="item"
            onClick={() => setOpen(item)}
            style={{ textAlign: "left", width: "100%", borderRadius: 16 }}
          >
            <h2>
              {item.title} <span className="ok">{item.phase}</span>
            </h2>
            <div className="meta">{item.why}</div>
          </button>
        ))}
      </div>
      {open ? (
        <section className="drawer" aria-label="Task detail">
          <h2>{open.title}</h2>
          <p>{open.why}</p>
          {open.pr?.url ? (
            <p>
              Draft PR {open.pr.number}: {open.pr.url}
            </p>
          ) : null}
        </section>
      ) : null}
      <pre>{log}</pre>
    </main>
  );
}
