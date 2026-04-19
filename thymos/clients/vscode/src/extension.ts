// Thymos VSCode extension — bridge between VSCode and the Thymos
// governed-cognition runtime. Owns a docked sidebar view with inline
// Approve/Deny + Open Diff buttons.

import * as vscode from "vscode";

type Config = {
  url: string;
  apiKey: string;
  cliPath: string;
  provider: string;
  preset: "default" | "code";
  openPanelOnStartup: boolean;
};

function getConfig(): Config {
  const c = vscode.workspace.getConfiguration("thymos");
  return {
    url: c.get<string>("url", "http://localhost:3001"),
    apiKey: c.get<string>("apiKey", ""),
    cliPath: c.get<string>("cliPath", "thymos"),
    provider: c.get<string>("provider", "anthropic"),
    preset: c.get<"default" | "code">("preset", "code"),
    openPanelOnStartup: c.get<boolean>("openPanelOnStartup", true),
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
// Docked sidebar view — persistent Thymos console with inline approvals.
// ────────────────────────────────────────────────────────────────────────────

class ThymosSidebar implements vscode.WebviewViewProvider {
  static readonly viewType = "thymos.sidebar";
  private static current: ThymosSidebar | undefined;
  private view: vscode.WebviewView | undefined;
  private pendingDecisions = new Map<string, (approved: boolean) => void>();
  private queuedMessages: Array<{ type: string; [key: string]: any }> = [];
  private disposables: vscode.Disposable[] = [];

  constructor(_context: vscode.ExtensionContext) {
    ThymosSidebar.current = this;
  }

  resolveWebviewView(webviewView: vscode.WebviewView) {
    this.view = webviewView;
    this.view.webview.options = {
      enableScripts: true,
    };
    this.view.webview.html = this.render();
    this.view.webview.onDidReceiveMessage(
      (msg) => this.onMessage(msg),
      null,
      this.disposables,
    );
    this.flushQueuedMessages();
  }

  private onMessage(msg: any) {
    if (msg.type === "decide") {
      const resolve = this.pendingDecisions.get(msg.id);
      if (resolve) {
        this.pendingDecisions.delete(msg.id);
        resolve(!!msg.approve);
      }
    } else if (msg.type === "openDiff") {
      void vscode.commands.executeCommand(
        "vscode.diff",
        vscode.Uri.parse(msg.leftUri),
        vscode.Uri.parse(msg.rightUri),
        msg.title ?? "Thymos proposal",
        { preview: true },
      );
    }
  }

  private postMessage(message: { type: string; [key: string]: any }) {
    if (!this.view) {
      this.queuedMessages.push(message);
      return;
    }
    void this.view.webview.postMessage(message);
  }

  private flushQueuedMessages() {
    if (!this.view || this.queuedMessages.length === 0) return;
    const messages = [...this.queuedMessages];
    this.queuedMessages = [];
    for (const message of messages) {
      void this.view.webview.postMessage(message);
    }
  }

  log(text: string, level: "info" | "warn" | "error" = "info") {
    this.postMessage({ type: "log", text, level });
  }

  runStarted(runId: string, task: string) {
    this.postMessage({ type: "run", runId, task });
  }

  statusChanged(status: string) {
    this.postMessage({ type: "status", status });
  }

  finalAnswer(text: string) {
    this.postMessage({ type: "final", text });
  }

  reveal() {
    void vscode.commands.executeCommand("workbench.view.extension.thymos");
  }

  /** Show an approval card and resolve when the user clicks Approve/Deny. */
  promptApproval(card: {
    id: string;
    channel: string;
    reason: string;
    tool?: string;
    path?: string;
    mode?: string;
    lineDelta?: { removed: number; added: number };
    argsPreview?: string;
    diffUri?: { leftUri: string; rightUri: string; title: string };
  }): Promise<boolean> {
    this.reveal();
    return new Promise<boolean>((resolve) => {
      this.pendingDecisions.set(card.id, resolve);
      this.postMessage({ type: "approval", card });
    });
  }

  private render(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<style>
  body {
    font-family: var(--vscode-font-family);
    font-size: 13px;
    color: var(--vscode-editor-foreground);
    background: var(--vscode-editor-background);
    padding: 12px;
  }
  h2 { margin: 0 0 10px; font-size: 14px; font-weight: 600; }
  #header {
    display: flex; justify-content: space-between; align-items: baseline;
    border-bottom: 1px solid var(--vscode-editorWidget-border);
    padding-bottom: 8px; margin-bottom: 12px;
  }
  #status { font-size: 12px; color: var(--vscode-descriptionForeground); }
  .row { margin: 4px 0; white-space: pre-wrap; word-break: break-word; }
  .row.warn { color: var(--vscode-editorWarning-foreground); }
  .row.error { color: var(--vscode-editorError-foreground); }
  .card {
    border: 1px solid var(--vscode-focusBorder);
    border-radius: 6px;
    padding: 12px;
    margin: 14px 0;
    background: var(--vscode-editorWidget-background);
  }
  .card h3 {
    margin: 0 0 6px; font-size: 13px; font-weight: 600;
    color: var(--vscode-editorWarning-foreground);
  }
  .kv { display: grid; grid-template-columns: 80px 1fr; gap: 2px 10px; margin-bottom: 8px; }
  .kv span:nth-child(odd) { color: var(--vscode-descriptionForeground); }
  .actions { display: flex; gap: 8px; margin-top: 10px; }
  button {
    font-family: inherit; font-size: 12px; padding: 6px 14px;
    border-radius: 4px; border: none; cursor: pointer;
  }
  button.approve {
    background: var(--vscode-button-background);
    color: var(--vscode-button-foreground);
  }
  button.approve:hover { background: var(--vscode-button-hoverBackground); }
  button.deny {
    background: var(--vscode-button-secondaryBackground);
    color: var(--vscode-button-secondaryForeground);
  }
  button.deny:hover { background: var(--vscode-button-secondaryHoverBackground); }
  button.diff { background: transparent; color: var(--vscode-textLink-foreground); }
  button.diff:hover { text-decoration: underline; }
  pre.args {
    margin: 6px 0 0; padding: 8px; border-radius: 4px;
    background: var(--vscode-textCodeBlock-background);
    font-family: var(--vscode-editor-font-family);
    font-size: 12px; overflow-x: auto; max-height: 180px;
  }
  .final {
    margin-top: 14px; padding: 10px;
    border-left: 3px solid var(--vscode-textLink-foreground);
    background: var(--vscode-editorWidget-background);
  }
  .final h3 { margin: 0 0 6px; font-size: 12px; }
</style>
</head>
<body>
  <div id="header">
    <h2>🧠 Thymos</h2>
    <div id="status">idle</div>
  </div>
  <div id="log"></div>
  <script>
    const vscode = acquireVsCodeApi();
    const log = document.getElementById("log");
    const status = document.getElementById("status");

    function appendRow(text, level) {
      const div = document.createElement("div");
      div.className = "row" + (level && level !== "info" ? " " + level : "");
      div.textContent = text;
      log.appendChild(div);
      window.scrollTo(0, document.body.scrollHeight);
    }

    function appendCard(card) {
      const div = document.createElement("div");
      div.className = "card";
      div.id = "card-" + card.id;
      const delta = card.lineDelta
        ? (" (−" + card.lineDelta.removed + " +" + card.lineDelta.added + ")")
        : "";
      const diffButton = card.diffUri
        ? '<button class="diff" data-action="diff">Open diff editor</button>'
        : "";
      div.innerHTML = \`
        <h3>⚠️ Approval requested — \${escape(card.channel)}</h3>
        <div class="kv">
          <span>reason</span><span>\${escape(card.reason)}</span>
          \${card.tool ? "<span>tool</span><span>" + escape(card.tool) + "</span>" : ""}
          \${card.path ? "<span>path</span><span>" + escape(card.path) + delta + "</span>" : ""}
          \${card.mode ? "<span>mode</span><span>" + escape(card.mode) + "</span>" : ""}
        </div>
        \${card.argsPreview ? "<pre class=\\"args\\">" + escape(card.argsPreview) + "</pre>" : ""}
        <div class="actions">
          <button class="approve" data-action="approve">Approve</button>
          <button class="deny" data-action="deny">Deny</button>
          \${diffButton}
        </div>
      \`;
      div.querySelectorAll("button").forEach((btn) => {
        btn.addEventListener("click", (e) => {
          const action = e.target.getAttribute("data-action");
          if (action === "approve" || action === "deny") {
            vscode.postMessage({ type: "decide", id: card.id, approve: action === "approve" });
            div.querySelectorAll("button").forEach((b) => (b.disabled = true));
            const tag = document.createElement("div");
            tag.className = "row";
            tag.textContent = action === "approve" ? "  → approved" : "  → denied";
            div.appendChild(tag);
          } else if (action === "diff" && card.diffUri) {
            vscode.postMessage({
              type: "openDiff",
              leftUri: card.diffUri.leftUri,
              rightUri: card.diffUri.rightUri,
              title: card.diffUri.title,
            });
          }
        });
      });
      log.appendChild(div);
      window.scrollTo(0, document.body.scrollHeight);
    }

    function escape(s) {
      return String(s ?? "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
    }

    window.addEventListener("message", (e) => {
      const msg = e.data;
      if (msg.type === "log") {
        appendRow(msg.text, msg.level);
      } else if (msg.type === "run") {
        appendRow("▶ run " + msg.runId + " — " + msg.task, "info");
      } else if (msg.type === "status") {
        status.textContent = msg.status;
      } else if (msg.type === "approval") {
        appendCard(msg.card);
      } else if (msg.type === "final") {
        const div = document.createElement("div");
        div.className = "final";
        div.innerHTML = '<h3>Final answer</h3><div class="row"></div>';
        div.querySelector(".row").textContent = msg.text;
        log.appendChild(div);
        window.scrollTo(0, document.body.scrollHeight);
      }
    });
  </script>
</body>
</html>`;
  }

  dispose() {
    ThymosSidebar.current = undefined;
    for (const resolve of this.pendingDecisions.values()) {
      resolve(false);
    }
    this.pendingDecisions.clear();
    while (this.disposables.length) {
      const d = this.disposables.pop();
      try {
        d?.dispose();
      } catch {
        /* ignore */
      }
    }
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Status bar: live health indicator.
// ────────────────────────────────────────────────────────────────────────────

let statusItem: vscode.StatusBarItem;
let panelItem: vscode.StatusBarItem;
let shellItem: vscode.StatusBarItem;
let sidebar: ThymosSidebar;

async function refreshHealth(): Promise<void> {
  const cfg = getConfig();
  try {
    const { status } = await httpJson(`${cfg.url}/health`, {
      headers: authHeaders(cfg),
    });
    if (status === 200) {
      statusItem.text = "$(pulse) Thymos";
      statusItem.tooltip = `Thymos server OK — ${cfg.url}\nClick for launcher`;
      statusItem.backgroundColor = undefined;
    } else {
      statusItem.text = "$(warning) Thymos";
      statusItem.tooltip = `Thymos ${status} — ${cfg.url}\nClick for launcher`;
      statusItem.backgroundColor = new vscode.ThemeColor(
        "statusBarItem.warningBackground",
      );
    }
  } catch {
    statusItem.text = "$(debug-disconnect) Thymos";
    statusItem.tooltip = `Thymos unreachable — ${cfg.url}\nClick for launcher`;
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
  terminal.sendText(`${cfg.cliPath} shell`);
  if (cwd) {
    terminal.sendText(`set preset ${cfg.preset}`);
    terminal.sendText(`set workspace ${cwd}`);
  }
}

async function cmdQuickOpen(context: vscode.ExtensionContext) {
  const choice = await vscode.window.showQuickPick(
    [
      {
        label: "$(preview) Show Sidebar",
        description: "Reveal the docked Thymos sidebar",
        action: "panel",
      },
      {
        label: "$(terminal) Open Shell",
        description: "Launch the integrated Thymos terminal",
        action: "shell",
      },
      {
        label: "$(play) Run Task",
        description: "Submit a task to Thymos",
        action: "run",
      },
      {
        label: "$(pulse) Health Check",
        description: "Verify the Thymos server connection",
        action: "health",
      },
    ],
    {
      title: "Thymos",
      placeHolder: "Choose what to open",
      ignoreFocusOut: true,
    },
  );

  if (!choice) return;

  switch (choice.action) {
    case "panel":
      sidebar.reveal();
      break;
    case "shell":
      cmdOpenShell();
      break;
    case "run":
      await cmdRunTask(context);
      break;
    case "health":
      await cmdHealth();
      break;
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

async function cmdRunTask(context: vscode.ExtensionContext) {
  const cfg = getConfig();
  const cwd = workspaceRoot();
  if (!cwd) {
    vscode.window.showErrorMessage(
      "Open a folder before running a Thymos task.",
    );
    return;
  }

  const task = await vscode.window.showInputBox({
    prompt: "What should Thymos do?",
    placeHolder: "e.g. add a README section describing the shell",
    ignoreFocusOut: true,
  });
  if (!task) return;

  const panel = sidebar;
  panel.reveal();
  panel.log(`submitting: ${task}`);
  panel.statusChanged("submitting");

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
    panel.log(`error ${status}: ${JSON.stringify(body)}`, "error");
    vscode.window.showErrorMessage(`Thymos run failed: ${status}`);
    return;
  }

  const runId: string = body.run_id;
  panel.runStarted(runId, task);
  await pollRun(cfg, runId, cwd, panel);
}

async function pollRun(
  cfg: Config,
  runId: string,
  cwd: string,
  panel: ThymosSidebar,
): Promise<void> {
  const handled = new Set<number>();
  let lastStatus: string | null = null;

  while (true) {
    const status = await httpJson(`${cfg.url}/runs/${runId}`, {
      headers: authHeaders(cfg),
    });
    const st: string = status.body?.status ?? "?";
    if (st !== lastStatus) {
      panel.statusChanged(st);
      panel.log(`status: ${st}`);
      lastStatus = st;
    }
    if (st === "completed" || st === "failed" || st === "cancelled") {
      const final = status.body?.summary?.final_answer;
      if (final) {
        panel.finalAnswer(final);
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
      const approved = await handleApproval(cfg, cwd, entry, panel);
      const channel: string = entry.detail?.channel ?? "";
      if (channel) {
        await httpJson(`${cfg.url}/runs/${runId}/approvals/${channel}`, {
          method: "POST",
          headers: { "Content-Type": "application/json", ...authHeaders(cfg) },
          body: JSON.stringify({ approve: approved }),
        });
        panel.log(`${approved ? "approved" : "denied"} channel=${channel}`);
      }
      handled.add(entry.seq);
    }

    await new Promise((r) => setTimeout(r, 500));
  }
}

// ────────────────────────────────────────────────────────────────────────────
// Approval card + native diff support.
// ────────────────────────────────────────────────────────────────────────────

async function handleApproval(
  cfg: Config,
  cwd: string,
  entry: any,
  panel: ThymosSidebar,
): Promise<boolean> {
  const channel: string = entry.detail?.channel ?? "?";
  const reason: string = entry.detail?.reason ?? "(no reason)";
  const proposal = entry.detail?.proposal;
  const tool: string | undefined = proposal?.body?.plan?.tool;
  const args = proposal?.body?.plan?.args ?? {};

  const card: any = {
    id: `${entry.seq}`,
    channel,
    reason,
    tool,
  };

  if (tool === "fs_patch") {
    const path: string | undefined = args.path;
    const mode: string | undefined = args.mode;
    card.path = path;
    card.mode = mode;

    if (path && mode) {
      const absolute = path.startsWith("/") ? path : `${cwd}/${path}`;
      const fileUri = vscode.Uri.file(absolute);

      let current = "";
      try {
        const buf = await vscode.workspace.fs.readFile(fileUri);
        current = Buffer.from(buf).toString("utf8");
      } catch {
        current = "";
      }

      let proposed = current;
      if (mode === "write") {
        proposed = args.content ?? "";
      } else if (mode === "replace") {
        const anchor: string = args.anchor ?? "";
        const replacement: string = args.replacement ?? "";
        const occurrences = anchor ? current.split(anchor).length - 1 : 0;
        proposed = occurrences === 1 ? current.replace(anchor, replacement) : replacement;
      }

      card.lineDelta = {
        removed: current.split("\n").length,
        added: proposed.split("\n").length,
      };

      const proposedDoc = await vscode.workspace.openTextDocument({
        content: proposed,
        language: languageFor(path),
      });
      card.diffUri = {
        leftUri: fileUri.toString(),
        rightUri: proposedDoc.uri.toString(),
        title: `Thymos proposal: ${path}`,
      };
    }
  } else {
    card.argsPreview = JSON.stringify(args, null, 2);
  }

  return panel.promptApproval(card);
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

async function cmdReviewPending(context: vscode.ExtensionContext) {
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
  const panel = sidebar;
  panel.reveal();
  const cwd = workspaceRoot() ?? "";
  for (const entry of pending) {
    const approved = await handleApproval(cfg, cwd, entry, panel);
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
  sidebar = new ThymosSidebar(context);

  statusItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100,
  );
  statusItem.command = "thymos.quickOpen";
  statusItem.text = "$(pulse) Thymos";
  statusItem.show();
  panelItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    99,
  );
  panelItem.command = "thymos.showPanel";
  panelItem.text = "$(preview) Sidebar";
  panelItem.tooltip = "Reveal the Thymos sidebar";
  panelItem.show();

  shellItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    98,
  );
  shellItem.command = "thymos.openShell";
  shellItem.text = "$(terminal) Shell";
  shellItem.tooltip = "Open the Thymos integrated shell";
  shellItem.show();

  context.subscriptions.push(statusItem, panelItem, shellItem);
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(ThymosSidebar.viewType, sidebar, {
      webviewOptions: { retainContextWhenHidden: true },
    }),
    { dispose: () => sidebar.dispose() },
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("thymos.openShell", cmdOpenShell),
    vscode.commands.registerCommand("thymos.health", cmdHealth),
    vscode.commands.registerCommand("thymos.runTask", () =>
      cmdRunTask(context),
    ),
    vscode.commands.registerCommand("thymos.reviewPending", () =>
      cmdReviewPending(context),
    ),
    vscode.commands.registerCommand("thymos.showPanel", () => sidebar.reveal()),
    vscode.commands.registerCommand("thymos.quickOpen", () =>
      cmdQuickOpen(context),
    ),
  );

  const cfg = getConfig();
  if (cfg.openPanelOnStartup) {
    setTimeout(() => {
      try {
        sidebar.reveal();
      } catch {
        /* ignore sidebar startup errors */
      }
    }, 250);
  }

  refreshHealth();
  const interval = setInterval(refreshHealth, 15_000);
  context.subscriptions.push({ dispose: () => clearInterval(interval) });
}

export function deactivate() {}
