"use client";

import { useEffect, useState, useCallback, use } from "react";
import {
  getRun,
  getWorld,
  subscribeEntries,
  subscribeStream,
  type RunRecord,
  type EntryDto,
  type ResourceDto,
  type CognitionEvent,
} from "@/lib/thymos-api";
import { EntryTimeline } from "@/components/trajectory/EntryTimeline";
import { StreamView } from "@/components/trajectory/StreamView";
import { WorldView } from "@/components/trajectory/WorldView";

export default function RunPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = use(params);
  const [run, setRun] = useState<RunRecord | null>(null);
  const [entries, setEntries] = useState<EntryDto[]>([]);
  const [streamEvents, setStreamEvents] = useState<CognitionEvent[]>([]);
  const [resources, setResources] = useState<ResourceDto[]>([]);
  const [tab, setTab] = useState<"timeline" | "stream" | "world">("timeline");

  // Poll run status.
  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      while (!cancelled) {
        try {
          const r = await getRun(id);
          setRun(r);
          if (r.status !== "running") break;
        } catch { /* retry */ }
        await new Promise((r) => setTimeout(r, 1000));
      }
    };
    poll();
    return () => { cancelled = true; };
  }, [id]);

  // SSE: ledger entries.
  useEffect(() => {
    const es = subscribeEntries(
      id,
      (entry) => setEntries((prev) => [...prev, entry]),
    );
    return () => es.close();
  }, [id]);

  // SSE: cognition stream.
  useEffect(() => {
    const es = subscribeStream(
      id,
      (evt) => setStreamEvents((prev) => [...prev, evt]),
    );
    return () => es.close();
  }, [id]);

  // Fetch world when run completes.
  const fetchWorld = useCallback(async () => {
    try {
      const w = await getWorld(id);
      setResources(w.resources ?? []);
    } catch { /* not ready */ }
  }, [id]);

  useEffect(() => {
    if (run?.status === "completed") fetchWorld();
  }, [run?.status, fetchWorld]);

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

      {/* Status bar */}
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
        {run?.task && (
          <span style={{ color: "#a1a1aa", flex: 1 }}>{run.task}</span>
        )}
        {run?.summary && (
          <span style={{ color: "#71717a", fontSize: 12 }}>
            {run.summary.steps_executed} steps, {run.summary.commits} commits,{" "}
            {run.summary.rejections} rejections
          </span>
        )}
      </div>

      {/* Final answer */}
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

      {/* Tabs */}
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

      {/* Tab content */}
      {tab === "timeline" && <EntryTimeline entries={entries} />}
      {tab === "stream" && <StreamView events={streamEvents} />}
      {tab === "world" && <WorldView resources={resources} />}
    </div>
  );
}
