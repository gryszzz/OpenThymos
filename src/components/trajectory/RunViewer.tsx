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
    <main
      style={{
        width: "min(1380px, calc(100% - 32px))",
        margin: "0 auto",
        padding: "28px 0 56px",
        color: "#eef4fb",
      }}
    >
      <section
        style={{
          position: "relative",
          overflow: "hidden",
          borderRadius: 28,
          border: "1px solid rgba(146, 163, 184, 0.16)",
          background:
            "linear-gradient(180deg, rgba(255,255,255,0.05), transparent 18%), rgba(8, 12, 18, 0.92)",
          boxShadow: "0 28px 80px rgba(0,0,0,0.34)",
          padding: "26px 24px 22px",
        }}
      >
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            inset: "-18% auto auto 62%",
            width: 320,
            height: 320,
            borderRadius: "50%",
            background: `radial-gradient(circle, ${palette.glow}, transparent 70%)`,
            filter: "blur(18px)",
          }}
        />

        <div
          style={{
            position: "relative",
            display: "grid",
            gap: 20,
          }}
        >
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: 12,
              alignItems: "center",
            }}
          >
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: 8,
                padding: "6px 12px",
                borderRadius: 999,
                border: `1px solid ${palette.glow}`,
                background: "rgba(255,255,255,0.04)",
                fontSize: 12,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
                color: palette.tone,
                fontWeight: 700,
              }}
            >
              <span
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  background: palette.tone,
                  boxShadow: `0 0 18px ${palette.glow}`,
                }}
              />
              {palette.label}
            </span>
            <span style={{ color: "rgba(201, 212, 225, 0.68)", fontSize: 12, letterSpacing: "0.12em", textTransform: "uppercase" }}>
              Thymos Runtime
            </span>
            <code style={{ color: "rgba(201, 212, 225, 0.72)", fontSize: 12, marginLeft: "auto" }}>
              {id}
            </code>
          </div>

          <div style={{ display: "grid", gap: 10 }}>
            <h1
              style={{
                margin: 0,
                fontSize: "clamp(1.8rem, 3.6vw, 2.8rem)",
                lineHeight: 1.06,
                fontFamily: "var(--font-display), sans-serif",
              }}
            >
              Unified Thymos execution console
            </h1>
            <p style={{ margin: 0, color: "rgba(223, 232, 243, 0.78)", lineHeight: 1.65, maxWidth: 920 }}>
              {session?.task ?? "Loading task context..."}
            </p>
            <div style={{ color: palette.tone, fontSize: 14, fontWeight: 600 }}>
              {session?.operator_state ?? "Connecting to the shared runtime state..."}
            </div>
          </div>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
              gap: 12,
            }}
          >
            <MetricCard label="Step" value={`${session?.current_step ?? 0}/${session?.max_steps ?? 0}`} />
            <MetricCard label="Intents" value={String(session?.counters.intents_declared ?? 0)} />
            <MetricCard label="Commits" value={String(session?.counters.commits ?? 0)} />
            <MetricCard label="Recoveries" value={String((session?.counters.recoveries ?? 0) + (session?.counters.retries ?? 0))} />
            <MetricCard label="Approvals" value={String(session?.counters.approvals_pending ?? 0)} />
            <MetricCard label="Active Tool" value={session?.active_tool ?? "standby"} />
          </div>
        </div>
      </section>

      <section
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1.3fr) minmax(300px, 0.7fr)",
          gap: 18,
          marginTop: 18,
        }}
      >
        <div
          style={{
            borderRadius: 24,
            border: "1px solid rgba(146, 163, 184, 0.16)",
            background:
              "linear-gradient(180deg, rgba(255,255,255,0.04), transparent 18%), rgba(8, 12, 18, 0.9)",
            padding: 18,
            minWidth: 0,
          }}
        >
          <div style={{ display: "flex", gap: 10, flexWrap: "wrap", marginBottom: 16 }}>
            {(["execution", "stream", "world"] as ConsoleTab[]).map((item) => (
              <button
                key={item}
                onClick={() => setTab(item)}
                style={{
                  border: tab === item ? "1px solid rgba(119, 169, 255, 0.3)" : "1px solid rgba(146, 163, 184, 0.16)",
                  background: tab === item ? "rgba(119, 169, 255, 0.12)" : "rgba(255,255,255,0.03)",
                  color: tab === item ? "#eef4fb" : "rgba(201, 212, 225, 0.74)",
                  padding: "10px 14px",
                  borderRadius: 999,
                  fontSize: 12,
                  letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  cursor: "pointer",
                }}
              >
                {item}
              </button>
            ))}
          </div>

          {tab === "execution" && <ExecutionLog session={session} />}
          {tab === "stream" && <StreamView events={streamEvents} />}
          {tab === "world" && <WorldView resources={resources} />}
        </div>

        <aside
          style={{
            display: "grid",
            gap: 18,
            alignSelf: "start",
          }}
        >
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

          <div
            style={{
              borderRadius: 22,
              border: "1px solid rgba(146, 163, 184, 0.16)",
              background: "rgba(8, 12, 18, 0.88)",
              padding: 16,
            }}
          >
            <div style={{ display: "flex", justifyContent: "space-between", gap: 12, marginBottom: 12 }}>
              <strong style={{ fontSize: 13, letterSpacing: "0.12em", textTransform: "uppercase", color: "#c9d4e1" }}>
                World Replay
              </strong>
              <span style={{ color: "rgba(150, 163, 181, 0.8)", fontSize: 12 }}>{currentScrubSeq} / {maxSeq}</span>
            </div>

            {maxSeq > 0 ? (
              <>
                <input
                  type="range"
                  min={0}
                  max={maxSeq}
                  value={currentScrubSeq}
                  onChange={(event) => setScrubSeq(Number(event.target.value))}
                  style={{ width: "100%" }}
                  aria-label="Trajectory replay scrubber"
                />
                <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginTop: 12 }}>
                  {scrubSeq !== null && (
                    <PanelButton onClick={() => setScrubSeq(null)}>Jump To Head</PanelButton>
                  )}
                  {(() => {
                    const commit = commitLogs.find((entry) => entry.seq === currentScrubSeq);
                    if (!commit?.commit_id) return null;
                    return <PanelButton onClick={() => handleBranch(commit.commit_id!)}>Branch Here</PanelButton>;
                  })()}
                </div>
              </>
            ) : (
              <p style={{ margin: 0, color: "rgba(201, 212, 225, 0.72)", lineHeight: 1.6 }}>
                Commit-backed world replay becomes available once the runtime records execution results.
              </p>
            )}

            {branchMsg && (
              <p style={{ margin: "12px 0 0", color: "#cfe0ff", fontSize: 12, lineHeight: 1.5 }}>{branchMsg}</p>
            )}
          </div>
        </aside>
      </section>
    </main>
  );
}

