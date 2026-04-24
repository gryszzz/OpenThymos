"use client";

import { useMemo } from "react";
import type { CognitionEvent } from "@/lib/thymos-api";

export function StreamView({ events }: { events: CognitionEvent[] }) {
  const { text, toolUses } = useMemo(() => {
    let nextText = "";
    const nextToolUses: { tool: string; id: string }[] = [];

    for (const evt of events) {
      switch (evt.type) {
        case "token":
          nextText += evt.text ?? "";
          break;
        case "tool_use_start":
          nextToolUses.push({ tool: evt.tool ?? "?", id: evt.id ?? "" });
          break;
        case "turn_complete":
          if (evt.final_answer) {
            nextText += `\n\n--- Final Answer ---\n${evt.final_answer}`;
          }
          break;
        case "error":
          nextText += `\n[ERROR] ${evt.message}`;
          break;
      }
    }

    return { text: nextText, toolUses: nextToolUses };
  }, [events]);

  if (events.length === 0) {
    return (
      <div className="thymos-empty-state">
        <strong>Model Stream Waiting</strong>
        <p>Waiting for cognition stream...</p>
      </div>
    );
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
