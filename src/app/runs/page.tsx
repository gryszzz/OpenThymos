"use client";

import { useEffect, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import { RunViewer } from "@/components/trajectory/RunViewer";
import { createRun } from "@/lib/thymos-api";

export default function RunsPage() {
  const router = useRouter();
  const [runId, setRunId] = useState<string | null>(null);
  const [task, setTask] = useState("");
  const [maxSteps, setMaxSteps] = useState(16);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const id = new URLSearchParams(window.location.search).get("id");
    setRunId(id);
  }, []);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!task.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await createRun(task, maxSteps);
      router.push(`/runs?id=${encodeURIComponent(res.run_id)}`);
    } catch (err) {
      setError(String(err));
      setLoading(false);
    }
  }

  if (runId) {
    return <RunViewer id={runId} />;
  }

  return (
    <div
      style={{
        maxWidth: 640,
        margin: "0 auto",
        padding: "48px 24px",
        fontFamily: "var(--font-body), system-ui, sans-serif",
        color: "#d4d4d8",
      }}
    >
      <h1
        style={{
          fontFamily: "var(--font-display), system-ui, sans-serif",
          fontSize: 32,
          fontWeight: 700,
          marginBottom: 8,
        }}
      >
        Thymos Runs
      </h1>
      <p style={{ color: "#71717a", marginBottom: 32 }}>
        Start a new governed agent run. The agent operates under a signed Writ
        with scoped capabilities.
      </p>

      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 16 }}>
        <label style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <span style={{ fontWeight: 600, fontSize: 14 }}>Task</span>
          <textarea
            value={task}
            onChange={(e) => setTask(e.target.value)}
            placeholder="e.g. Set greeting to hello thymos and read it back"
            rows={3}
            style={{
              background: "#18181b",
              border: "1px solid #27272a",
              borderRadius: 6,
              padding: 12,
              color: "#d4d4d8",
              fontSize: 14,
              resize: "vertical",
            }}
          />
        </label>

        <label style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <span style={{ fontWeight: 600, fontSize: 14 }}>Max Steps</span>
          <input
            type="number"
            value={maxSteps}
            onChange={(e) => setMaxSteps(Number(e.target.value))}
            min={1}
            max={100}
            style={{
              background: "#18181b",
              border: "1px solid #27272a",
              borderRadius: 6,
              padding: "8px 12px",
              color: "#d4d4d8",
              fontSize: 14,
              width: 100,
            }}
          />
        </label>

        {error && (
          <p style={{ color: "#ef4444", fontSize: 13 }}>{error}</p>
        )}

        <button
          type="submit"
          disabled={loading || !task.trim()}
          style={{
            background: loading ? "#3f3f46" : "#6366f1",
            color: "#fff",
            border: "none",
            borderRadius: 6,
            padding: "10px 20px",
            fontSize: 14,
            fontWeight: 600,
            cursor: loading ? "not-allowed" : "pointer",
            alignSelf: "flex-start",
          }}
        >
          {loading ? "Starting..." : "Start Run"}
        </button>
      </form>
    </div>
  );
}
