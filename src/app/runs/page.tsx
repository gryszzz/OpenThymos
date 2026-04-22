"use client";

import { useEffect, useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import { RunViewer } from "@/components/trajectory/RunViewer";
import { createRun } from "@/lib/thymos-api";

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
    <main
      style={{
        width: "min(1180px, calc(100% - 32px))",
        margin: "0 auto",
        padding: "34px 0 62px",
        color: "#eef4fb",
      }}
    >
      <section
        style={{
          position: "relative",
          overflow: "hidden",
          display: "grid",
          gridTemplateColumns: "minmax(0, 1.08fr) minmax(320px, 0.92fr)",
          gap: 20,
          borderRadius: 30,
          border: "1px solid rgba(146, 163, 184, 0.16)",
          background:
            "linear-gradient(180deg, rgba(255,255,255,0.05), transparent 18%), rgba(8, 12, 18, 0.92)",
          boxShadow: "0 28px 90px rgba(0,0,0,0.36)",
          padding: 24,
        }}
      >
        <div
          aria-hidden="true"
          style={{
            position: "absolute",
            right: -120,
            top: -90,
            width: 360,
            height: 360,
            borderRadius: "50%",
            background: "radial-gradient(circle, rgba(119, 169, 255, 0.2), transparent 70%)",
            filter: "blur(18px)",
          }}
        />

        <div style={{ position: "relative", display: "grid", gap: 18 }}>
          <div style={{ color: "rgba(119, 169, 255, 0.9)", fontSize: 12, letterSpacing: "0.16em", textTransform: "uppercase", fontWeight: 700 }}>
            Thymos Unified Runtime
          </div>
          <h1
            style={{
              margin: 0,
              fontSize: "clamp(2rem, 4vw, 3.35rem)",
              lineHeight: 1.03,
              fontFamily: "var(--font-display), sans-serif",
              maxWidth: 720,
            }}
          >
            Start one task. Watch the same execution session everywhere.
          </h1>
          <p style={{ margin: 0, color: "rgba(223, 232, 243, 0.78)", lineHeight: 1.75, maxWidth: 760 }}>
            Thymos runs the full operator loop from intent to result against a shared backend runtime.
            CLI, sidebar, and web console all attach to the same live execution state, log, approvals,
            and completion record.
          </p>

          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: 12 }}>
            {[
              "Intent → Proposal → Execution → Result",
              "Shared live execution log",
              "Runtime-led recovery and retries",
              "Replayable world state and branching",
            ].map((item) => (
              <div
                key={item}
                style={{
                  borderRadius: 18,
                  border: "1px solid rgba(146, 163, 184, 0.14)",
                  background: "rgba(255,255,255,0.03)",
                  padding: "14px 16px",
                  color: "#d9e6f5",
                  lineHeight: 1.55,
                }}
              >
                {item}
              </div>
            ))}
          </div>
        </div>

        <form
          onSubmit={handleSubmit}
          style={{
            position: "relative",
            zIndex: 1,
            display: "grid",
            gap: 16,
            borderRadius: 24,
            border: "1px solid rgba(146, 163, 184, 0.16)",
            background: "rgba(6, 10, 15, 0.9)",
            padding: 20,
          }}
        >
          <div>
            <label
              htmlFor="task"
              style={{ display: "block", marginBottom: 8, color: "#c9d4e1", fontSize: 12, letterSpacing: "0.14em", textTransform: "uppercase" }}
            >
              Operator Task
            </label>
            <textarea
              id="task"
              value={task}
              onChange={(event) => setTask(event.target.value)}
              placeholder="Refactor the TypeScript API client, run the relevant checks, and leave the runtime in a verified state."
              rows={7}
              style={{
                width: "100%",
                resize: "vertical",
                borderRadius: 18,
                border: "1px solid rgba(146, 163, 184, 0.18)",
                background: "rgba(255,255,255,0.04)",
                padding: "16px 18px",
                color: "#eef4fb",
                fontSize: 15,
                lineHeight: 1.65,
              }}
            />
          </div>

          <div style={{ display: "flex", flexWrap: "wrap", gap: 14, alignItems: "center" }}>
            <label style={{ display: "grid", gap: 8 }}>
              <span style={{ color: "#c9d4e1", fontSize: 12, letterSpacing: "0.14em", textTransform: "uppercase" }}>
                Max Steps
              </span>
              <input
                type="number"
                value={maxSteps}
                min={1}
                max={100}
                onChange={(event) => setMaxSteps(Number(event.target.value))}
                style={{
                  width: 120,
                  borderRadius: 999,
                  border: "1px solid rgba(146, 163, 184, 0.18)",
                  background: "rgba(255,255,255,0.04)",
                  padding: "10px 14px",
                  color: "#eef4fb",
                  fontSize: 15,
                }}
              />
            </label>

            <button
              type="submit"
              disabled={loading || !task.trim()}
              style={{
                alignSelf: "end",
                borderRadius: 999,
                border: "1px solid rgba(119, 169, 255, 0.28)",
                background: loading
                  ? "rgba(148, 163, 184, 0.18)"
                  : "linear-gradient(180deg, rgba(133, 175, 255, 0.28), rgba(119, 169, 255, 0.1))",
                boxShadow: loading ? "none" : "0 16px 40px rgba(64, 105, 167, 0.26)",
                color: "#eef4fb",
                padding: "12px 18px",
                minWidth: 180,
                cursor: loading ? "not-allowed" : "pointer",
                fontSize: 12,
                fontWeight: 700,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
              }}
            >
              {loading ? "Launching Runtime" : "Start Thymos Run"}
            </button>
          </div>

          {error && (
            <p style={{ margin: 0, color: "#fca5a5", lineHeight: 1.6 }}>
              {error}
            </p>
          )}
        </form>
      </section>
    </main>
  );
}
