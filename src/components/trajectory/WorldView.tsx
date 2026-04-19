"use client";

import type { ResourceDto } from "@/lib/thymos-api";

export function WorldView({ resources }: { resources: ResourceDto[] }) {
  if (resources.length === 0) {
    return (
      <p style={{ color: "#a1a1aa", fontStyle: "italic" }}>
        No resources yet.
      </p>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      {resources.map((r) => (
        <div
          key={`${r.kind}:${r.id}`}
          style={{
            background: "#18181b",
            border: "1px solid #27272a",
            borderRadius: 6,
            padding: 12,
            fontSize: 13,
          }}
        >
          <div style={{ display: "flex", gap: 8, marginBottom: 6 }}>
            <span
              style={{
                background: "#3b82f6",
                color: "#fff",
                padding: "1px 6px",
                borderRadius: 3,
                fontSize: 11,
                fontWeight: 600,
              }}
            >
              {r.kind}
            </span>
            <span style={{ color: "#d4d4d8", fontWeight: 600 }}>{r.id}</span>
            <span style={{ color: "#71717a", fontSize: 11 }}>v{r.version}</span>
          </div>
          <pre
            style={{
              color: "#a1a1aa",
              fontSize: 12,
              margin: 0,
              whiteSpace: "pre-wrap",
              wordBreak: "break-word",
            }}
          >
            {JSON.stringify(r.value, null, 2)}
          </pre>
        </div>
      ))}
    </div>
  );
}
