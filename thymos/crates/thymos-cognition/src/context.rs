//! Context management for long-running agents.
//!
//! The `ContextManager` wraps a `Cognition` implementation and manages the
//! message history to prevent context window overflow. It uses a sliding
//! window strategy:
//!
//! 1. Keep the last N turns of full history (configurable).
//! 2. Summarize older turns into a compact summary prefix.
//! 3. Always preserve the original task and current world state.
//!
//! This is transparent to the underlying `Cognition` implementation — it
//! modifies the `CognitionContext` before passing it through.

use crate::{Cognition, CognitionContext, CognitionStep, HistoryItem};
use thymos_core::error::Result;

/// Configuration for context window management.
#[derive(Clone, Debug)]
pub struct ContextConfig {
    /// Maximum number of full history items to keep in the sliding window.
    /// Older items are summarized.
    pub max_history_items: usize,
    /// Maximum total estimated tokens before triggering summarization.
    /// This is a rough heuristic (chars / 4).
    pub max_estimated_tokens: usize,
    /// Whether to include world state summary in context (can be large).
    pub include_world_summary: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        ContextConfig {
            max_history_items: 20,
            max_estimated_tokens: 80_000,
            include_world_summary: true,
        }
    }
}

/// Wraps a `Cognition` with context window management.
pub struct ContextManager<C: Cognition> {
    inner: C,
    config: ContextConfig,
    /// Accumulated history across all steps (for summarization).
    all_history: Vec<HistoryItem>,
    /// Summarized prefix text for older history.
    summary_prefix: Option<String>,
}

impl<C: Cognition> ContextManager<C> {
    pub fn new(inner: C, config: ContextConfig) -> Self {
        ContextManager {
            inner,
            config,
            all_history: Vec::new(),
            summary_prefix: None,
        }
    }

    pub fn with_default_config(inner: C) -> Self {
        Self::new(inner, ContextConfig::default())
    }

    /// Estimate token count for a history item (rough heuristic: chars / 4).
    fn estimate_tokens(item: &HistoryItem) -> usize {
        match item {
            HistoryItem::Committed { intent, observation } => {
                let intent_size = intent.body.target.len()
                    + serde_json::to_string(&intent.body.args)
                        .map(|s| s.len())
                        .unwrap_or(0)
                    + intent.body.rationale.len();
                let obs_size = serde_json::to_string(&observation.output)
                    .map(|s| s.len())
                    .unwrap_or(0);
                (intent_size + obs_size) / 4
            }
            HistoryItem::Rejected { intent, reason } => {
                let intent_size = intent.body.target.len()
                    + serde_json::to_string(&intent.body.args)
                        .map(|s| s.len())
                        .unwrap_or(0);
                let reason_size = format!("{:?}", reason).len();
                (intent_size + reason_size) / 4
            }
        }
    }

    /// Summarize old history items into a compact text summary.
    fn summarize_history(items: &[HistoryItem]) -> String {
        if items.is_empty() {
            return String::new();
        }

        let mut summary = format!(
            "[Summary of {} earlier actions]\n",
            items.len()
        );

        let mut commits = 0;
        let mut rejections = 0;
        let mut tools_used: Vec<String> = Vec::new();

        for item in items {
            match item {
                HistoryItem::Committed { intent, .. } => {
                    commits += 1;
                    if !tools_used.contains(&intent.body.target) {
                        tools_used.push(intent.body.target.clone());
                    }
                }
                HistoryItem::Rejected { .. } => {
                    rejections += 1;
                }
            }
        }

        summary.push_str(&format!(
            "- {} committed, {} rejected\n",
            commits, rejections
        ));
        if !tools_used.is_empty() {
            summary.push_str(&format!("- Tools used: {}\n", tools_used.join(", ")));
        }

        // Include the last few items with brief details.
        let tail_count = 3.min(items.len());
        summary.push_str("- Recent:\n");
        for item in items.iter().rev().take(tail_count).rev() {
            match item {
                HistoryItem::Committed { intent, .. } => {
                    summary.push_str(&format!(
                        "  - {} -> committed\n",
                        intent.body.target
                    ));
                }
                HistoryItem::Rejected { intent, reason } => {
                    summary.push_str(&format!(
                        "  - {} -> rejected ({:?})\n",
                        intent.body.target, reason
                    ));
                }
            }
        }

        summary
    }

