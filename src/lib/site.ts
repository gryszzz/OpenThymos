const defaultSiteUrl = "https://thymos.ai";

function normalizeBasePath(value: string | undefined): string {
  const trimmed = value?.trim() ?? "";

  if (!trimmed || trimmed === "/") {
    return "";
  }

  return `/${trimmed.replace(/^\/+|\/+$/g, "")}`;
}

function normalizeSiteUrl(value: string | undefined): string {
  const trimmed = value?.trim().replace(/\/+$/, "") ?? "";

  if (trimmed) {
    return trimmed;
  }

  const customDomain = process.env.PAGES_CUSTOM_DOMAIN?.trim().replace(/\/+$/, "");
  if (customDomain) {
    return `https://${customDomain.replace(/^https?:\/\//, "")}`;
  }

  return defaultSiteUrl;
}

const basePath = normalizeBasePath(process.env.NEXT_PUBLIC_BASE_PATH);
const githubUrl = "https://github.com/gryszzz/OpenThymos";

export const siteConfig = {
  name: process.env.NEXT_PUBLIC_APP_NAME || "OpenThymos",
  tagline: "A provider-neutral execution framework for agentic software.",
  headline: "Governed execution for AI agents.",
  subheadline:
    "OpenThymos turns model output into typed intents, checks them against signed authority, executes approved tools, and records every outcome in a replayable ledger.",
  basePath,
  siteUrl: normalizeSiteUrl(process.env.NEXT_PUBLIC_SITE_URL || defaultSiteUrl),
  supportEmail: "team@thymos.ai",
  githubUrl,
  docsUrl: `${githubUrl}/tree/main/docs`,
  readmeUrl: `${githubUrl}#readme`,
  wikiUrl: `${githubUrl}/wiki`,
  org: "Exponet Labs",
};
