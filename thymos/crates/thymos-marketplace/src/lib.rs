//! Tool marketplace: publish, discover, and install manifest tools and MCP servers.
//!
//! The marketplace is an in-memory registry (Phase 1) that holds tool packages.
//! Each package describes either a **manifest tool** (JSON schema + executor) or
//! an **MCP server** (command + args) that can be installed into a ToolRegistry.
//!
//! Packages are versioned with semver and content-hashed for integrity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("package not found: {0}")]
    NotFound(String),
    #[error("version conflict: {name}@{version} already exists")]
    VersionConflict { name: String, version: String },
    #[error("integrity mismatch: expected {expected}, got {actual}")]
    IntegrityMismatch { expected: String, actual: String },
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
}

// ── Package schema ───────────────────────────────────────────────────────────

/// How the tool is executed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackageKind {
    /// A manifest tool: inline JSON schema + executor config.
    Manifest {
        /// The tool manifest JSON (same format as thymos-tools ToolManifest).
        manifest: serde_json::Value,
    },
    /// An MCP server: spawn a subprocess, discover tools via JSON-RPC.
    McpServer {
        /// Command to run (e.g. "uvx", "npx", "node").
        command: String,
        /// Arguments (e.g. ["my-mcp-server", "--port", "0"]).
        args: Vec<String>,
        /// Environment variables to set.
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

/// A published tool package in the marketplace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    /// Unique package name (e.g. "thymos/kv-tools", "community/weather").
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Author or org.
    pub author: String,
    /// Tags for search/discovery.
    #[serde(default)]
    pub tags: Vec<String>,
    /// The tool kind and its configuration.
    pub kind: PackageKind,
    /// BLAKE3 hash of the canonical JSON representation (integrity check).
    #[serde(default)]
    pub content_hash: String,
    /// ISO-8601 timestamp of publication.
    #[serde(default)]
    pub published_at: String,
}

impl Package {
    /// Compute the content hash from the package's kind payload.
    pub fn compute_hash(&self) -> String {
        let payload = serde_json::to_vec(&self.kind).unwrap_or_default();
        let hash = blake3::hash(&payload);
        hex::encode(hash.as_bytes())
    }

    /// Verify the content hash matches the payload.
    pub fn verify_integrity(&self) -> Result<(), MarketplaceError> {
        if self.content_hash.is_empty() {
            return Ok(()); // No hash set, skip verification.
        }
        let actual = self.compute_hash();
        if actual != self.content_hash {
            return Err(MarketplaceError::IntegrityMismatch {
                expected: self.content_hash.clone(),
                actual,
            });
        }
        Ok(())
    }
}

// ── Search ───────────────────────────────────────────────────────────────────

/// Search query for the marketplace.
#[derive(Clone, Debug, Default)]
pub struct SearchQuery {
    /// Substring match on name or description.
    pub text: Option<String>,
    /// Filter by tags (all must match).
    pub tags: Vec<String>,
    /// Filter by author.
    pub author: Option<String>,
    /// Filter by kind ("manifest" or "mcp_server").
    pub kind: Option<String>,
}

// ── Registry ─────────────────────────────────────────────────────────────────

/// In-memory tool marketplace registry.
#[derive(Default)]
pub struct Marketplace {
    /// name -> (version -> Package). Supports multiple versions per package.
    packages: HashMap<String, HashMap<String, Package>>,
}

impl Marketplace {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish a package to the marketplace.
    pub fn publish(&mut self, mut pkg: Package) -> Result<(), MarketplaceError> {
        // Compute content hash if not set.
        if pkg.content_hash.is_empty() {
            pkg.content_hash = pkg.compute_hash();
        } else {
            pkg.verify_integrity()?;
        }

        let versions = self.packages.entry(pkg.name.clone()).or_default();
        if versions.contains_key(&pkg.version) {
            return Err(MarketplaceError::VersionConflict {
                name: pkg.name.clone(),
                version: pkg.version.clone(),
            });
        }
        versions.insert(pkg.version.clone(), pkg);
        Ok(())
    }

    /// Get a specific package version. If version is None, returns the latest.
    pub fn get(&self, name: &str, version: Option<&str>) -> Result<&Package, MarketplaceError> {
        let versions = self
            .packages
            .get(name)
            .ok_or_else(|| MarketplaceError::NotFound(name.to_string()))?;

        match version {
            Some(v) => versions
                .get(v)
                .ok_or_else(|| MarketplaceError::NotFound(format!("{name}@{v}"))),
            None => {
                // Return the "latest" — simple lexicographic max for now.
                versions
                    .values()
                    .max_by(|a, b| a.version.cmp(&b.version))
                    .ok_or_else(|| MarketplaceError::NotFound(name.to_string()))
            }
        }
    }

    /// List all packages (latest version of each).
    pub fn list(&self) -> Vec<&Package> {
        self.packages
            .values()
            .filter_map(|versions| versions.values().max_by(|a, b| a.version.cmp(&b.version)))
            .collect()
    }

