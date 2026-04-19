import { ThymosLogo } from "@/components/branding/ThymosLogo";
import { siteConfig } from "@/lib/site";

const trustItems = ["rust", "ed25519 writs", "sqlite ledger", "pre-alpha"];
const heroParticles = Array.from({ length: 14 }, (_, index) => index);

const frameworkCards = [
  {
    label: "Core",
    title: "Typed primitives.",
    body: "Writ, Intent, Proposal, Commit, Delta, World. Content-addressed via BLAKE3.",
  },
  {
    label: "Writs",
    title: "Signed capabilities.",
    body: "Ed25519-signed tokens bound to tool scopes, budget, effect class, and time window.",
  },
  {
    label: "Ledger",
    title: "Append-only, replayable.",
    body: "SQLite-backed, parent-chained. World state is projected by folding the chain.",
  },
  {
    label: "Policy",
    title: "Decide before effect.",
    body: "Allow, deny, or suspend for human approval — evaluated before any commit is written.",
  },
  {
    label: "Tools",
    title: "Contract-based effects.",
    body: "Schema, effect class, pre/postconditions, cost estimate. KV, memory, shell, HTTP, delegate, MCP bridge ship in-box.",
  },
  {
    label: "Cognition",
    title: "Multi-provider adapters.",
    body: "Anthropic (with prompt caching + extended thinking), OpenAI, local OpenAI-compatible endpoints, deterministic mock.",
  },
];

const architectureCards = [
  {
    step: "01",
    title: "Intent",
    body: "The model emits a typed intent. Intents are inert — they never mutate state.",
  },
  {
    step: "02",
    title: "Proposal",
    body: "The compiler resolves the writ, typechecks, runs policy, and checks the budget.",
  },
  {
    step: "03",
    title: "Commit",
    body: "The runtime invokes the tool contract, verifies postconditions, and signs the outcome.",
  },
  {
    step: "04",
    title: "Ledger",
    body: "The commit is appended, content-addressed, and parent-chained to the trajectory.",
  },
];

const crateList = [
  "thymos-core",
  "thymos-ledger",
  "thymos-policy",
  "thymos-tools",
  "thymos-compiler",
  "thymos-cognition",
  "thymos-runtime",
  "thymos-server",
  "thymos-cli",
  "thymos-client",
];

const provenanceCards = [
  {
    label: "Authorization",
    title: "Every mutation is bound to a Writ.",
    body: "No Writ, no commit. Child Writs can only be subsets of their parent — delegation is bounded by construction.",
  },
  {
    label: "Replay",
    title: "World is a fold of the ledger.",
    body: "Resource state is never stored separately. You can reconstruct exactly what the agent saw at any commit.",
  },
  {
    label: "Streaming",
    title: "Token-level observability.",
    body: "The HTTP server emits both cognition tokens and ledger entries as SSE, so every step is auditable live.",
  },
];

