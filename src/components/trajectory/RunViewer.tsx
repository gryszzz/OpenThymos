"use client";

import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  branchFrom,
  getExecution,
  getWorld,
  getWorldAt,
  subscribeExecution,
  subscribeStream,
  type CognitionEvent,
  type ExecutionSession,
  type ResourceDto,
} from "@/lib/thymos-api";
import { ExecutionLog } from "@/components/trajectory/ExecutionLog";
import { StreamView } from "@/components/trajectory/StreamView";
import { WorldView } from "@/components/trajectory/WorldView";

type ConsoleTab = "execution" | "stream" | "world";

const statusStyles: Record<
  ExecutionSession["status"],
  { tone: string; glow: string; label: string }
> = {
  running: { tone: "#7dd3fc", glow: "rgba(125, 211, 252, 0.38)", label: "Live" },
  waiting_approval: { tone: "#fbbf24", glow: "rgba(251, 191, 36, 0.34)", label: "Awaiting approval" },
  completed: { tone: "#34d399", glow: "rgba(52, 211, 153, 0.34)", label: "Resolved" },
  failed: { tone: "#f87171", glow: "rgba(248, 113, 113, 0.34)", label: "Failed" },
  cancelled: { tone: "#94a3b8", glow: "rgba(148, 163, 184, 0.28)", label: "Cancelled" },
};

const tabLabels: Record<ConsoleTab, string> = {
  execution: "Execution Log",
  stream: "Model Stream",
  world: "World State",
};