    /// Search packages by query.
    pub fn search(&self, query: &SearchQuery) -> Vec<&Package> {
        self.list()
            .into_iter()
            .filter(|pkg| {
                // Text filter.
                if let Some(text) = &query.text {
                    let t = text.to_lowercase();
                    if !pkg.name.to_lowercase().contains(&t)
                        && !pkg.description.to_lowercase().contains(&t)
                    {
                        return false;
                    }
                }
                // Tags filter (all must match).
                for tag in &query.tags {
                    if !pkg.tags.contains(tag) {
                        return false;
                    }
                }
                // Author filter.
                if let Some(author) = &query.author {
                    if pkg.author != *author {
                        return false;
                    }
                }
                // Kind filter.
                if let Some(kind) = &query.kind {
                    let matches = match (&pkg.kind, kind.as_str()) {
                        (PackageKind::Manifest { .. }, "manifest") => true,
                        (PackageKind::McpServer { .. }, "mcp_server") => true,
                        _ => false,
                    };
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Remove a specific version. Returns the removed package if found.
    pub fn unpublish(
        &mut self,
        name: &str,
        version: &str,
    ) -> Result<Package, MarketplaceError> {
        let versions = self
            .packages
            .get_mut(name)
            .ok_or_else(|| MarketplaceError::NotFound(name.to_string()))?;

        let pkg = versions
            .remove(version)
            .ok_or_else(|| MarketplaceError::NotFound(format!("{name}@{version}")))?;

        // Clean up empty version maps.
        if versions.is_empty() {
            self.packages.remove(name);
        }

        Ok(pkg)
    }

    /// Total number of packages (counting each version separately).
    pub fn total_versions(&self) -> usize {
        self.packages.values().map(|v| v.len()).sum()
    }

    /// Number of unique package names.
    pub fn total_packages(&self) -> usize {
        self.packages.len()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_pkg(name: &str, version: &str) -> Package {
        Package {
            name: name.into(),
            version: version.into(),
            description: "A sample manifest tool".into(),
            author: "thymos".into(),
            tags: vec!["kv".into(), "storage".into()],
            kind: PackageKind::Manifest {
                manifest: serde_json::json!({
                    "name": "kv_set",
                    "description": "Set a key-value pair",
                    "parameters": { "key": "string", "value": "string" },
                    "executor": { "type": "noop" }
                }),
            },
            content_hash: String::new(),
            published_at: String::new(),
        }
    }

    fn sample_mcp_pkg(name: &str, version: &str) -> Package {
        Package {
            name: name.into(),
            version: version.into(),
            description: "An MCP weather server".into(),
            author: "community".into(),
            tags: vec!["weather".into(), "mcp".into()],
            kind: PackageKind::McpServer {
                command: "uvx".into(),
                args: vec!["weather-server".into()],
                env: HashMap::new(),
            },
            content_hash: String::new(),
            published_at: String::new(),
        }
    }

    #[test]
    fn publish_and_get() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();

        let pkg = mp.get("thymos/kv", Some("0.1.0")).unwrap();
        assert_eq!(pkg.name, "thymos/kv");
        assert!(!pkg.content_hash.is_empty());
    }

    #[test]
    fn get_latest_version() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.2.0")).unwrap();

        let pkg = mp.get("thymos/kv", None).unwrap();
        assert_eq!(pkg.version, "0.2.0");
    }

    #[test]
    fn version_conflict() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        let err = mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap_err();
        assert!(matches!(err, MarketplaceError::VersionConflict { .. }));
    }

    #[test]
    fn search_by_text() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_mcp_pkg("community/weather", "1.0.0")).unwrap();

        let results = mp.search(&SearchQuery {
            text: Some("weather".into()),
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "community/weather");
    }

    #[test]
    fn search_by_tag() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_mcp_pkg("community/weather", "1.0.0")).unwrap();

        let results = mp.search(&SearchQuery {
            tags: vec!["mcp".into()],
            ..Default::default()
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "community/weather");
    }

    #[test]
    fn search_by_kind() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_mcp_pkg("community/weather", "1.0.0")).unwrap();

        let manifest_results = mp.search(&SearchQuery {
            kind: Some("manifest".into()),
            ..Default::default()
        });
        assert_eq!(manifest_results.len(), 1);

        let mcp_results = mp.search(&SearchQuery {
            kind: Some("mcp_server".into()),
            ..Default::default()
        });
        assert_eq!(mcp_results.len(), 1);
    }

    #[test]
    fn unpublish() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.2.0")).unwrap();

        let removed = mp.unpublish("thymos/kv", "0.1.0").unwrap();
        assert_eq!(removed.version, "0.1.0");
        assert_eq!(mp.total_versions(), 1);

        // Latest is still 0.2.0.
        let pkg = mp.get("thymos/kv", None).unwrap();
        assert_eq!(pkg.version, "0.2.0");
    }

    #[test]
    fn integrity_check() {
        let pkg = sample_manifest_pkg("thymos/kv", "0.1.0");
        let hash = pkg.compute_hash();
        assert!(!hash.is_empty());

        // Correct hash passes.
        let mut pkg2 = pkg.clone();
        pkg2.content_hash = hash;
        pkg2.verify_integrity().unwrap();

        // Wrong hash fails.
        let mut pkg3 = pkg;
        pkg3.content_hash = "deadbeef".into();
        assert!(matches!(
            pkg3.verify_integrity(),
            Err(MarketplaceError::IntegrityMismatch { .. })
        ));
    }

    #[test]
    fn list_shows_latest_only() {
        let mut mp = Marketplace::new();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.1.0")).unwrap();
        mp.publish(sample_manifest_pkg("thymos/kv", "0.2.0")).unwrap();
        mp.publish(sample_mcp_pkg("community/weather", "1.0.0")).unwrap();

        let all = mp.list();
        assert_eq!(all.len(), 2);
        assert_eq!(mp.total_versions(), 3);
        assert_eq!(mp.total_packages(), 2);
    }

    #[test]
    fn serde_roundtrip() {
        let pkg = sample_mcp_pkg("community/weather", "1.0.0");
        let json = serde_json::to_string(&pkg).unwrap();
        let restored: Package = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, pkg.name);
        assert_eq!(restored.version, pkg.version);
        match &restored.kind {
            PackageKind::McpServer { command, args, .. } => {
                assert_eq!(command, "uvx");
                assert_eq!(args, &["weather-server"]);
            }
            _ => panic!("expected McpServer"),
        }
    }
}
