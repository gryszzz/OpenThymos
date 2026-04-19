// Thymos VSCode extension — minimal bridge between VSCode and the Thymos
// governed-cognition runtime. Uses the CLI for interactive work (opens a
// terminal running `thymos shell`) and talks to the HTTP API directly for
// programmatic flows (run + poll + approve with a native diff viewer).

import * as vscode from "vscode";

type Config = {
  url: string;
  apiKey: string;
  cliPath: string;
  provider: string;
  preset: "default" | "code";
};

function getConfig(): Config {
  const c = vscode.workspace.getConfiguration("thymos");
  return {
    url: c.get<string>("url", "http://localhost:3001"),
    apiKey: c.get<string>("apiKey", ""),
    cliPath: c.get<string>("cliPath", "thymos"),
    provider: c.get<string>("provider", "anthropic"),
    preset: c.get<"default" | "code">("preset", "code"),
  };
}

function authHeaders(cfg: Config): Record<string, string> {
  return cfg.apiKey ? { Authorization: `Bearer ${cfg.apiKey}` } : {};
}

function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

async function httpJson(
  url: string,
  opts: RequestInit = {},
): Promise<{ status: number; body: any }> {
  const resp = await fetch(url, opts);
  const text = await resp.text();
  let body: any;
  try {
    body = JSON.parse(text);
  } catch {
    body = text;
  }
  return { status: resp.status, body };
}

// ────────────────────────────────────────────────────────────────────────────
// Status bar: live health indicator.
// ────────────────────────────────────────────────────────────────────────────

let statusItem: vscode.StatusBarItem;

