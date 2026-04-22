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
    return (
      <div className="thymos-empty-state">
        <strong>Runtime Linking</strong>
        <p>Connecting to Thymos runtime...</p>
      </div>
    );
  }

  if (session.log.length === 0) {
    return (
      <div className="thymos-empty-state">
        <strong>Execution Log Waiting</strong>
        <p>Execution log is waiting for the first runtime event.</p>
      </div>
    );
  }

  return (
    <div className="thymos-log-list">
      {session.log.map((entry) => {
        const style = levelStyles[entry.level];
        return (
          <article
            key={entry.idx}
            className="thymos-log-entry"
            data-level={entry.level}
          >
            <div className="thymos-log-head">
              <span
                className="thymos-chip thymos-chip--phase"
                style={{ borderColor: style.border, background: style.badge, color: style.text }}
              >
                {phaseLabel(entry.phase)}
              </span>
              <strong className="thymos-log-title">{entry.title}</strong>
              <span className="thymos-log-meta">{formatTime(entry.timestamp_ms)}</span>
            </div>

            <p className="thymos-log-copy">{entry.detail}</p>

            <div className="thymos-chip-row" style={{ marginTop: 12 }}>
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
  return <span className="thymos-chip">{label}</span>;
}
