const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";
const githubUrl = "https://github.com/gryszzz/OpenThymos";

export const siteConfig = {
  name: process.env.NEXT_PUBLIC_APP_NAME || "OpenThymos",
  tagline: "Models propose. OpenThymos governs execution.",
  headline: "Governed execution for AI systems.",
  subheadline:
    "A model-agnostic runtime where typed intents, signed writs, policy gates, and a replayable trajectory ledger keep machine action bounded and auditable.",
  basePath,
  supportEmail: "team@thymos.ai",
  githubUrl,
  docsUrl: `${githubUrl}/tree/main/docs`,
  readmeUrl: `${githubUrl}#readme`,
  wikiUrl: `${githubUrl}/wiki`,
  org: "Exponet Labs",
};
