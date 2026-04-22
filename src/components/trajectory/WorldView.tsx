"use client";

import type { ResourceDto } from "@/lib/thymos-api";

export function WorldView({ resources }: { resources: ResourceDto[] }) {
  if (resources.length === 0) {
    return (
      <div className="thymos-empty-state">
        <strong>No World Resources Yet</strong>
        <p>The world projection will appear here once the runtime records durable state.</p>
      </div>
    );
  }

  return (
    <div className="thymos-resource-list">
      {resources.map((r) => (
        <div key={`${r.kind}:${r.id}`} className="thymos-resource-card">
          <div className="thymos-resource-head">
            <span className="thymos-chip">{r.kind}</span>
            <strong>{r.id}</strong>
            <span className="thymos-resource-version">v{r.version}</span>
          </div>
          <pre>{JSON.stringify(r.value, null, 2)}</pre>
        </div>
      ))}
    </div>
  );
}
