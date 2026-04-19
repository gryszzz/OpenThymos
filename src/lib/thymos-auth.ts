/**
 * Auth-to-Writ bridge.
 *
 * Maps an authenticated user session to a Thymos run with a tenant-scoped
 * Writ. The server mints the Writ; the client just passes identity claims.
 *
 * Works with any auth provider — just provide user ID and optional metadata.
 */

const BASE = process.env.NEXT_PUBLIC_THYMOS_URL ?? "http://localhost:3001";

export interface AuthContext {
  /** Unique user identifier from your auth provider. */
  userId: string;
  /** Tenant/org ID for multi-tenant isolation. */
  tenantId: string;
  /** Display name (optional, for audit trails). */
  name?: string;
  /** Email (optional, for audit trails). */
  email?: string;
  /** Roles or permissions from your auth provider. */
  roles?: string[];
}

export interface WritConfig {
  /** Tool scopes the user is allowed (default: ["*"]). */
  toolScopes?: string[];
  /** Max steps for the run (default: 16). */
  maxSteps?: number;
  /** Budget overrides. */
  budget?: {
    tokens?: number;
    toolCalls?: number;
    wallClockMs?: number;
    usdMillicents?: number;
  };
}

/** Map roles to tool scopes. Customize this for your access control model. */
export function rolesToScopes(roles: string[]): string[] {
  const scopes: string[] = [];
  for (const role of roles) {
    switch (role) {
      case "admin":
        scopes.push("*");
        break;
      case "editor":
        scopes.push("kv_*", "memory_*", "http");
        break;
      case "viewer":
        scopes.push("kv_get", "memory_recall");
        break;
      default:
        scopes.push("kv_get", "memory_recall");
    }
  }
  return [...new Set(scopes)];
}

/** Create a tenant-scoped run from an auth context. */
export async function createAuthenticatedRun(
  auth: AuthContext,
  task: string,
  config?: WritConfig,
) {
  const scopes = config?.toolScopes
    ?? (auth.roles ? rolesToScopes(auth.roles) : ["*"]);

  const res = await fetch(`${BASE}/runs`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      // Pass auth context as headers for the server to embed in the Writ.
      "x-thymos-user-id": auth.userId,
      "x-thymos-tenant-id": auth.tenantId,
      ...(auth.name ? { "x-thymos-user-name": auth.name } : {}),
      ...(auth.email ? { "x-thymos-user-email": auth.email } : {}),
    },
    body: JSON.stringify({
      task,
      max_steps: config?.maxSteps ?? 16,
      tool_scopes: scopes,
    }),
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Thymos run creation failed (${res.status}): ${body}`);
  }

  return res.json() as Promise<{
    run_id: string;
    task: string;
    status: string;
  }>;
}