export function RunViewer({ id }: { id: string }) {
  const [session, setSession] = useState<ExecutionSession | null>(null);
  const [streamEvents, setStreamEvents] = useState<CognitionEvent[]>([]);
  const [resources, setResources] = useState<ResourceDto[]>([]);
  const [tab, setTab] = useState<ConsoleTab>("execution");
  const [scrubSeq, setScrubSeq] = useState<number | null>(null);
  const [branchMsg, setBranchMsg] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void getExecution(id)
      .then((snapshot) => {
        if (!cancelled) {
          setSession(snapshot);
        }
      })
      .catch(() => {
        /* handled by stream fallback */
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  useEffect(() => {
    const es = subscribeExecution(id, (snapshot) => setSession(snapshot));
    return () => es.close();
  }, [id]);

  useEffect(() => {
    const es = subscribeStream(id, (evt) => setStreamEvents((prev) => [...prev, evt]));
    return () => es.close();
  }, [id]);

  const fetchWorld = useCallback(async () => {
    try {
      if (scrubSeq !== null) {
        const world = await getWorldAt(id, scrubSeq);
        setResources(world.resources ?? []);
      } else {
        const world = await getWorld(id);
        setResources(world.resources ?? []);
      }
    } catch {
      /* world not ready */
    }
  }, [id, scrubSeq]);

  useEffect(() => {
    if (tab === "world") {
      void fetchWorld();
    }
  }, [tab, fetchWorld]);

  const commitLogs = useMemo(
    () => session?.log.filter((entry) => entry.commit_id && entry.seq !== undefined) ?? [],
    [session],
  );
  const maxSeq = commitLogs.reduce((max, entry) => Math.max(max, entry.seq ?? 0), 0);
  const currentScrubSeq = scrubSeq ?? maxSeq;

  const handleBranch = useCallback(
    async (commitId: string) => {
      try {
        const res = await branchFrom(id, commitId, "thymos operator branch");
        setBranchMsg(`Shadow branch created at ${res.branch_trajectory_id.slice(0, 12)}...`);
      } catch (error) {
        setBranchMsg(`Branch failed: ${(error as Error).message}`);
      }
    },
    [id],
  );

  const status = session?.status ?? "running";
  const palette = statusStyles[status];

  return (
    <main className="thymos-runtime-shell">
      <section className="thymos-console-panel thymos-runtime-header">
        <div
          aria-hidden="true"
          className="thymos-console-orb"
          style={{
            inset: "-18% auto auto 62%",
            width: 320,
            height: 320,
            background: `radial-gradient(circle, ${palette.glow}, transparent 70%)`,
          }}
        />

        <div className="thymos-console-topline">
          <span
            className="thymos-status-pill"
            style={{ borderColor: palette.glow, color: palette.tone }}
          >
            <span
              className="thymos-status-pill-dot"
              style={{ background: palette.tone, boxShadow: `0 0 18px ${palette.glow}` }}
            />
            {palette.label}
          </span>
          <span className="thymos-eyebrow">Thymos Runtime</span>
          <code className="thymos-console-id">{id}</code>
        </div>

        <div className="thymos-runtime-task">
          <h1>Unified Thymos execution console</h1>
          <p>{session?.task ?? "Loading task context..."}</p>
          <div className="thymos-operator-state" style={{ color: palette.tone }}>
            {session?.operator_state ?? "Connecting to the shared runtime state..."}
          </div>
        </div>

        <div className="thymos-console-stat-grid">
          <MetricCard label="Step" value={`${session?.current_step ?? 0}/${session?.max_steps ?? 0}`} />
          <MetricCard label="Intents" value={String(session?.counters.intents_declared ?? 0)} />
          <MetricCard label="Commits" value={String(session?.counters.commits ?? 0)} />
          <MetricCard label="Recoveries" value={String((session?.counters.recoveries ?? 0) + (session?.counters.retries ?? 0))} />
          <MetricCard label="Approvals" value={String(session?.counters.approvals_pending ?? 0)} />
          <MetricCard label="Active Tool" value={session?.active_tool ?? "standby"} />
        </div>
      </section>

      <section className="thymos-console-layout">
        <div className="thymos-console-main">
          <div className="thymos-tab-row">
            {(["execution", "stream", "world"] as ConsoleTab[]).map((item) => (
              <button
                key={item}
                className={tab === item ? "thymos-tab is-active" : "thymos-tab"}
                onClick={() => setTab(item)}
              >
                {tabLabels[item]}
              </button>
            ))}
          </div>

          {tab === "execution" && <ExecutionLog session={session} />}
          {tab === "stream" && <StreamView events={streamEvents} />}
          {tab === "world" && <WorldView resources={resources} />}
        </div>

        <aside className="thymos-console-sidebar">
          <SidePanel
            title="Execution State"
            body={`Phase: ${session?.phase ?? "system"}\nLast update: ${
              session ? new Date(session.updated_at_ms).toLocaleTimeString() : "--"
            }\nTrajectory: ${session?.trajectory_id ? session.trajectory_id.slice(0, 16) : "pending"}`}
          />

          <SidePanel
            title="Outcome"
            body={session?.final_answer ?? "Final answer will appear here when the runtime resolves the task."}
            accent={session?.status === "completed" ? "#34d399" : "#77a9ff"}
          />

          <div className="thymos-console-panel-soft thymos-panel">
            <div className="thymos-world-replay-head">
              <strong className="thymos-panel-title">World Replay</strong>
              <span>
                {currentScrubSeq} / {maxSeq}
              </span>
            </div>

            {maxSeq > 0 ? (
              <>
                <input
                  className="thymos-range"
                  type="range"
                  min={0}
                  max={maxSeq}
                  value={currentScrubSeq}
                  onChange={(event) => setScrubSeq(Number(event.target.value))}
                  aria-label="Trajectory replay scrubber"
                />
                <div className="thymos-chip-row" style={{ marginTop: 12 }}>
                  {scrubSeq !== null ? (
                    <PanelButton onClick={() => setScrubSeq(null)}>Jump To Head</PanelButton>
                  ) : null}
                  {(() => {
                    const commit = commitLogs.find((entry) => entry.seq === currentScrubSeq);
                    if (!commit?.commit_id) return null;
                    return <PanelButton onClick={() => handleBranch(commit.commit_id!)}>Branch Here</PanelButton>;
                  })()}
                </div>
              </>
            ) : (
              <div className="thymos-empty-state">
                <strong>Replay Waiting</strong>
                <p>
                  Commit-backed world replay becomes available once the runtime records execution
                  results.
                </p>
              </div>
            )}

            {branchMsg ? (
              <p className="thymos-panel-copy" style={{ marginTop: 12, color: "#cfe0ff" }}>
                {branchMsg}
              </p>
            ) : null}
          </div>
        </aside>
      </section>
    </main>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="thymos-console-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SidePanel({ title, body, accent = "#77a9ff" }: { title: string; body: string; accent?: string }) {
  return (
    <div className="thymos-console-panel-soft thymos-panel">
      <strong className="thymos-panel-title">{title}</strong>
      <div className="thymos-panel-bar" style={{ borderLeftColor: accent }}>
        {body}
      </div>
    </div>
  );
}

function PanelButton({ onClick, children }: { onClick: () => void; children: ReactNode }) {
  return (
    <button onClick={onClick} className="thymos-button-secondary">
      {children}
    </button>
  );
}
