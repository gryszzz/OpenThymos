//! Thymos runtime orchestration.
//!
//! Wires the Cognition Gateway, Compiler, Tool Gateway, Policy Engine, and
//! Ledger into the IPC (Intent → Proposal → Commit) cycle.
//!
//! Phase 1 is synchronous and single-agent. The runtime owns a fresh `World`
//! projection and rebuilds it from the ledger on `Run::resume`.

use thymos_compiler::{compile_with_context, CompileContext, Compiled};
use thymos_core::{
    commit::{Commit, CommitBody},
    error::{Error, Result},
    intent::Intent,
    proposal::RejectionReason,
    world::World,
    writ::BudgetCost,
    CommitId, TrajectoryId, COMPILER_VERSION,
};
use thymos_ledger::{project_commits, EntryPayload, Ledger};
use thymos_policy::PolicyEngine;
use thymos_tools::{ToolInvocation, ToolRegistry};

pub mod agent;
pub use agent::{run_agent, AgentRunOptions, AgentRunSummary, Termination};

#[cfg(feature = "async")]
pub mod agent_async;
#[cfg(feature = "async")]
pub use agent_async::run_agent_streaming;

pub struct Runtime {
    pub ledger: Ledger,
    pub tools: ToolRegistry,
    pub policy: PolicyEngine,
}

impl Runtime {
    pub fn new(ledger: Ledger, tools: ToolRegistry, policy: PolicyEngine) -> Self {
        Runtime {
            ledger,
            tools,
            policy,
        }
    }

    /// Create a new trajectory and return a Run bound to it.
    pub fn create_run(&self, note: &str, seed: &[u8]) -> Result<Run<'_>> {
        let trajectory_id = TrajectoryId::new_from_seed(seed);
        self.ledger.append_root(trajectory_id, note)?;
        Ok(Run {
            runtime: self,
            trajectory_id,
        })
    }

    /// Resume an existing trajectory. The Run picks up where it left off;
    /// world projection will fold every commit already in the ledger. Returns
    /// an error if the trajectory hasn't been rooted yet.
    pub fn resume_run(&self, trajectory_id: TrajectoryId) -> Result<Run<'_>> {
        if !self.ledger.has_trajectory(trajectory_id) {
            return Err(Error::Ledger(format!(
                "trajectory {:?} does not exist",
                trajectory_id
            )));
        }
        Ok(Run {
            runtime: self,
            trajectory_id,
        })
    }
}

pub struct Run<'a> {
    runtime: &'a Runtime,
    trajectory_id: TrajectoryId,
}

/// The result of submitting one Intent to the runtime.
#[derive(Debug)]
pub enum Step {
    Committed(CommitId),
    Rejected(RejectionReason),
    /// Policy returned RequireApproval; the proposal is reified in the ledger.
    Suspended {
        channel: String,
        reason: String,
    },
    /// A delegation was executed — child ran to completion.
    Delegated {
        child_trajectory_id: TrajectoryId,
        final_answer: Option<String>,
    },
}

impl<'a> Run<'a> {
    pub fn trajectory_id(&self) -> TrajectoryId {
        self.trajectory_id
    }

    /// Accessor for the enclosing runtime. Used by the agent loop to reach
    /// the ledger for observation lookup.
    pub fn runtime(&self) -> &Runtime {
        self.runtime
    }

    /// Reconstruct the World projection by folding the ledger for this
    /// trajectory up to the current head. For branched trajectories, first
    /// folds the ancestor chain up to the branch point, then this trajectory's
    /// own commits on top.
    pub fn project_world(&self) -> Result<World> {
        let entries = self.runtime.ledger.entries(self.trajectory_id)?;

        // Check if this is a branch. If so, recursively fold the ancestor.
        let mut world = if let Some(entry) = entries.first() {
            if let EntryPayload::Branch {
                source_trajectory_id,
                source_commit_id,
                ..
            } = &entry.payload
            {
                project_world_up_to(
                    &self.runtime.ledger,
                    *source_trajectory_id,
                    Some(*source_commit_id),
                )?
            } else {
                World::default()
            }
        } else {
            World::default()
        };

        let commits = project_commits(&entries);
        for c in commits {
            world.apply(&c.body.delta, c.id)?;
        }
        Ok(world)
    }

