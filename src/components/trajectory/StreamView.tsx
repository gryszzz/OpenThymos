"use client";

import type { CognitionEvent } from "@/lib/thymos-api";

export function StreamView({ events }: { events: CognitionEvent[] }) {
  if (events.length === 0) {
    return (
      <p style={{ color: "#a1a1aa", fontStyle: "italic" }}>
        Waiting for cognition stream...
      </p>
    );
  }

  // Reconstruct the streaming text from token events.
  let text = "";
  const toolUses: { tool: string; id: string }[] = [];

  for (const evt of events) {
    switch (evt.type) {
      case "token":
        text += evt.text ?? "";
        break;
      case "tool_use_start":
        toolUses.push({ tool: evt.tool ?? "?", id: evt.id ?? "" });
        break;
      case "turn_complete":
        if (evt.final_answer) {
          text += `\n\n--- Final Answer ---\n${evt.final_answer}`;
        }
        break;
      case "error":
        text += `\n[ERROR] ${evt.message}`;
        break;
    }
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {text && (
        <pre
          style={{
            background: "#18181b",
            border: "1px solid #27272a",
            borderRadius: 6,
            padding: 12,
            color: "#d4d4d8",
            fontSize: 13,
            whiteSpace: "pre-wrap",
            wordBreak: "break-word",
            maxHeight: 400,
            overflow: "auto",
          }}
        >
          {text}
        </pre>
      )}
      {toolUses.length > 0 && (
        <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
          {toolUses.map((t) => (
            <span
              key={t.id}
              style={{
                background: "#7c3aed",
                color: "#fff",
                padding: "2px 8px",
                borderRadius: 4,
                fontSize: 11,
                fontWeight: 600,
              }}
            >
              {t.tool}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