async function refreshHealth(): Promise<void> {
  const cfg = getConfig();
  try {
    const { status } = await httpJson(`${cfg.url}/health`, {
      headers: authHeaders(cfg),
    });
    if (status === 200) {
      statusItem.text = "$(pulse) Thymos";
      statusItem.tooltip = `Thymos server OK — ${cfg.url}`;
      statusItem.backgroundColor = undefined;
    } else {
      statusItem.text = "$(warning) Thymos";
      statusItem.tooltip = `Thymos ${status} — ${cfg.url}`;
      statusItem.backgroundColor = new vscode.ThemeColor(
        "statusBarItem.warningBackground",
      );
    }
  } catch (e) {
    statusItem.text = "$(debug-disconnect) Thymos";
    statusItem.tooltip = `Thymos unreachable — ${cfg.url}`;
    statusItem.backgroundColor = new vscode.ThemeColor(
      "statusBarItem.errorBackground",
    );
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Commands.
// ────────────────────────────────────────────────────────────────────────────

function cmdOpenShell() {
  const cfg = getConfig();
  const cwd = workspaceRoot();
  const terminal = vscode.window.createTerminal({
    name: "Thymos Shell",
    cwd,
    env: {
      THYMOS_URL: cfg.url,
      ...(cfg.apiKey ? { THYMOS_API_KEY: cfg.apiKey } : {}),
    },
  });
  terminal.show();
  // Pre-configure the shell for coding work against this workspace.
  terminal.sendText(`${cfg.cliPath} shell`);
  if (cwd) {
    terminal.sendText(`set preset ${cfg.preset}`);
    terminal.sendText(`set workspace ${cwd}`);
  }
}

async function cmdHealth() {
  const cfg = getConfig();
  try {
    const { status, body } = await httpJson(`${cfg.url}/health`, {
      headers: authHeaders(cfg),
    });
    vscode.window.showInformationMessage(
      `Thymos ${status} — ${JSON.stringify(body)}`,
    );
  } catch (e: any) {
    vscode.window.showErrorMessage(
      `Thymos unreachable at ${cfg.url}: ${e.message ?? e}`,
    );
  } finally {
    await refreshHealth();
  }
}

async function cmdRunTask(output: vscode.OutputChannel) {
  const cfg = getConfig();
  const cwd = workspaceRoot();
  if (!cwd) {
    vscode.window.showErrorMessage("Open a folder before running a Thymos task.");
    return;
  }

  const task = await vscode.window.showInputBox({
    prompt: "What should Thymos do?",
    placeHolder: "e.g. add a README section describing the shell",
    ignoreFocusOut: true,
  });
  if (!task) return;

  output.show(true);
  output.appendLine(`[thymos] submitting: ${task}`);

  const { status, body } = await httpJson(`${cfg.url}/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders(cfg) },
    body: JSON.stringify({
      task,
      max_steps: cfg.preset === "code" ? 64 : 16,
      cognition: { provider: cfg.provider },
      ...(cfg.preset === "code"
        ? {
            tool_scopes: [
              "fs_read",
              "fs_patch",
              "list_files",
              "repo_map",
              "grep",
              "test_run",
              "memory_store",
              "memory_recall",
            ],
          }
        : {}),
    }),
  });

  if (status >= 400 || !body?.run_id) {
    output.appendLine(`[thymos] error ${status}: ${JSON.stringify(body)}`);
    vscode.window.showErrorMessage(`Thymos run failed: ${status}`);
    return;
  }

  const runId: string = body.run_id;
  output.appendLine(`[thymos] run: ${runId}`);

  await pollRunToTerminal(cfg, runId, cwd, output);
}

async function pollRunToTerminal(
  cfg: Config,
  runId: string,
  cwd: string,
  output: vscode.OutputChannel,
): Promise<void> {
  const handled = new Set<number>();
  let lastStatus: string | null = null;

  while (true) {
    const status = await httpJson(`${cfg.url}/runs/${runId}`, {
      headers: authHeaders(cfg),
    });
    const st: string = status.body?.status ?? "?";
    if (st !== lastStatus) {
      output.appendLine(`[thymos] status: ${st}`);
      lastStatus = st;
    }
    if (st === "completed" || st === "failed" || st === "cancelled") {
      const final = status.body?.summary?.final_answer;
      if (final) {
        output.appendLine(`\n── final answer ──\n${final}\n`);
      }
      return;
    }

    const entries = await httpJson(
      `${cfg.url}/audit/entries?run_id=${runId}&kind=pending_approval&limit=200`,
      { headers: authHeaders(cfg) },
    );
    const pending: any[] = entries.body?.entries ?? [];
    for (const entry of pending) {
      if (handled.has(entry.seq)) continue;
      const approved = await reviewPendingApproval(cfg, cwd, entry, output);
      const channel: string = entry.detail?.channel ?? "";
      if (!channel) continue;
      await httpJson(`${cfg.url}/runs/${runId}/approvals/${channel}`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...authHeaders(cfg) },
        body: JSON.stringify({ approve: approved }),
      });
      output.appendLine(
        `[thymos] ${approved ? "approved" : "denied"} channel=${channel}`,
      );
      handled.add(entry.seq);
    }

    await new Promise((r) => setTimeout(r, 500));
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Approval review — if the proposal is an fs_patch, show a native VSCode
// diff editor between the on-disk file and the proposed content, and use
// the modal dialog for the decision.
// ────────────────────────────────────────────────────────────────────────────

async function reviewPendingApproval(
  cfg: Config,
  cwd: string,
  entry: any,
  output: vscode.OutputChannel,
): Promise<boolean> {
  const channel: string = entry.detail?.channel ?? "?";
  const reason: string = entry.detail?.reason ?? "(no reason)";
  const proposal = entry.detail?.proposal;
  const tool: string | undefined = proposal?.body?.plan?.tool;
  const args = proposal?.body?.plan?.args ?? {};

  output.appendLine(
    `[thymos] approval requested — channel=${channel} tool=${tool} reason=${reason}`,
  );

  if (tool === "fs_patch") {
    const ok = await showFsPatchDiff(cwd, args);
    if (ok !== undefined) return ok;
  }

  const choice = await vscode.window.showInformationMessage(
    `Thymos approval — ${tool ?? "tool"}: ${reason}`,
    { modal: true, detail: JSON.stringify(args, null, 2) },
    "Approve",
    "Deny",
  );
  return choice === "Approve";
}

async function showFsPatchDiff(
  cwd: string,
  args: any,
): Promise<boolean | undefined> {
  const path: string | undefined = args.path;
  const mode: string | undefined = args.mode;
  if (!path || !mode) return undefined;

  const absolute = path.startsWith("/") ? path : `${cwd}/${path}`;
  const fileUri = vscode.Uri.file(absolute);

  let proposed: string;
  if (mode === "write") {
    proposed = args.content ?? "";
  } else if (mode === "replace") {
    let current = "";
    try {
      const buf = await vscode.workspace.fs.readFile(fileUri);
      current = Buffer.from(buf).toString("utf8");
    } catch {
      current = "";
    }
    const anchor: string = args.anchor ?? "";
    const replacement: string = args.replacement ?? "";
    const occurrences = anchor ? current.split(anchor).length - 1 : 0;
    if (occurrences === 1) {
      proposed = current.replace(anchor, replacement);
    } else {
      proposed = replacement;
    }
  } else {
    return undefined;
  }

  const doc = await vscode.workspace.openTextDocument({
    content: proposed,
    language: languageFor(path),
  });
  await vscode.commands.executeCommand(
    "vscode.diff",
    fileUri,
    doc.uri,
    `Thymos proposal: ${path}`,
    { preview: true },
  );

  const choice = await vscode.window.showInformationMessage(
    `Apply ${mode} to ${path}?`,
    { modal: true },
    "Approve",
    "Deny",
  );
  return choice === "Approve";
}

function languageFor(path: string): string {
  if (path.endsWith(".rs")) return "rust";
  if (path.endsWith(".ts") || path.endsWith(".tsx")) return "typescript";
  if (path.endsWith(".js") || path.endsWith(".jsx")) return "javascript";
  if (path.endsWith(".py")) return "python";
  if (path.endsWith(".go")) return "go";
  if (path.endsWith(".md")) return "markdown";
  if (path.endsWith(".json")) return "json";
  if (path.endsWith(".toml")) return "toml";
  return "plaintext";
}

async function cmdReviewPending() {
  const cfg = getConfig();
  const runId = await vscode.window.showInputBox({
    prompt: "Run ID to review",
    ignoreFocusOut: true,
  });
  if (!runId) return;
  const entries = await httpJson(
    `${cfg.url}/audit/entries?run_id=${runId}&kind=pending_approval&limit=10`,
    { headers: authHeaders(cfg) },
  );
  const pending: any[] = entries.body?.entries ?? [];
  if (pending.length === 0) {
    vscode.window.showInformationMessage("No pending approvals for that run.");
    return;
  }
  const output = vscode.window.createOutputChannel("Thymos");
  const cwd = workspaceRoot() ?? "";
  for (const entry of pending) {
    const approved = await reviewPendingApproval(cfg, cwd, entry, output);
    const channel: string = entry.detail?.channel ?? "";
    if (!channel) continue;
    await httpJson(`${cfg.url}/runs/${runId}/approvals/${channel}`, {
      method: "POST",
      headers: { "Content-Type": "application/json", ...authHeaders(cfg) },
      body: JSON.stringify({ approve: approved }),
    });
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Activation.
// ────────────────────────────────────────────────────────────────────────────

export function activate(context: vscode.ExtensionContext) {
  const output = vscode.window.createOutputChannel("Thymos");

  statusItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100,
  );
  statusItem.command = "thymos.health";
  statusItem.text = "$(pulse) Thymos";
  statusItem.show();
  context.subscriptions.push(statusItem);

  context.subscriptions.push(
    vscode.commands.registerCommand("thymos.openShell", cmdOpenShell),
    vscode.commands.registerCommand("thymos.health", cmdHealth),
    vscode.commands.registerCommand("thymos.runTask", () => cmdRunTask(output)),
    vscode.commands.registerCommand("thymos.reviewPending", cmdReviewPending),
  );

  refreshHealth();
  const interval = setInterval(refreshHealth, 15_000);
  context.subscriptions.push({ dispose: () => clearInterval(interval) });
}

export function deactivate() {}
