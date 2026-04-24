const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";
const githubUrl = "https://github.com/gryszzz/OpenThymos";

export const siteConfig = {
  name: process.env.NEXT_PUBLIC_APP_NAME || "OpenThymos",
  tagline: "A provider-neutral execution framework for agentic software.",
  headline: "Governed execution for AI agents.",
  subheadline:
    "OpenThymos turns model output into typed intents, checks them against signed authority, executes approved tools, and records every outcome in a replayable ledger.",
  basePath,
  supportEmail: "team@thymos.ai",
  githubUrl,
  docsUrl: `${githubUrl}/tree/main/docs`,
  readmeUrl: `${githubUrl}#readme`,
  wikiUrl: `${githubUrl}/wiki`,
  org: "Exponet Labs",
};
