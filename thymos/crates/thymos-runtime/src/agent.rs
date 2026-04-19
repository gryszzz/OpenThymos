//! Agent loop: drives a `Cognition` through the Thymos IPC Triad until
//! termination, budget exhaustion, or max steps.
//!
//! The loop's only job is:
//!   (1) build a fresh `CognitionContext` each step,
//!   (2) ask cognition for a batch of Intents,
//!   (3) submit each through the Triad, capturing typed outcomes,
//!   (4) accumulate those outcomes as `HistoryItem`s for the next step,
//!   (5) stop when cognition returns an empty step.
//!
//! All state lives in the ledger. The agent loop is stateless except for the
//! small window of `HistoryItem`s produced in the current step (which is
//! handed to cognition on the next one).

use thymos_cognition::{Cognition, CognitionContext, HistoryItem};
use thymos_core::{
    commit::Observation,
    error::{Error, Result},
    intent::Intent,
    writ::Writ,
    TrajectoryId,
};
use thymos_ledger::{Entry, EntryPayload};

use crate::{Run, Runtime, Step};

#[derive(Debug)]
pub struct AgentRunSummary {
    pub trajectory_id: TrajectoryId,
    pub steps_executed: u32,
    pub intents_submitted: u32,
    pub commits: u32,
    pub rejections: u32,
    pub final_answer: Option<String>,
    pub terminated_by: Termination,
}

#[derive(Debug)]
pub enum Termination {
    CognitionDone,
    MaxStepsReached,
    Suspended,
    WritExpired,
}

pub struct AgentRunOptions {
    pub max_steps: u32,
}

impl Default for AgentRunOptions {
    fn default() -> Self {
        // 4.7 handles longer tool-chained reasoning; the budget in the Writ
        // remains the authoritative cap, so this ceiling only guards runaway
        // cognition loops.
        AgentRunOptions { max_steps: 32 }
    }
}

/// Drive a Cognition to completion against the Thymos runtime.
///
/// The loop is explicit: cognition proposes, runtime decides, ledger remembers.
pub fn run_agent(
    runtime: &Runtime,
    cognition: &mut dyn Cognition,
    task: &str,
    writ: &Writ,
    opts: AgentRunOptions,
) -> Result<AgentRunSummary> {
    let run = runtime.create_run(task, task.as_bytes())?;
    let trajectory_id = run.trajectory_id();

    #[cfg(feature = "telemetry")]
    tracing::info!(
        %trajectory_id,
        max_steps = opts.max_steps,
        "agent run started"
    );

    let mut since_last: Vec<HistoryItem> = Vec::new();
    let mut steps_executed = 0u32;
    let mut intents_submitted = 0u32;
    let mut commits = 0u32;
    let mut rejections = 0u32;
    let mut final_answer: Option<String> = None;
    let mut terminated_by = Termination::MaxStepsReached;

    for step_idx in 0..opts.max_steps {
        #[cfg(feature = "telemetry")]
        let _step_span = tracing::info_span!("agent.step", step = step_idx).entered();

        let world = run.project_world()?;

        let step = cognition.step(&CognitionContext {
            task,
            writ,
            world: &world,
            tools: &runtime.tools,
            since_last: std::mem::take(&mut since_last),
            step_index: step_idx,
        })?;

        steps_executed += 1;

        if step.intents.is_empty() {
            final_answer = step.final_answer;
            terminated_by = Termination::CognitionDone;
            break;
        }

        for intent in step.intents {
            intents_submitted += 1;
            let result = run.submit(intent.clone(), writ)?;

            match result {
                Step::Committed(commit_id) => {
                    commits += 1;
                    let observation = last_observation(&run, commit_id)?;
                    since_last.push(HistoryItem::Committed {
                        intent,
                        observation,
                    });
                }
                Step::Rejected(reason) => {
                    rejections += 1;
                    since_last.push(HistoryItem::Rejected { intent, reason });
                }
                Step::Suspended { channel, reason } => {
                    terminated_by = Termination::Suspended;
                    return Ok(AgentRunSummary {
                        trajectory_id,
                        steps_executed,
                        intents_submitted,
                        commits,
                        rejections,
                        final_answer: Some(format!(
                            "suspended for approval on channel '{}': {}",
                            channel, reason
                        )),
                        terminated_by,
                    });
                }
                Step::Delegated {
                    child_trajectory_id,
                    final_answer: child_answer,
                } => {
                    // Surface the delegation result as a committed observation
                    // so cognition sees it on the next turn.
                    let observation = thymos_core::commit::Observation {
                        tool: "delegate".into(),
                        output: serde_json::json!({
                            "child_trajectory_id": child_trajectory_id.to_string(),
                            "final_answer": child_answer,
                        }),
                        latency_ms: 0,
                    };
                    since_last.push(HistoryItem::Committed {
                        intent,
                        observation,
                    });
                }
            }
        }
    }

    #[cfg(feature = "telemetry")]
    tracing::info!(
        %trajectory_id,
        steps_executed,
        intents_submitted,
        commits,
        rejections,
        terminated_by = ?terminated_by,
        "agent run finished"
    );

    Ok(AgentRunSummary {
        trajectory_id,
        steps_executed,
        intents_submitted,
        commits,
        rejections,
        final_answer,
        terminated_by,
    })
}

/// Fetch the Observation from the commit that just landed. The ledger is the
/// source of truth — we don't trust cached values.
fn last_observation(run: &Run<'_>, commit_id: thymos_core::CommitId) -> Result<Observation> {
    let entries: Vec<Entry> = run.runtime().ledger.entries(run.trajectory_id())?;
    for e in entries.into_iter().rev() {
        if let EntryPayload::Commit(c) = &e.payload {
            if c.id == commit_id {
                return c
                    .body
                    .observations
                    .first()
                    .cloned()
                    .ok_or_else(|| Error::Other("commit has no observation".into()));
            }
        }
    }
    Err(Error::Other(format!(
        "commit {:?} not found in trajectory",
        commit_id
    )))
}

// Convenience: so that `Intent` equality in history is cheap for debugging.
#[allow(dead_code)]
fn _intent_eq(a: &Intent, b: &Intent) -> bool {
    a.id == b.id
}
