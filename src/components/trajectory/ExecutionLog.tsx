"use client";

import type { ExecutionLogEntry, ExecutionSession } from "@/lib/thymos-api";

const levelStyles: Record<ExecutionLogEntry["level"], { border: string; badge: string; text: string }> = {
  info: {
    border: "rgba(119, 169, 255, 0.32)",
    badge: "rgba(119, 169, 255, 0.18)",
    text: "#d8e8ff",
  },
  success: {
    border: "rgba(52, 211, 153, 0.34)",
    badge: "rgba(52, 211, 153, 0.16)",
    text: "#d5ffe9",
  },
  warning: {
    border: "rgba(245, 158, 11, 0.34)",
    badge: "rgba(245, 158, 11, 0.16)",
    text: "#ffe7bb",
  },
  error: {
    border: "rgba(248, 113, 113, 0.34)",
    badge: "rgba(248, 113, 113, 0.16)",
    text: "#ffd8d8",
  },
};

function phaseLabel(phase: ExecutionLogEntry["phase"]) {
  return phase.replace("_", " ");
}

function formatTime(timestampMs: number) {
  return new Date(timestampMs).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function ExecutionLog({ session }: { session: ExecutionSession | null }) {
  if (!session) {
    return <p style={{ color: "rgba(201, 212, 225, 0.72)" }}>Connecting to Thymos runtime...</p>;
  }

  if (session.log.length === 0) {
    return <p style={{ color: "rgba(201, 212, 225, 0.72)" }}>Execution log is waiting for the first runtime event.</p>;
  }

  return (
    <div style={{ display: "grid", gap: 12 }}>
      {session.log.map((entry) => {
        const style = levelStyles[entry.level];
        return (
          <article
            key={entry.idx}
            style={{
              border: `1px solid ${style.border}`,
              borderRadius: 16,
              padding: "14px 16px",
              background:
                "linear-gradient(180deg, rgba(255,255,255,0.04), transparent 16%), rgba(10,14,21,0.88)",
              boxShadow: "0 18px 44px rgba(0,0,0,0.22)",
            }}
          >
            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: 10,
                alignItems: "center",
                marginBottom: 10,
              }}
            >
              <span
                style={{
                  padding: "4px 9px",
                  borderRadius: 999,
                  background: style.badge,
                  color: style.text,
                  fontSize: 11,
                  letterSpacing: "0.12em",
                  textTransform: "uppercase",
                  fontWeight: 700,
                }}
              >
                {phaseLabel(entry.phase)}
              </span>
              <strong style={{ color: "#eef4fb", fontSize: 14 }}>{entry.title}</strong>
              <span style={{ color: "rgba(150, 163, 181, 0.88)", fontSize: 12, marginLeft: "auto" }}>
                {formatTime(entry.timestamp_ms)}
              </span>
            </div>

            <p
              style={{
                margin: 0,
                color: "rgba(223, 232, 243, 0.84)",
                lineHeight: 1.6,
                fontSize: 13,
                whiteSpace: "pre-wrap",
              }}
            >
              {entry.detail}
            </p>

            <div style={{ display: "flex", flexWrap: "wrap", gap: 8, marginTop: 12 }}>
              {entry.step_index !== undefined && (
                <LogChip label={`step ${entry.step_index + 1}`} />
              )}
              {entry.tool && <LogChip label={entry.tool} />}
              {entry.seq !== undefined && <LogChip label={`seq ${entry.seq}`} />}
              {entry.commit_id && <LogChip label={`commit ${entry.commit_id.slice(0, 10)}`} />}
            </div>
          </article>
        );
      })}
    </div>
  );
}

function LogChip({ label }: { label: string }) {
  return (
    <span
      style={{
        borderRadius: 999,
        border: "1px solid rgba(146, 163, 184, 0.18)",
        padding: "4px 8px",
        color: "rgba(201, 212, 225, 0.82)",
        fontSize: 11,
        background: "rgba(255,255,255,0.03)",
      }}
    >
      {label}
    </span>
  );
}
