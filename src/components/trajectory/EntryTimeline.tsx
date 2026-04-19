"use client";

import type { EntryDto } from "@/lib/thymos-api";

const kindColors: Record<string, string> = {
  root: "#6366f1",
  commit: "#22c55e",
  rejection: "#ef4444",
  pending_approval: "#f59e0b",
  delegation: "#8b5cf6",
  branch: "#06b6d4",
};

function KindBadge({ kind }: { kind: string }) {
  const bg = kindColors[kind] ?? "#71717a";
  return (
    <span
      style={{
        background: bg,
        color: "#fff",
        padding: "2px 8px",
        borderRadius: 4,
        fontSize: 12,
        fontWeight: 600,
        textTransform: "uppercase",
        letterSpacing: "0.05em",
      }}
    >
      {kind}
    </span>
  );
}

export function EntryTimeline({ entries }: { entries: EntryDto[] }) {
  if (entries.length === 0) {
    return (
      <p style={{ color: "#a1a1aa", fontStyle: "italic" }}>
        Waiting for events...
      </p>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      {entries.map((e) => (
        <div
          key={`${e.seq}-${e.id}`}
          style={{
            display: "flex",
            gap: 12,
            alignItems: "flex-start",
            padding: "8px 12px",
            background: "#18181b",
            borderRadius: 6,
            border: "1px solid #27272a",
            fontFamily: "var(--font-body), monospace",
            fontSize: 13,
          }}
        >
          <span style={{ color: "#71717a", minWidth: 30, textAlign: "right" }}>
            #{e.seq}
          </span>
          <KindBadge kind={e.kind} />
          <code style={{ color: "#a1a1aa", fontSize: 11 }}>
            {e.id.slice(0, 8)}
          </code>
          <span style={{ color: "#d4d4d8", flex: 1 }}>
            {formatDetail(e)}
          </span>
        </div>
      ))}
    </div>
  );
}

function formatDetail(e: EntryDto): string {
  const d = e.detail;
  switch (e.kind) {
    case "root":
      return String(d.note ?? "");
    case "commit":
      return `${d.delta_ops} ops, ${d.observations} obs`;
    case "rejection":
      return String(d.reason ?? "");
    case "pending_approval":
      return `[${d.channel}] ${d.reason}`;
    case "delegation":
      return `task: ${d.task}`;
    case "branch":
      return String(d.note ?? "");
    default:
      return JSON.stringify(d);
  }
}
