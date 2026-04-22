"use client";

import { useEffect, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import { RunViewer } from "@/components/trajectory/RunViewer";
import { createRun } from "@/lib/thymos-api";

const suggestedTasks = [
  {
    title: "Repository inspection",
    body: "Map the repo, explain the architecture, and call out the highest-value next steps.",
  },
  {
    title: "Runtime verification",
    body: "Trace the execution loop, run the right checks, and confirm the runtime is healthy.",
  },
  {
    title: "Product polish",
    body: "Tune the Thymos interfaces for clarity and brand consistency, then verify the affected surfaces.",
  },
];

export default function RunsPage() {
  const router = useRouter();
  const [runId, setRunId] = useState<string | null>(null);
  const [task, setTask] = useState("");
  const [maxSteps, setMaxSteps] = useState(24);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    setRunId(new URLSearchParams(window.location.search).get("id"));
  }, []);

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!task.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const result = await createRun(task, maxSteps);
      router.push(`/runs?id=${encodeURIComponent(result.run_id)}`);
    } catch (err) {
      setError(String(err));
      setLoading(false);
    }
  }

  if (runId) {
    return <RunViewer id={runId} />;
  }

  return (
    <main className="thymos-runtime-shell">
      <section className="thymos-console-panel thymos-runtime-hero">
        <div className="thymos-console-orb thymos-console-orb-a" aria-hidden="true" />
        <div className="thymos-console-orb thymos-console-orb-b" aria-hidden="true" />

        <div className="thymos-runtime-copy">
          <div className="thymos-eyebrow">Thymos Unified Runtime</div>
          <h1 className="thymos-runtime-title">
            Start one task. Watch the same execution session everywhere.
          </h1>
          <p className="thymos-runtime-summary">
            Thymos runs the full operator loop from intent to result against a shared backend
            runtime. CLI, sidebar, terminal, and web console all attach to the same live execution
            state, approvals, and completion record.
          </p>

          <div className="thymos-runtime-chip-grid">
            {[
              "Intent → Proposal → Execution → Result",
              "Shared live execution log",
              "Runtime-led recovery and retries",
              "Replayable world state and branching",
            ].map((item) => (
              <div className="thymos-runtime-chip" key={item}>
                {item}
              </div>
            ))}
          </div>

          <div className="thymos-console-banner">
            <strong>Operator Feed</strong>
            <p>
              Submit a task once, then watch the runtime plan, act, recover, and finish through
              the same observable loop every surface shares.
            </p>
            <div className="thymos-console-command">
              <span>$</span>
              <code>thymos run --follow "Inspect the runtime, verify the result, and report back"</code>
            </div>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="thymos-console-panel-soft thymos-runtime-form">
          <label htmlFor="task" className="thymos-field">
            <span className="thymos-field-label">Operator Task</span>
            <textarea
              id="task"
              className="thymos-textarea"
              value={task}
              onChange={(event) => setTask(event.target.value)}
              placeholder="Refactor the TypeScript API client, run the relevant checks, and leave the runtime in a verified state."
              rows={7}
            />
          </label>

          <div className="thymos-suggestion-row">
            {suggestedTasks.map((suggestion) => (
              <button
                key={suggestion.title}
                type="button"
                className="thymos-suggestion"
                onClick={() => setTask(suggestion.body)}
              >
                <strong>{suggestion.title}</strong>
                <span>{suggestion.body}</span>
              </button>
            ))}
          </div>

          <div className="thymos-form-row">
            <label className="thymos-field">
              <span className="thymos-field-label">Max Steps</span>
              <input
                className="thymos-number-input"
                type="number"
                value={maxSteps}
                min={1}
                max={100}
                onChange={(event) => setMaxSteps(Number(event.target.value))}
              />
            </label>

            <button type="submit" disabled={loading || !task.trim()} className="thymos-button">
              {loading ? "Launching Runtime" : "Start Thymos Run"}
            </button>
          </div>

          {error ? <p className="thymos-error">{error}</p> : null}
        </form>
      </section>
    </main>
  );
}