export function ThymosLanding() {
  return (
    <main className="thymos-page">
      <div className="thymos-backdrop thymos-backdrop-a" />
      <div className="thymos-backdrop thymos-backdrop-b" />
      <div className="thymos-backdrop thymos-backdrop-c" />
      <div className="thymos-noise" />

      <div className="thymos-shell">
        <header className="thymos-header thymos-reveal">
          <ThymosLogo />

          <nav className="thymos-nav" aria-label="Primary">
            <a href="#overview">Overview</a>
            <a href="#framework">Framework</a>
            <a href="#flow">Flow</a>
            <a href="#quickstart">Quickstart</a>
            <a href="#status">Status</a>
          </nav>

          <a
            className="thymos-header-cta"
            href={siteConfig.githubUrl}
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
        </header>

        <section className="thymos-hero" id="overview">
          <div className="thymos-hero-copy">
            <div className="thymos-pill thymos-reveal thymos-delay-1">
              <span className="thymos-pill-dot" />
              Pre-alpha · Rust workspace
            </div>

            <h1 className="thymos-reveal thymos-delay-2">{siteConfig.tagline}</h1>
            <p className="thymos-reveal thymos-delay-3">{siteConfig.headline}</p>
            <p className="thymos-hero-subcopy thymos-reveal thymos-delay-4">
              {siteConfig.subheadline}
            </p>

            <div className="thymos-hero-actions thymos-reveal thymos-delay-5">
              <a className="thymos-primary-action" href="#quickstart">
                Run the demo
              </a>
              <a
                className="thymos-secondary-action"
                href={siteConfig.githubUrl}
                target="_blank"
                rel="noreferrer"
              >
                View source
              </a>
            </div>

            <div className="thymos-trust-row thymos-reveal thymos-delay-6">
              {trustItems.map((item) => (
                <span className="thymos-trust-chip" key={item}>
                  {item}
                </span>
              ))}
            </div>
          </div>

          <div className="thymos-hero-art thymos-reveal thymos-delay-2">
            <div className="thymos-aurora-shell" />
            <div className="thymos-light-column thymos-light-column-a" />
            <div className="thymos-light-column thymos-light-column-b" />
            <div className="thymos-stage-floor" />
            <div className="thymos-particle-field" aria-hidden="true">
              {heroParticles.map((particle) => (
                <span className="thymos-particle" key={particle} />
              ))}
            </div>
            <div className="thymos-orbit thymos-orbit-a" />
            <div className="thymos-orbit thymos-orbit-b" />

            <div className="thymos-core-panel">
              <div className="thymos-core-glow" />
              <div className="thymos-core-screen">
                <div className="thymos-screen-topline">
                  <span>thymos.runtime</span>
                  <span>trajectory ledger</span>
                </div>

                <div className="thymos-screen-meters">
                  <span />
                  <span />
                  <span />
                </div>
              </div>
            </div>

            <div className="thymos-float thymos-float-left">
              <span className="thymos-float-label">Cognition</span>
              <strong>Intent emitted</strong>
              <div className="thymos-float-bars">
                <span />
                <span />
                <span />
              </div>
            </div>

            <div className="thymos-float thymos-float-right">
              <span className="thymos-float-label">Ledger</span>
              <strong>Commit #seq+1</strong>
              <div className="thymos-float-ring" />
            </div>
          </div>
        </section>

        <section className="thymos-framework thymos-section" id="framework">
          <div className="thymos-section-head">
            <span className="thymos-kicker">Framework</span>
            <h2>The runtime decides. The model proposes.</h2>
            <p>
              Cognition is the proposer. The runtime owns authorization, policy, effects, and
              history. Nothing reaches the outside world without a signed Writ and a ledger entry.
            </p>
          </div>

          <div className="thymos-pillars-grid">
            {frameworkCards.map((pillar) => (
              <article className="thymos-pillar-card" key={pillar.title}>
                <span className="thymos-card-label">{pillar.label}</span>
                <h3>{pillar.title}</h3>
                <p>{pillar.body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="thymos-architecture thymos-section" id="flow">
          <div className="thymos-section-head">
            <span className="thymos-kicker">IPC Triad</span>
            <h2>Intent → Proposal → Commit.</h2>
            <p>
              Four stages, one cycle. Every turn of the agent loop compiles down to this shape,
              which is why the whole history is replayable.
            </p>
          </div>

          <div className="thymos-architecture-grid compact">
            {architectureCards.map((item) => (
              <article className="thymos-architecture-card" key={item.step}>
                <span className="thymos-card-label">{item.step}</span>
                <h3>{item.title}</h3>
                <p>{item.body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="thymos-commercial thymos-section" id="provenance">
          <div className="thymos-section-head compact">
            <span className="thymos-kicker">Provenance</span>
            <h2>Why a runtime, not a prompt loop.</h2>
            <p>
              A loop of prompts plus JSON tool calls has no authorization layer and no durable
              record. Thymos fixes both.
            </p>
          </div>

          <div className="thymos-pillars-grid">
            {provenanceCards.map((item) => (
              <article className="thymos-pillar-card" key={item.title}>
                <span className="thymos-card-label">{item.label}</span>
                <h3>{item.title}</h3>
                <p>{item.body}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="thymos-architecture thymos-section" id="quickstart">
          <div className="thymos-section-head compact">
            <span className="thymos-kicker">Quickstart</span>
            <h2>Run a full Intent → Commit cycle in under a minute.</h2>
            <p>
              No API key needed. The <code>hello_triad</code> example signs a Writ, submits four
              intents, gets one rejected by policy, and prints the full trajectory.
            </p>
          </div>

          <div className="thymos-pillars-grid">
            <article className="thymos-pillar-card">
              <span className="thymos-card-label">1 · Clone</span>
              <h3>git clone</h3>
              <p>
                <code>git clone {siteConfig.githubUrl}</code>
              </p>
            </article>
            <article className="thymos-pillar-card">
              <span className="thymos-card-label">2 · Build</span>
              <h3>cargo build</h3>
              <p>
                <code>cd thymos && cargo build --release</code>
              </p>
            </article>
            <article className="thymos-pillar-card">
              <span className="thymos-card-label">3 · Triad demo</span>
              <h3>cargo run</h3>
              <p>
                <code>cargo run --example hello_triad -p thymos-runtime</code>
              </p>
            </article>
            <article className="thymos-pillar-card">
              <span className="thymos-card-label">4 · Server + UI</span>
              <h3>Live trajectory viewer</h3>
              <p>
                <code>cargo run -p thymos-server</code> then open{" "}
                <a href="/runs">/runs</a> to launch a mock run and watch the ledger stream.
              </p>
            </article>
          </div>
        </section>

        <section className="thymos-proofband thymos-section" id="status">
          <div className="thymos-section-head compact">
            <span className="thymos-kicker">Status · Phase 1</span>
            <h2>Reference implementation in Rust. 11 crates, 93 passing tests.</h2>
            <p>
              Pre-alpha. The IPC Triad, signed Writs, ledger, policy, tool contracts, cognition
              adapters, async runtime, HTTP server, and SSE streaming are implemented and tested.
              Not yet: shadow branches, deterministic replay, stratified memory, web debugger.
            </p>
          </div>

          <div className="thymos-usecase-row">
            {crateList.map((item) => (
              <span className="thymos-usecase-chip" key={item}>
                {item}
              </span>
            ))}
          </div>
        </section>

        <footer className="thymos-footer">
          <ThymosLogo />
          <div className="thymos-footer-copy">
            <strong>Thymos — governed-cognition runtime</strong>
            <span>
              An {siteConfig.org} project · Apache-2.0 · {siteConfig.supportEmail}
            </span>
          </div>
        </footer>
      </div>
    </main>
  );
}