function MetricCard({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        borderRadius: 18,
        border: "1px solid rgba(146, 163, 184, 0.14)",
        background: "rgba(255,255,255,0.03)",
        padding: "14px 16px",
      }}
    >
      <div style={{ color: "rgba(150, 163, 181, 0.82)", fontSize: 11, letterSpacing: "0.12em", textTransform: "uppercase" }}>
        {label}
      </div>
      <div style={{ marginTop: 8, fontSize: 20, fontWeight: 700, color: "#eef4fb" }}>{value}</div>
    </div>
  );
}

function SidePanel({ title, body, accent = "#77a9ff" }: { title: string; body: string; accent?: string }) {
  return (
    <div
      style={{
        borderRadius: 22,
        border: "1px solid rgba(146, 163, 184, 0.16)",
        background: "rgba(8, 12, 18, 0.88)",
        padding: 16,
      }}
    >
      <strong style={{ display: "block", marginBottom: 10, fontSize: 13, letterSpacing: "0.12em", textTransform: "uppercase", color: "#c9d4e1" }}>
        {title}
      </strong>
      <div
        style={{
          borderLeft: `2px solid ${accent}`,
          paddingLeft: 12,
          color: "rgba(223, 232, 243, 0.8)",
          lineHeight: 1.7,
          whiteSpace: "pre-wrap",
          fontSize: 13,
        }}
      >
        {body}
      </div>
    </div>
  );
}

function PanelButton({ onClick, children }: { onClick: () => void; children: ReactNode }) {
  return (
    <button
      onClick={onClick}
      style={{
        border: "1px solid rgba(119, 169, 255, 0.22)",
        background: "rgba(119, 169, 255, 0.1)",
        color: "#eef4fb",
        padding: "9px 12px",
        borderRadius: 999,
        cursor: "pointer",
        fontSize: 12,
        letterSpacing: "0.08em",
        textTransform: "uppercase",
      }}
    >
      {children}
    </button>
  );
}
