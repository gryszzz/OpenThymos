"use client";

import { useCallback, useEffect, useState } from "react";
import {
  branchFrom,
  getRun,
  getWorld,
  getWorldAt,
  subscribeEntries,
  subscribeStream,
  type CognitionEvent,
  type EntryDto,
  type ResourceDto,
  type RunRecord,
} from "@/lib/thymos-api";
import { EntryTimeline } from "@/components/trajectory/EntryTimeline";
import { StreamView } from "@/components/trajectory/StreamView";
import { WorldView } from "@/components/trajectory/WorldView";

export function RunViewer({ id }: { id: string }) {
  const [run, setRun] = useState<RunRecord | null>(null);
  const [entries, setEntries] = useState<EntryDto[]>([]);
  const [streamEvents, setStreamEvents] = useState<CognitionEvent[]>([]);
  const [resources, setResources] = useState<ResourceDto[]>([]);
  const [tab, setTab] = useState<"timeline" | "stream" | "world">("timeline");
  const [scrubSeq, setScrubSeq] = useState<number | null>(null);
  const [branchMsg, setBranchMsg] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      while (!cancelled) {
        try {
          const r = await getRun(id);
          setRun(r);
          if (r.status !== "running") break;
        } catch {
          // retry
        }
        await new Promise((r) => setTimeout(r, 1000));
      }
    };
    poll();
    return () => {
      cancelled = true;
    };
  }, [id]);

  useEffect(() => {
    const es = subscribeEntries(id, (entry) => setEntries((prev) => [...prev, entry]));
    return () => es.close();
  }, [id]);

  useEffect(() => {
    const es = subscribeStream(id, (evt) => setStreamEvents((prev) => [...prev, evt]));
    return () => es.close();
  }, [id]);

  const fetchWorld = useCallback(async () => {
    try {
      if (scrubSeq !== null) {
        const w = await getWorldAt(id, scrubSeq);
        setResources(w.resources ?? []);
      } else {
        const w = await getWorld(id);
        setResources(w.resources ?? []);
      }
    } catch {
      // not ready
    }
  }, [id, scrubSeq]);

  const handleBranch = useCallback(
    async (commitId: string) => {
      try {
        const res = await branchFrom(id, commitId, "shadow branch from viewer");
        setBranchMsg(`Branched -> ${res.branch_trajectory_id.slice(0, 12)}...`);
      } catch (e) {
        setBranchMsg(`Branch failed: ${(e as Error).message}`);
      }
    },
    [id],
  );

  useEffect(() => {
    if (run?.status === "completed") fetchWorld();
  }, [run?.status, fetchWorld]);

  useEffect(() => {
    if (tab === "world") fetchWorld();
  }, [scrubSeq, tab, fetchWorld]);

  const commitEntries = entries.filter((e) => e.kind === "commit");
  const maxSeq = entries.reduce((m, e) => Math.max(m, e.seq ?? 0), 0);
  const currentScrubSeq = scrubSeq ?? maxSeq;

  const statusColor =
    run?.status === "completed"
      ? "#22c55e"
      : run?.status === "failed"
        ? "#ef4444"
        : "#f59e0b";

  return (
    <div
      style={{
        maxWidth: 960,
        margin: "0 auto",
        padding: "32px 24px",
        fontFamily: "var(--font-body), system-ui, sans-serif",
        color: "#d4d4d8",
      }}
    >
      <h1
        style={{
          fontFamily: "var(--font-display), system-ui, sans-serif",
          fontSize: 28,
          fontWeight: 700,
          marginBottom: 4,
        }}
      >
        Trajectory Viewer
      </h1>

      <p style={{ color: "#71717a", marginBottom: 24, fontSize: 13 }}>
        Run <code style={{ color: "#a1a1aa" }}>{id.slice(0, 12)}...</code>
      </p>

      <div
        style={{
          display: "flex",
          gap: 16,
          alignItems: "center",
          marginBottom: 24,
          padding: "12px 16px",
          background: "#18181b",
          borderRadius: 8,
          border: "1px solid #27272a",
        }}
      >
        <span
          style={{
            width: 10,
            height: 10,
            borderRadius: "50%",
            background: statusColor,
          }}
        />
        <span style={{ fontWeight: 600, textTransform: "capitalize" }}>
          {run?.status ?? "loading"}
        </span>
        {run?.task && <span style={{ color: "#a1a1aa", flex: 1 }}>{run.task}</span>}
        {run?.summary && (
          <span style={{ color: "#71717a", fontSize: 12 }}>
            {run.summary.steps_executed} steps, {run.summary.commits} commits,{" "}
            {run.summary.rejections} rejections
          </span>
        )}
      </div>

      {run?.summary?.final_answer && (
        <div
          style={{
            padding: "12px 16px",
            background: "#14532d",
            border: "1px solid #166534",
            borderRadius: 8,
            marginBottom: 24,
            fontSize: 14,
          }}
        >
          <strong>Final Answer:</strong> {run.summary.final_answer}
        </div>
      )}

      {maxSeq > 0 && (
        <div
          style={{
            marginBottom: 16,
            padding: "10px 14px",
            background: "#0f172a",
            border: "1px solid #1e293b",
            borderRadius: 8,
            display: "flex",
            gap: 12,
            alignItems: "center",
            fontSize: 12,
          }}
        >
          <span style={{ color: "#94a3b8", minWidth: 80 }}>Replay to seq</span>
          <input
            type="range"
            min={0}
            max={maxSeq}
            value={currentScrubSeq}
            onChange={(e) => setScrubSeq(Number(e.target.value))}
            style={{ flex: 1 }}
            aria-label="Trajectory scrubber"
          />
          <span style={{ color: "#e2e8f0", minWidth: 60, textAlign: "right" }}>
            {currentScrubSeq} / {maxSeq}
          </span>
          {scrubSeq !== null && scrubSeq !== maxSeq && (
            <button
              onClick={() => setScrubSeq(null)}
              style={{
                background: "none",
                border: "1px solid #334155",
                color: "#cbd5e1",
                padding: "3px 8px",
                borderRadius: 4,
                cursor: "pointer",
                fontSize: 11,
              }}
            >
              jump to head
            </button>
          )}
          {(() => {
            const commitAtSeq = commitEntries.find((e) => e.seq === currentScrubSeq);
            const commitId = commitAtSeq?.commit_id;
            if (!commitId) return null;
            return (
              <button
                onClick={() => handleBranch(commitId)}
                style={{
                  background: "#4338ca",
                  border: "1px solid #6366f1",
                  color: "#e0e7ff",
                  padding: "3px 10px",
                  borderRadius: 4,
                  cursor: "pointer",
                  fontSize: 11,
                  fontWeight: 600,
                }}
              >
                branch from here
              </button>
            );
          })()}
        </div>
      )}

      {branchMsg && (
        <div
          style={{
            marginBottom: 12,
            padding: "8px 12px",
            background: "#1e293b",
            border: "1px solid #334155",
            borderRadius: 6,
            fontSize: 12,
            color: "#cbd5e1",
          }}
        >
          {branchMsg}
        </div>
      )}

      <div
        style={{
          display: "flex",
          gap: 0,
          marginBottom: 16,
          borderBottom: "1px solid #27272a",
        }}
      >
        {(["timeline", "stream", "world"] as const).map((t) => (
          <button
            key={t}
            onClick={() => {
              setTab(t);
              if (t === "world") fetchWorld();
            }}
            style={{
              padding: "8px 20px",
              background: "none",
              border: "none",
              borderBottom: tab === t ? "2px solid #6366f1" : "2px solid transparent",
              color: tab === t ? "#e4e4e7" : "#71717a",
              cursor: "pointer",
              fontWeight: 600,
              fontSize: 14,
              textTransform: "capitalize",
            }}
          >
            {t}
            {t === "timeline" && entries.length > 0 && (
              <span
                style={{
                  marginLeft: 6,
                  background: "#27272a",
                  padding: "1px 6px",
                  borderRadius: 8,
                  fontSize: 11,
                }}
              >
                {entries.length}
              </span>
            )}
          </button>
        ))}
      </div>

      {tab === "timeline" && <EntryTimeline entries={entries} />}
      {tab === "stream" && <StreamView events={streamEvents} />}
      {tab === "world" && <WorldView resources={resources} />}
    </div>
  );
}
