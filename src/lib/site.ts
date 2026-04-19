const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";

export const siteConfig = {
  name: process.env.NEXT_PUBLIC_APP_NAME || "OpenThymos",
  tagline: "Models propose. OpenThymos governs execution.",
  headline: "Governed execution for AI systems.",
  subheadline:
    "A model-agnostic runtime where typed intents, signed writs, policy gates, and a replayable trajectory ledger keep machine action bounded and auditable.",
  basePath,
  supportEmail: "team@thymos.ai",
  githubUrl: "https://github.com/gryszzz/THYMOS",
  docsUrl: "/docs",
  org: "Exponet Labs",
};
