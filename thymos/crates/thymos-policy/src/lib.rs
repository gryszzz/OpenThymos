//! Thymos policy engine.
//!
//! A policy is a pure function: `(Intent, Writ, WorldView) -> Decision`.
//! Policies are evaluated twice in the lifecycle — at compile time and at
//! stage time. Phase 1 exposes the compile-time gate only.

use thymos_core::{
    intent::Intent,
    proposal::{PolicyDecision, PolicyTrace},
    world::World,
    writ::Writ,
};

pub trait Policy: Send + Sync {
    /// Stable name used in `PolicyTrace.rules_evaluated`.
    fn name(&self) -> &'static str;

    /// Rules that opt-in to a given tool only. Return true to have the engine
    /// run this policy for the intent. Default: always.
    fn applies_to(&self, _intent: &Intent) -> bool {
        true
    }

    fn evaluate(&self, intent: &Intent, writ: &Writ, world: &World) -> PolicyDecision;
}

/// The default engine evaluates policies in declaration order, short-circuits
/// on the first non-permit, and emits a `PolicyTrace`.
pub struct PolicyEngine {
    rules: Vec<Box<dyn Policy>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        PolicyEngine { rules: Vec::new() }
    }

    pub fn with<P: Policy + 'static>(mut self, policy: P) -> Self {
        self.rules.push(Box::new(policy));
        self
    }

    pub fn evaluate(&self, intent: &Intent, writ: &Writ, world: &World) -> PolicyTrace {
        let mut evaluated = Vec::new();
        for rule in &self.rules {
            if !rule.applies_to(intent) {
                continue;
            }
            evaluated.push(rule.name().to_string());
            match rule.evaluate(intent, writ, world) {
                PolicyDecision::Permit => continue,
                decision => {
                    return PolicyTrace {
                        rules_evaluated: evaluated,
                        decision,
                    };
                }
            }
        }
        PolicyTrace {
            rules_evaluated: evaluated,
            decision: PolicyDecision::Permit,
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ----- stock policies -----------------------------------------------------

/// Denies anything not authorized by the Writ's tool scopes.
pub struct WritAuthorityPolicy;
impl Policy for WritAuthorityPolicy {
    fn name(&self) -> &'static str {
        "writ.authority"
    }
    fn applies_to(&self, intent: &Intent) -> bool {
        matches!(intent.body.kind, thymos_core::intent::IntentKind::Act)
    }
    fn evaluate(&self, intent: &Intent, writ: &Writ, _world: &World) -> PolicyDecision {
        if writ.authorizes_tool(&intent.body.target) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny(format!(
                "writ does not authorize tool '{}'",
                intent.body.target
            ))
        }
    }
}

/// Enforces tenant isolation: resources in the world that belong to a different
/// tenant cannot be accessed. This policy checks that the intent's target
/// resource (for Act intents targeting kv/memory resources) is within the
/// writ's tenant scope.
///
/// Convention: resource IDs for tenant-scoped data are prefixed with
/// `{tenant_id}/` (e.g., `tenant-123/greeting`). System resources (no prefix)
/// are accessible by any tenant.
pub struct TenantIsolationPolicy;

impl Policy for TenantIsolationPolicy {
    fn name(&self) -> &'static str {
        "tenant.isolation"
    }

    fn evaluate(&self, intent: &Intent, writ: &Writ, _world: &World) -> PolicyDecision {
        let tenant_id = &writ.body.tenant_id;
        if tenant_id.is_empty() {
            // System writ — no tenant boundary.
            return PolicyDecision::Permit;
        }

        // Check if the intent's args reference a key that belongs to another tenant.
        if let Some(key) = intent.body.args.get("key").and_then(|v| v.as_str()) {
            if let Some(prefix) = key.split('/').next() {
                // If the key has a tenant prefix and it doesn't match, deny.
                if !prefix.is_empty() && prefix != tenant_id.as_str() && key.contains('/') {
                    return PolicyDecision::Deny(format!(
                        "tenant '{}' cannot access resource with prefix '{}'",
                        tenant_id, prefix
                    ));
                }
            }
        }

        PolicyDecision::Permit
    }
}

/// Requires approval when an Intent's declared args cross a numeric threshold
/// on some named field (demo policy; real deployments compose multiple).
pub struct ThresholdApprovalPolicy {
    pub tool: &'static str,
    pub field: &'static str,
    pub max_before_approval: i64,
    pub channel: &'static str,
}

impl Policy for ThresholdApprovalPolicy {
    fn name(&self) -> &'static str {
        "threshold.approval"
    }
    fn applies_to(&self, intent: &Intent) -> bool {
        intent.body.target == self.tool
    }
    fn evaluate(&self, intent: &Intent, _writ: &Writ, _world: &World) -> PolicyDecision {
        let v = intent
            .body
            .args
            .get(self.field)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        if v > self.max_before_approval {
            PolicyDecision::RequireApproval {
                channel: self.channel.to_string(),
                reason: format!(
                    "{} = {} exceeds threshold {}",
                    self.field, v, self.max_before_approval
                ),
            }
        } else {
            PolicyDecision::Permit
        }
    }
}