    /// Create a new trajectory branched from a specific commit in this
    /// trajectory. The new Run starts with the world state as of that commit.
    pub fn branch_from(&self, commit_id: CommitId, note: &str) -> Result<Run<'_>> {
        let seed = format!("branch-{}-{}", self.trajectory_id, commit_id);
        let new_traj = TrajectoryId::new_from_seed(seed.as_bytes());
        self.runtime
            .ledger
            .append_branch_root(new_traj, self.trajectory_id, commit_id, note)?;
        Ok(Run {
            runtime: self.runtime,
            trajectory_id: new_traj,
        })
    }

    /// Project accumulated budget usage for this trajectory by summing
    /// `budget_cost` fields across all committed entries.
    pub fn project_budget_used(&self) -> Result<BudgetCost> {
        let entries = self.runtime.ledger.entries(self.trajectory_id)?;
        let mut acc = BudgetCost::default();
        for e in &entries {
            if let EntryPayload::Commit(c) = &e.payload {
                acc = acc.saturating_add(&c.body.budget_cost);
            }
        }
        Ok(acc)
    }

    /// Submit one Intent. Runs it through the full Triad.
    pub fn submit(&self, intent: Intent, writ: &thymos_core::writ::Writ) -> Result<Step> {
        #[cfg(feature = "telemetry")]
        let _span = tracing::info_span!(
            "triad.submit",
            tool = %intent.body.target,
            kind = ?intent.body.kind,
            trajectory = %self.trajectory_id,
        )
        .entered();

        // Fold world.
        let world = self.project_world()?;

        // Project budget usage for the compile context.
        let budget_used = self.project_budget_used()?;
        let ctx = CompileContext {
            budget_used,
            ..CompileContext::default()
        };

        // Compile (with budget + time-window checks).
        #[cfg(feature = "telemetry")]
        let _compile_span = tracing::info_span!("triad.compile").entered();

        let compiled = compile_with_context(
            &intent,
            writ,
            &world,
            &self.runtime.tools,
            &self.runtime.policy,
            &ctx,
        )?;

        #[cfg(feature = "telemetry")]
        drop(_compile_span);

        match compiled {
            Compiled::Rejected(reason) => {
                self.runtime.ledger.append_rejection(
                    self.trajectory_id,
                    intent.id,
                    reason.clone(),
                )?;
                Ok(Step::Rejected(reason))
            }
            Compiled::Suspended {
                proposal,
                channel,
                reason,
            } => {
                // Reify the pending approval in the ledger so it survives restarts.
                self.runtime.ledger.append_pending_approval(
                    self.trajectory_id,
                    proposal,
                    channel.clone(),
                    reason.clone(),
                )?;
                Ok(Step::Suspended { channel, reason })
            }
            Compiled::Staged(proposal) => {
                // Intercept delegation: spawn a child trajectory instead of
                // executing a tool.
                if proposal.body.plan.tool == "delegate" {
                    return self.execute_delegation(&proposal, writ);
                }

                let tool = self.runtime.tools.get(&proposal.body.plan.tool)?;

                // Pre-compute estimated cost for the commit record.
                let estimated_cost = tool.estimate_cost(&proposal.body.plan.args);

                let inv = ToolInvocation {
                    args: &proposal.body.plan.args,
                    world: &world,
                };

                #[cfg(feature = "telemetry")]
                let _exec_span = tracing::info_span!(
                    "triad.execute",
                    tool = %proposal.body.plan.tool,
                )
                .entered();

                let outcome = tool
                    .execute(&inv)
                    .map_err(|e| Error::ToolExecution(e.to_string()))?;

                // Verify postconditions (contract-declared).
                tool.check_postconditions(&inv, &outcome.delta)?;

                #[cfg(feature = "telemetry")]
                {
                    tracing::info!(
                        latency_ms = outcome.observation.latency_ms,
                        delta_ops = outcome.delta.0.len(),
                        "tool executed"
                    );
                    drop(_exec_span);
                }

                // Look up parent head for the commit.
                let (parent_hash, parent_seq) = self.runtime.ledger.head(self.trajectory_id)?;

                // Trial-apply the delta to make sure it would commit cleanly.
                let mut trial = world.clone();
                trial.apply(&outcome.delta, CommitId(parent_hash))?;

                // Record actual latency into the budget cost.
                let budget_cost = BudgetCost {
                    wall_clock_ms: outcome.observation.latency_ms,
                    ..estimated_cost
                };

                #[cfg(feature = "telemetry")]
                let _commit_span =
                    tracing::info_span!("triad.commit", seq = parent_seq + 1).entered();

                let commit_body = CommitBody {
                    parent: vec![CommitId(parent_hash)],
                    trajectory_id: self.trajectory_id,
                    proposal_id: proposal.id,
                    writ_id: writ.id,
                    seq: parent_seq + 1,
                    delta: outcome.delta,
                    observations: vec![outcome.observation],
                    compiler_version: COMPILER_VERSION.into(),
                    budget_cost,
                    signature: None,
                };
                let commit = Commit::new(commit_body)?;
                let committed_id = CommitId(commit.id.0);

                self.runtime.ledger.append_commit(commit)?;

                #[cfg(feature = "telemetry")]
                tracing::info!(commit_id = %committed_id, "committed");

                Ok(Step::Committed(committed_id))
            }
        }
    }

    /// Execute a delegation: mint a child writ, create a child trajectory,
    /// record the delegation edge. Returns `Step::Delegated`. The child
    /// trajectory is created but not driven — the caller (agent loop) is
    /// responsible for providing cognition for the child.
    fn execute_delegation(
        &self,
        proposal: &thymos_core::proposal::Proposal,
        parent_writ: &thymos_core::writ::Writ,
    ) -> Result<Step> {
        use thymos_core::writ::{ToolPattern, WritBody};

        let args = &proposal.body.plan.args;
        let child_task = args
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("delegated task")
            .to_string();

        // Extract tool_scopes from args (optional; defaults to parent scopes).
        let child_scopes: Vec<ToolPattern> = args
            .get("tool_scopes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| ToolPattern::exact(s)))
                    .collect()
            })
            .unwrap_or_else(|| parent_writ.body.tool_scopes.clone());

        // Mint a child writ. Budget is halved from parent's remaining.
        let child_budget = thymos_core::writ::Budget {
            tokens: parent_writ.body.budget.tokens / 2,
            tool_calls: parent_writ.body.budget.tool_calls / 2,
            wall_clock_ms: parent_writ.body.budget.wall_clock_ms / 2,
            usd_millicents: parent_writ.body.budget.usd_millicents / 2,
        };

        // Generate a child key (the child subject becomes the child's issuer
        // for further delegation).
        let child_key = thymos_core::crypto::generate_signing_key();

        let _child_body = WritBody {
            issuer: parent_writ.body.subject.clone(),
            issuer_pubkey: parent_writ.body.subject_pubkey,
            subject: format!("{}-child", parent_writ.body.subject),
            subject_pubkey: thymos_core::crypto::public_key_of(&child_key),
            parent: None,
            tenant_id: parent_writ.body.tenant_id.clone(),
            tool_scopes: child_scopes,
            budget: child_budget,
            effect_ceiling: parent_writ.body.effect_ceiling.clone(),
            time_window: parent_writ.body.time_window.clone(),
            delegation: thymos_core::writ::DelegationBounds {
                max_depth: parent_writ.body.delegation.max_depth.saturating_sub(1),
                may_subdivide: parent_writ.body.delegation.may_subdivide,
            },
        };

        // We need the parent subject's signing key to mint the child. Since
        // we don't have it here (the runtime only has the writ, not the key),
        // we record the delegation edge as pending — the child writ is noted
        // but its execution must be driven externally.
        //
        // For Phase 1: create the child trajectory, record the delegation
        // edge, and return the child trajectory id so the agent loop can
        // drive it.
        let child_seed = format!("delegate-{}-{}", child_task, proposal.id);
        let child_traj = TrajectoryId::new_from_seed(child_seed.as_bytes());
        self.runtime
            .ledger
            .append_root(child_traj, &format!("delegated: {}", child_task))?;

        // Record the delegation edge in the parent trajectory.
        self.runtime
            .ledger
            .append_delegation(self.trajectory_id, child_traj, &child_task, None)?;

        Ok(Step::Delegated {
            child_trajectory_id: child_traj,
            final_answer: None,
        })
    }

    /// Resume a previously suspended proposal. If `approve` is true, the
    /// proposal is executed through the tool and committed. If false, it's
    /// rejected as PolicyDenied.
    pub fn resume_with_approval(
        &self,
        proposal_id: thymos_core::ProposalId,
        approve: bool,
        writ: &thymos_core::writ::Writ,
    ) -> Result<Step> {
        // Find the PendingApproval entry for this proposal.
        let entries = self.runtime.ledger.entries(self.trajectory_id)?;
        let pending = entries.iter().find_map(|e| {
            if let EntryPayload::PendingApproval { proposal, .. } = &e.payload {
                if proposal.id == proposal_id {
                    return Some(proposal.clone());
                }
            }
            None
        });
        let proposal = pending.ok_or_else(|| {
            Error::Other(format!(
                "no pending approval for proposal {:?}",
                proposal_id
            ))
        })?;

        if !approve {
            self.runtime.ledger.append_rejection(
                self.trajectory_id,
                proposal.body.intent_id,
                RejectionReason::PolicyDenied("approval denied by operator".into()),
            )?;
            return Ok(Step::Rejected(RejectionReason::PolicyDenied(
                "approval denied by operator".into(),
            )));
        }

        // Approved: re-execute the tool against the current world.
        let world = self.project_world()?;
        let tool = self.runtime.tools.get(&proposal.body.plan.tool)?;
        let estimated_cost = tool.estimate_cost(&proposal.body.plan.args);

        let inv = ToolInvocation {
            args: &proposal.body.plan.args,
            world: &world,
        };
        let outcome = tool
            .execute(&inv)
            .map_err(|e| Error::ToolExecution(e.to_string()))?;
        tool.check_postconditions(&inv, &outcome.delta)?;

        let (parent_hash, parent_seq) = self.runtime.ledger.head(self.trajectory_id)?;

        let mut trial = world.clone();
        trial.apply(&outcome.delta, CommitId(parent_hash))?;

        let budget_cost = BudgetCost {
            wall_clock_ms: outcome.observation.latency_ms,
            ..estimated_cost
        };
        let commit_body = CommitBody {
            parent: vec![CommitId(parent_hash)],
            trajectory_id: self.trajectory_id,
            proposal_id: proposal.id,
            writ_id: writ.id,
            seq: parent_seq + 1,
            delta: outcome.delta,
            observations: vec![outcome.observation],
            compiler_version: COMPILER_VERSION.into(),
            budget_cost,
            signature: None,
        };
        let commit = Commit::new(commit_body)?;
        let committed_id = CommitId(commit.id.0);
        self.runtime.ledger.append_commit(commit)?;
        Ok(Step::Committed(committed_id))
    }

    /// Summarize the trajectory for debugging/demo output.
    pub fn summary(&self) -> Result<TrajectorySummary> {
        let entries = self.runtime.ledger.entries(self.trajectory_id)?;
        let mut commits = 0usize;
        let mut rejections = 0usize;
        let mut roots = 0usize;
        let mut pending_approvals = 0usize;
        for e in &entries {
            match e.kind {
                thymos_ledger::EntryKind::Root => roots += 1,
                thymos_ledger::EntryKind::Commit => commits += 1,
                thymos_ledger::EntryKind::Rejection => rejections += 1,
                thymos_ledger::EntryKind::PendingApproval => pending_approvals += 1,
                thymos_ledger::EntryKind::Delegation => {}
                thymos_ledger::EntryKind::Branch => {}
            }
        }
        self.runtime.ledger.verify_integrity(self.trajectory_id)?;
        Ok(TrajectorySummary {
            entries_total: entries.len(),
            roots,
            commits,
            rejections,
            pending_approvals,
            entries,
        })
    }
}

