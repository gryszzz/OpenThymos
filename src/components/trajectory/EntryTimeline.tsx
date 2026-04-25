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
      className="thymos-ledger-kind"
      style={{
        background: bg,
      }}
    >
      {kind}
    </span>
  );
}

export function EntryTimeline({ entries }: { entries: EntryDto[] }) {
  if (entries.length === 0) {
    return (
      <div className="thymos-empty-state">
        <strong>Ledger Stream Waiting</strong>
        <p>Waiting for trajectory entries from the runtime ledger...</p>
      </div>
    );
  }

  return (
    <div className="thymos-ledger-list">
      {entries.map((e) => (
        <div
          key={`${e.seq}-${e.id}`}
          className="thymos-ledger-entry"
        >
          <span className="thymos-ledger-seq">
            #{e.seq}
          </span>
          <KindBadge kind={e.kind} />
          <code className="thymos-ledger-id">
            {e.id.slice(0, 8)}
          </code>
          <span className="thymos-ledger-detail">
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
