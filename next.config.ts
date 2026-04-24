import type { NextConfig } from "next";

function normalizeBasePath(value: string | undefined): string {
  const trimmed = value?.trim() ?? "";

  if (!trimmed || trimmed === "/") {
    return "";
  }

  return `/${trimmed.replace(/^\/+|\/+$/g, "")}`;
}

function hasCustomDomain(): boolean {
  if (process.env.PAGES_CUSTOM_DOMAIN?.trim()) {
    return true;
  }

  const siteUrl = process.env.NEXT_PUBLIC_SITE_URL?.trim();
  if (!siteUrl) {
    return false;
  }

  try {
    const hostname = new URL(siteUrl).hostname.toLowerCase();
    return Boolean(hostname && !hostname.endsWith(".github.io"));
  } catch {
    return false;
  }
}

const isGithubPages = process.env.GITHUB_ACTIONS === "true";
const repoName = process.env.GITHUB_REPOSITORY?.split("/")[1] ?? "OpenThymos";
const inferredBasePath = isGithubPages && !hasCustomDomain() ? `/${repoName}` : "";
const basePath = normalizeBasePath(process.env.NEXT_PUBLIC_BASE_PATH || inferredBasePath);

const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "export",
  trailingSlash: true,
  basePath,
  images: {
    unoptimized: true,
  },
  env: {
    NEXT_PUBLIC_BASE_PATH: basePath,
  },
};

export default nextConfig;