/// Project world state for a trajectory, optionally stopping at a specific
/// commit (inclusive). Handles recursive ancestor chains for branched
/// trajectories.
fn project_world_up_to(
    ledger: &Ledger,
    trajectory_id: TrajectoryId,
    up_to: Option<CommitId>,
) -> Result<World> {
    let entries = ledger.entries(trajectory_id)?;

    // Recurse into ancestor if this is a branch.
    let mut world = if let Some(entry) = entries.first() {
        if let EntryPayload::Branch {
            source_trajectory_id,
            source_commit_id,
            ..
        } = &entry.payload
        {
            project_world_up_to(ledger, *source_trajectory_id, Some(*source_commit_id))?
        } else {
            World::default()
        }
    } else {
        World::default()
    };

    let commits = project_commits(&entries);
    for c in commits {
        world.apply(&c.body.delta, c.id)?;
        if up_to == Some(c.id) {
            break;
        }
    }
    Ok(world)
}

pub struct TrajectorySummary {
    pub entries_total: usize,
    pub roots: usize,
    pub commits: usize,
    pub rejections: usize,
    pub pending_approvals: usize,
    pub entries: Vec<thymos_ledger::Entry>,
}

// Convenience re-exports for example code.
pub use thymos_core::{
    crypto::{generate_signing_key, public_key_of},
    delta::{DeltaOp, StructuredDelta as Delta},
    intent::{Intent as CoreIntent, IntentBody, IntentKind},
    proposal::{PolicyDecision, RejectionReason as CoreRejectionReason},
    world::{ResourceKey, World as CoreWorld},
    writ::{Budget, DelegationBounds, EffectCeiling, TimeWindow, ToolPattern, Writ, WritBody},
};
