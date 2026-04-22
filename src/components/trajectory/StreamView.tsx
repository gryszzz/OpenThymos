"use client";

import type { CognitionEvent } from "@/lib/thymos-api";

export function StreamView({ events }: { events: CognitionEvent[] }) {
  if (events.length === 0) {
    return (
      <div className="thymos-empty-state">
        <strong>Model Stream Waiting</strong>
        <p>Waiting for cognition stream...</p>
      </div>
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
    <div className="thymos-resource-list">
      {text && (
        <div className="thymos-stream-block">
          <pre>{text}</pre>
        </div>
      )}
      {toolUses.length > 0 && (
        <div className="thymos-chip-row">
          {toolUses.map((t) => (
            <span key={t.id} className="thymos-chip">
              {t.tool}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
