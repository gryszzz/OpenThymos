/** Thymos server API client. */

const BASE = process.env.NEXT_PUBLIC_THYMOS_URL ?? "http://localhost:3001";

export interface RunRecord {
  trajectory_id: string;
  task: string;
  status: "running" | "completed" | "failed";
  summary?: RunSummary;
}

export interface RunSummary {
  steps_executed: number;
  intents_submitted: number;
  commits: number;
  rejections: number;
  final_answer?: string;
  terminated_by: string;
}

export interface EntryDto {
  seq: number;
  kind: string;
  id: string;
  detail: Record<string, unknown>;
  /** Full 64-hex commit id (commits only). */
  commit_id?: string;
}

export interface WorldDto {
  resources: ResourceDto[];
}

export interface ResourceDto {
  kind: string;
  id: string;
  version: number;
  value: unknown;
}

export interface CognitionEvent {
  type: "token" | "tool_use_start" | "tool_use_arg_delta" | "tool_use_done" | "turn_complete" | "error";
  text?: string;
  tool?: string;
  id?: string;
  delta?: string;
  intents_count?: number;
  final_answer?: string;
  message?: string;
}

/** Cognition provider selection for multi-model support. */
export type CognitionProvider =
  | "anthropic"
  | "openai"
  | "local"
  | "lmstudio"
  | "huggingface"
  | "mock";

export interface CognitionConfig {
  provider: CognitionProvider;
  /** Model name override (e.g. "claude-sonnet-4-5-20250514", "gpt-4o-mini", "llama3"). */
  model?: string;
  /** Max tokens for the response. */
  max_tokens?: number;
  /** Base URL for local/custom endpoints. */
  base_url?: string;
}

/** POST /runs — start a new agent run. */
export async function createRun(
  task: string,
  maxSteps = 16,
  cognition?: CognitionConfig,
) {
  const res = await fetch(`${BASE}/runs`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      task,
      max_steps: maxSteps,
      ...(cognition ? { cognition } : {}),
    }),
  });
  return res.json() as Promise<{ run_id: string; task: string; status: string }>;
}

/** GET /runs/:id — get run status. */
export async function getRun(id: string) {
  const res = await fetch(`${BASE}/runs/${id}`);
  return res.json() as Promise<RunRecord>;
}

/** GET /runs/:id/world — get world projection. */
export async function getWorld(id: string) {
  const res = await fetch(`${BASE}/runs/${id}/world`);
  return res.json() as Promise<WorldDto>;
}

/** GET /runs/:id/world/at?seq=N — replay world projection up to commit seq N. */
export async function getWorldAt(id: string, seq: number) {
  const res = await fetch(`${BASE}/runs/${id}/world/at?seq=${seq}`);
  return res.json() as Promise<WorldDto & {
    seq: number;
    commits_replayed: number;
    head_commit: string | null;
  }>;
}

/** POST /runs/:id/branch — create a shadow (counterfactual) branch from a commit. */
export async function branchFrom(id: string, commitId: string, note?: string) {
  const res = await fetch(`${BASE}/runs/${id}/branch`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ commit_id: commitId, note }),
  });
  if (!res.ok) throw new Error(`branch failed: ${res.status}`);
  return res.json() as Promise<{
    branch_trajectory_id: string;
    source_trajectory_id: string;
    source_commit_id: string;
    note: string;
  }>;
}

/** Subscribe to SSE trajectory entry events. */
export function subscribeEntries(
  runId: string,
  onEntry: (entry: EntryDto) => void,
  onDone?: () => void,
): EventSource {
  const es = new EventSource(`${BASE}/runs/${runId}/events`);
  es.onmessage = (e) => {
    try {
      onEntry(JSON.parse(e.data));
    } catch { /* skip */ }
  };
  es.onerror = () => {
    es.close();
    onDone?.();
  };
  return es;
}

/** Subscribe to SSE cognition streaming events. */
export function subscribeStream(
  runId: string,
  onEvent: (evt: CognitionEvent) => void,
  onDone?: () => void,
): EventSource {
  const es = new EventSource(`${BASE}/runs/${runId}/stream`);

  for (const eventType of [
    "token",
    "tool_use_start",
    "tool_use_arg_delta",
    "tool_use_done",
    "turn_complete",
    "error",
  ]) {
    es.addEventListener(eventType, (e: MessageEvent) => {
      try {
        onEvent(JSON.parse(e.data));
      } catch { /* skip */ }
    });
  }

  es.onerror = () => {
    es.close();
    onDone?.();
  };
  return es;
}