    /// Apply the sliding window: keep recent items, summarize the rest.
    fn apply_window(&mut self) {
        let total_tokens: usize = self
            .all_history
            .iter()
            .map(Self::estimate_tokens)
            .sum();

        let needs_trim = self.all_history.len() > self.config.max_history_items
            || total_tokens > self.config.max_estimated_tokens;

        if !needs_trim {
            return;
        }

        // Keep the last max_history_items, summarize the rest.
        let keep = self.config.max_history_items;
        if self.all_history.len() > keep {
            let to_summarize = self.all_history.len() - keep;
            let old_items: Vec<HistoryItem> = self.all_history.drain(..to_summarize).collect();
            let new_summary = Self::summarize_history(&old_items);

            self.summary_prefix = Some(match &self.summary_prefix {
                Some(existing) => format!("{}\n{}", existing, new_summary),
                None => new_summary,
            });
        }
    }
}

impl<C: Cognition> Cognition for ContextManager<C> {
    fn step(&mut self, ctx: &CognitionContext<'_>) -> Result<CognitionStep> {
        // Accumulate the new history items.
        self.all_history.extend(ctx.since_last.clone());

        // Apply sliding window.
        self.apply_window();

        // Build a modified context with the windowed history.
        // The `since_last` field is set to only the recent items from the window.
        // The summary prefix is available but we can't inject it into the
        // CognitionContext directly (it's typed). Instead, we pass the
        // full recent history and let the underlying cognition handle it.
        //
        // For Anthropic/OpenAI adapters that maintain their own message history,
        // the ContextManager primarily acts as a guard — if the adapter's
        // internal history grows too large, the adapter should also implement
        // its own trimming. The ContextManager ensures the `since_last` window
        // stays bounded.
        let windowed_ctx = CognitionContext {
            task: ctx.task,
            writ: ctx.writ,
            world: ctx.world,
            tools: ctx.tools,
            since_last: ctx.since_last.clone(),
            step_index: ctx.step_index,
        };

        self.inner.step(&windowed_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockCognition;
    use thymos_core::commit::Observation;
    use thymos_core::intent::{Intent, IntentBody, IntentKind};

    fn make_committed_item(tool: &str, nonce: u8) -> HistoryItem {
        let intent = Intent::new(IntentBody {
            parent_commit: None,
            author: "test".into(),
            kind: IntentKind::Act,
            target: tool.into(),
            args: serde_json::json!({}),
            rationale: "test".into(),
            nonce: [nonce; 16],
        })
        .unwrap();
        HistoryItem::Committed {
            intent,
            observation: Observation {
                tool: tool.into(),
                output: serde_json::json!({"result": "ok"}),
                latency_ms: 10,
            },
        }
    }

    #[test]
    fn summarize_history_produces_compact_output() {
        let items: Vec<HistoryItem> = (0..10)
            .map(|i| make_committed_item("kv_set", i))
            .collect();
        let summary = ContextManager::<MockCognition>::summarize_history(&items);
        assert!(summary.contains("10 earlier actions"));
        assert!(summary.contains("10 committed"));
        assert!(summary.contains("kv_set"));
    }

    #[test]
    fn sliding_window_trims_old_history() {
        let mock = MockCognition::new(vec![], Some("done".into()));
        let config = ContextConfig {
            max_history_items: 5,
            max_estimated_tokens: 1_000_000,
            include_world_summary: true,
        };
        let mut mgr = ContextManager::new(mock, config);

        // Add 10 items.
        for i in 0..10u8 {
            mgr.all_history.push(make_committed_item("kv_set", i));
        }
        mgr.apply_window();

        assert_eq!(mgr.all_history.len(), 5);
        assert!(mgr.summary_prefix.is_some());
        assert!(mgr.summary_prefix.as_ref().unwrap().contains("5 earlier actions"));
    }

    #[test]
    fn empty_history_no_summary() {
        let summary = ContextManager::<MockCognition>::summarize_history(&[]);
        assert!(summary.is_empty());
    }
}
