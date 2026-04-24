/** Thymos server API client. */

const BASE = process.env.NEXT_PUBLIC_THYMOS_URL ?? "http://localhost:3001";

async function readJson<T>(res: Response, action: string): Promise<T> {
  const text = await res.text();
  let body: unknown = null;
  if (text) {
    try {
      body = JSON.parse(text);
    } catch {
      body = text;
    }
  }

  if (!res.ok) {
    const payload = body && typeof body === "object" ? body as Record<string, unknown> : null;
    const message =
      typeof payload?.error === "string"
        ? payload.error
        : typeof payload?.message === "string"
          ? payload.message
          : text || res.statusText;
    throw new Error(`${action} failed (${res.status}): ${message}`);
  }

  return body as T;
}

export type StreamConnectionState = "connecting" | "live" | "reconnecting";

export interface StreamCallbacks {
  onOpen?: () => void;
  onError?: () => void;
}

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
  failures: number;
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

export interface ExecutionCounters {
  steps_started: number;
  intents_declared: number;
  proposals_staged: number;
  commits: number;
  rejections: number;
  failures: number;
  recoveries: number;
  retries: number;
  approvals_pending: number;
}

export interface ExecutionLogEntry {
  idx: number;
  timestamp_ms: number;
  phase: "system" | "intent" | "proposal" | "execution" | "result";
  level: "info" | "success" | "warning" | "error";
  title: string;
  detail: string;
  step_index?: number;
  tool?: string;
  intent_id?: string;
  proposal_id?: string;
  commit_id?: string;
  seq?: number;
}

export interface ExecutionSession {
  run_id: string;
  task: string;
  trajectory_id?: string;
  status: "running" | "waiting_approval" | "completed" | "failed" | "cancelled";
  phase: "system" | "intent" | "proposal" | "execution" | "result";
  operator_state: string;
  current_step: number;
  max_steps: number;
  active_tool?: string;
  final_answer?: string;
  counters: ExecutionCounters;
  updated_at_ms: number;
  log: ExecutionLogEntry[];
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
  /** Model name override (e.g. "opus", "gpt-4o-mini", "llama3"). */
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
  const safeMaxSteps = Number.isFinite(maxSteps)
    ? Math.min(100, Math.max(1, Math.trunc(maxSteps)))
    : 16;
  const res = await fetch(`${BASE}/runs`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      task,
      max_steps: safeMaxSteps,
      ...(cognition ? { cognition } : {}),
    }),
  });
  return readJson<{ run_id: string; task: string; status: string }>(res, "create run");
}

/** GET /runs/:id — get run status. */
export async function getRun(id: string) {
  const res = await fetch(`${BASE}/runs/${id}`);
  return readJson<RunRecord>(res, "get run");
}

/** GET /runs/:id/execution — get unified execution session state. */
export async function getExecution(id: string) {
  const res = await fetch(`${BASE}/runs/${id}/execution`);
  return readJson<ExecutionSession>(res, "get execution");
}

/** GET /runs/:id/world — get world projection. */
export async function getWorld(id: string) {
  const res = await fetch(`${BASE}/runs/${id}/world`);
  return readJson<WorldDto>(res, "get world");
}

/** GET /runs/:id/world/at?seq=N — replay world projection up to commit seq N. */
export async function getWorldAt(id: string, seq: number) {
  const res = await fetch(`${BASE}/runs/${id}/world/at?seq=${seq}`);
  return readJson<WorldDto & {
    seq: number;
    commits_replayed: number;
    head_commit: string | null;
  }>(res, "get world replay");
}

/** POST /runs/:id/branch — create a shadow (counterfactual) branch from a commit. */
export async function branchFrom(id: string, commitId: string, note?: string) {
  const res = await fetch(`${BASE}/runs/${id}/branch`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ commit_id: commitId, note }),
  });
  return readJson<{
    branch_trajectory_id: string;
    source_trajectory_id: string;
    source_commit_id: string;
    note: string;
  }>(res, "create branch");
}

/** Subscribe to SSE trajectory entry events. */
export function subscribeEntries(
  runId: string,
  onEntry: (entry: EntryDto) => void,
  callbacks?: StreamCallbacks,
): EventSource {
  const es = new EventSource(`${BASE}/runs/${runId}/events`);
  es.onopen = () => callbacks?.onOpen?.();
  es.onmessage = (e) => {
    try {
      onEntry(JSON.parse(e.data));
    } catch { /* skip */ }
  };
  es.onerror = () => callbacks?.onError?.();
  return es;
}

/** Subscribe to SSE cognition streaming events. */
export function subscribeStream(
  runId: string,
  onEvent: (evt: CognitionEvent) => void,
  callbacks?: StreamCallbacks,
): EventSource {
  const es = new EventSource(`${BASE}/runs/${runId}/stream`);
  es.onopen = () => callbacks?.onOpen?.();

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

  es.onerror = () => callbacks?.onError?.();
  return es;
}

/** Subscribe to unified execution session snapshots. */
export function subscribeExecution(
  runId: string,
  onSnapshot: (session: ExecutionSession) => void,
  callbacks?: StreamCallbacks,
): EventSource {
  const es = new EventSource(`${BASE}/runs/${runId}/execution/stream`);
  es.onopen = () => callbacks?.onOpen?.();
  es.addEventListener("snapshot", (e: MessageEvent) => {
    try {
      onSnapshot(JSON.parse(e.data));
    } catch {
      /* skip */
    }
  });
  es.onerror = () => callbacks?.onError?.();
  return es;
}
