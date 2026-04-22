---
layout: default
title: Thymos
hide_title: true
hero: true
permalink: /
---

<section class="hero">
  <div class="hero-eyebrow">
    <span class="dot"></span>
    THYMOS · UNIFIED AI EXECUTION RUNTIME
  </div>

  <h1>
    One runtime.<br />
    Every interface sees the same run.
  </h1>

  <p class="lede">
    Thymos is a shared execution system for coding agents. Start a task from
    the CLI, a VS Code sidebar, a terminal shell, or a web console and attach
    every surface to the same backend runtime, the same execution loop, and
    the same live execution log.
  </p>

  <div class="hero-cta">
    <a class="btn btn-primary" href="{{ '/getting-started' | relative_url }}">
      Start in 5 minutes <span class="btn-arrow">→</span>
    </a>
    <a class="btn btn-ghost" href="{{ '/interfaces' | relative_url }}">
      See the interfaces
    </a>
  </div>

  <div class="hero-meta">
    <span class="mono">Intent -> Proposal -> Execution -> Result</span>
    <span>·</span>
    <span>CLI · VS Code · Terminal · Web</span>
    <span>·</span>
    <span>Real tools · Real logs · Real runtime state</span>
  </div>
</section>

<section class="section">
  <div class="triad reveal">
    <div class="card">
      <h4>Agentic</h4>
      <p>It keeps carrying work forward.</p>
      <p class="sub">
        Thymos plans, executes, observes, retries, and adapts until a task is
        completed, blocked, or cancelled.
      </p>
    </div>
    <div class="card">
      <h4>Shared</h4>
      <p>One run, many surfaces.</p>
      <p class="sub">
        The CLI, VS Code sidebar, terminal shell, and web console are all
        windows into the same execution session.
      </p>
    </div>
    <div class="card">
      <h4>Observable</h4>
      <p>Every phase is visible.</p>
      <p class="sub">
        Intent, proposal, execution, approval, commit, failure, retry, and
        result are all first-class runtime events.
      </p>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-h reveal">
    <p class="eyebrow">What Thymos really is</p>
    <h2>Not a chat UI. Not a loose tool loop.</h2>
    <p>
      Thymos is an execution runtime for agentic work. The model proposes the
      next move, but the runtime owns authority, execution, observation,
      logging, and completion state. That is why every surface can stay in
      sync and every run can stay controlled.
    </p>
  </div>

  <div class="cards reveal">
    <div class="fcard">
      <div class="icon">◎</div>
      <h3>Structured flow</h3>
      <p>
        Every task runs through the same four-step model:
        Intent, Proposal, Execution, and Result.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⎇</div>
      <h3>Shared execution session</h3>
      <p>
        The backend exposes a live session with current phase, operator state,
        counters, and an execution log for every run.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">✦</div>
      <h3>Real runtime recovery</h3>
      <p>
        Tool execution failures and transient cognition failures become visible
        runtime events the agent can recover from.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⬢</div>
      <h3>Controlled autonomy</h3>
      <p>
        Signed writs, policy checks, typed tools, and approvals let the agent
        work forward without becoming unbounded.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⟐</div>
      <h3>Operator-grade visibility</h3>
      <p>
        The log is not an afterthought. It is the main product surface for
        understanding what the runtime is doing.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⎈</div>
      <h3>Model flexibility</h3>
      <p>
        Hosted or local cognition can drive the same runtime:
        Anthropic, OpenAI, LM Studio, Hugging Face, Ollama, and more.
      </p>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-h reveal">
    <p class="eyebrow">Use it your way</p>
    <h2>Start anywhere. Stay on the same run.</h2>
    <p>
      Thymos is designed so different users can meet the runtime from the
      interface they already prefer.
    </p>
  </div>

  <div class="cards reveal">
    <div class="fcard">
      <div class="icon">⌘</div>
      <h3>CLI</h3>
      <p>
        Launch, follow, inspect, diff, resume, and cancel runs from a terminal
        without losing live execution state.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">▣</div>
      <h3>Web console</h3>
      <p>
        Use the operator console for live execution state, world replay,
        branching, and a premium execution-log view.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">▤</div>
      <h3>VS Code sidebar</h3>
      <p>
        Review live run state and approvals without leaving the editor. The
        sidebar is attached to the same backend run as everything else.
      </p>
    </div>
  </div>
</section>

<section class="section">
  <div class="terminal reveal">
    <div class="terminal-bar">
      <span class="dot r"></span><span class="dot y"></span><span class="dot g"></span>
      <span class="title">thymos execution session</span>
    </div>
    <div class="terminal-body" data-speed="14" data-type='[
      {"text":"$ thymos run \"add a retry helper and verify it\" --provider mock --follow","cls":"cmd","pause":240},
      {"text":"[running | intent] Planning step 1","cls":"muted","pause":180},
      {"text":"[INFO:intent step 1 repo_map] Intent issued for repo_map","cls":"out","pause":200},
      {"text":"[SUCCESS:proposal step 1 repo_map] Proposal staged for repo_map","cls":"out","pause":180},
      {"text":"[INFO:execution step 1 repo_map] Execution started for repo_map","cls":"out","pause":180},
      {"text":"[SUCCESS:result step 1 repo_map] Result committed for repo_map","cls":"ok","pause":180},
      {"text":"[ERROR:result step 2 test_run] Execution failed for test_run","cls":"err","pause":200},
      {"text":"[running | result] Recovering from a runtime failure","cls":"muted","pause":180},
      {"text":"[SUCCESS:result step 3 fs_patch] Result committed for fs_patch","cls":"ok","pause":180},
      {"text":"[completed | result] Task resolved and verified by runtime","cls":"hl","pause":0}
    ]'>
    </div>
  </div>
</section>

<section class="section">
  <div class="cta-wrap reveal">
    <h2>Learn the product from the outside in.</h2>
    <p>
      Start with the onboarding path, then choose the interface, then go deep
      on architecture and API details only when you need them.
    </p>
    <div class="hero-cta">
      <a class="btn btn-primary" href="{{ '/getting-started' | relative_url }}">
        Get started <span class="btn-arrow">→</span>
      </a>
      <a class="btn btn-ghost" href="{{ '/faq' | relative_url }}">
        Read the FAQ
      </a>
    </div>
  </div>
</section>
