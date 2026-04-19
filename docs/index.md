---
layout: default
title: Thymos — the governed runtime for code agents
hide_title: true
hero: true
permalink: /
---

<section class="hero">
  <div class="hero-eyebrow">
    <span class="dot"></span>
    PRE-ALPHA · BUILT BY EXPONET LABS
  </div>

  <h1>
    The governed runtime<br />
    for <em>code agents</em>.
  </h1>

  <p class="lede">
    Thymos treats a language model as a bounded <b>proposer</b> against a
    policy-governed, ledgered runtime. Cognition proposes. The runtime decides.
    The ledger remembers. Ship a coding agent that is reproducible,
    auditable, and runs on the model you want — cloud or local.
  </p>

  <div class="hero-cta">
    <a class="btn btn-primary" href="{{ '/getting-started' | relative_url }}">
      Launch locally <span class="btn-arrow">→</span>
    </a>
    <a class="btn btn-ghost" href="{{ site.repo }}" target="_blank" rel="noopener">
      View on GitHub
    </a>
  </div>

  <div class="hero-meta">
    <span class="mono">cargo&nbsp;run&nbsp;-p&nbsp;thymos-server</span>
    <span>·</span>
    <span>Rust · 12 crates · 93 tests passing</span>
    <span>·</span>
    <span>Apache-2.0</span>
  </div>
</section>

<section class="section">
  <div class="triad reveal">
    <div class="card">
      <h4>Cognition</h4>
      <p>Proposes.</p>
      <p class="sub">
        The model emits typed <span style="font-family:var(--font-mono)">Intents</span>.
        It never executes, never mutates state, never touches the ledger.
      </p>
    </div>
    <div class="card">
      <h4>Runtime</h4>
      <p>Decides.</p>
      <p class="sub">
        Compiles Intents into Proposals under a signed Capability Writ.
        Evaluates policy. Stages effects through typed tool contracts.
      </p>
    </div>
    <div class="card">
      <h4>Ledger</h4>
      <p>Remembers.</p>
      <p class="sub">
        Append-only, content-addressed via BLAKE3, parent-chained.
        Every run replays byte-for-byte.
      </p>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-h reveal">
    <p class="eyebrow">Live · in the repo today</p>
    <h2>A coding agent you actually trust</h2>
    <p>
      Typed file ops. Patched edits with pre/post-condition hashes.
      A secure shell with capability profiles and execution receipts.
      Run it against Claude, GPT, or a model loaded in LM Studio on your
      laptop. Same guarantees either way.
    </p>
  </div>

  <div class="terminal reveal">
    <div class="terminal-bar">
      <span class="dot r"></span><span class="dot y"></span><span class="dot g"></span>
      <span class="title">thymos — code run</span>
    </div>
    <div class="terminal-body" data-speed="14" data-type='[
      {"text":"$ thymos code run \"add retry to the HTTP client\" --provider lmstudio","cls":"cmd","pause":260},
      {"text":"→ writ minted  · scopes: fs_read,fs_patch,repo_map,test_run","cls":"muted","pause":180},
      {"text":"→ repo_map     · 412 files · cargo workspace detected","cls":"out","pause":200},
      {"text":"→ fs_read      · crates/net/src/client.rs  (3.2 KB)","cls":"out","pause":200},
      {"text":"→ fs_patch     · 1 hunk · +18 −2  · pre-hash ok · post-hash ok","cls":"out","pause":220},
      {"text":"→ test_run     · cargo test -p thymos-net  · 14 passed","cls":"ok","pause":220},
      {"text":"→ commit       · c7f4ab02 · ledger ← trajectory 41a9...","cls":"out","pause":180},
      {"text":"done · 3 steps · 2 commits · 0 rejections · 4.2s","cls":"hl","pause":0}
    ]'>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-h reveal">
    <p class="eyebrow">Why this instead of a wrapper CLI</p>
    <h2>Governance is the feature.</h2>
    <p>
      Everything an agent does has a typed contract, a signed writ, and a
      ledger entry. Nothing runs because the model said so. Everything runs
      because policy allowed it.
    </p>
  </div>

  <div class="cards reveal">
    <div class="fcard">
      <div class="icon">⬢</div>
      <h3>Signed writs</h3>
      <p>
        Ed25519-signed capability writs. Scoped tools, budgets, time windows,
        delegation bounds. Forged writs fail at the compiler.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⟐</div>
      <h3>Typed tool contracts</h3>
      <p>
        Effect class, risk class, pre/post-conditions against a world
        projection. Tools declare the state delta they produce.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">✦</div>
      <h3>Secure tool fabric</h3>
      <p>
        Shell and HTTP execute behind a subprocess worker boundary with
        capability profiles, timeouts, and execution receipts.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⎇</div>
      <h3>Replayable trajectory</h3>
      <p>
        BLAKE3 content-addressed commits, parent-chained.
        Branch, resume, diff, verify — same ledger primitives as git.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">◎</div>
      <h3>Any model, one loop</h3>
      <p>
        Anthropic, OpenAI, or any OpenAI-compatible endpoint — LM&nbsp;Studio,
        Ollama, vLLM, llama.cpp. Swap providers without re-plumbing.
      </p>
    </div>
    <div class="fcard">
      <div class="icon">⎈</div>
      <h3>Production seams</h3>
      <p>
        Async runtime, SSE token streaming, JWT + API-gateway auth, tenant
        isolation, run persistence, OpenTelemetry tracing.
      </p>
    </div>
  </div>
</section>

<section class="section">
  <div class="section-h reveal">
    <p class="eyebrow">Bring your own model</p>
    <h2>First-class local cognition.</h2>
    <p>
      The <span style="font-family:var(--font-mono)">OpenAiCognition</span>
      adapter accepts any base URL. Point it at a model running on your laptop
      and the same governance layer applies.
    </p>
  </div>

  <div class="providers reveal">
    <div class="provider"><div class="name">Anthropic</div><div class="meta">Opus 4.7 · Sonnet · Haiku</div></div>
    <div class="provider"><div class="name">OpenAI</div><div class="meta">GPT-4o family</div></div>
    <div class="provider"><div class="name">LM Studio</div><div class="meta">localhost:1234</div></div>
    <div class="provider"><div class="name">Ollama</div><div class="meta">localhost:11434</div></div>
  </div>
</section>

<section class="section">
  <div class="cta-wrap reveal">
    <h2>Stand up a governed coding agent today.</h2>
    <p>
      One <span style="font-family:var(--font-mono)">cargo run</span>.
      Full trajectory ledger. Zero-config mock cognition out of the box,
      production-grade cognition when you wire a key or a local endpoint.
    </p>
    <div class="hero-cta">
      <a class="btn btn-primary" href="{{ '/getting-started' | relative_url }}">
        Get started <span class="btn-arrow">→</span>
      </a>
      <a class="btn btn-ghost" href="{{ '/coding-agent' | relative_url }}">Read the coding-agent spec</a>
    </div>
  </div>
</section>
