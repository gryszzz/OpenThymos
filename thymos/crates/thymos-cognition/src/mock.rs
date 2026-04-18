//! Deterministic mock cognition for tests and offline demos.
//!
//! `MockCognition` replays a scripted sequence of Intent batches. Each call to
//! `step` returns the next batch. When the script is exhausted the gateway
//! returns an empty step with an optional `final_answer`.

use thymos_core::{error::Result, intent::Intent};

use crate::{Cognition, CognitionContext, CognitionStep};

pub struct MockCognition {
    script: std::vec::IntoIter<Vec<Intent>>,
    final_answer: Option<String>,
}

impl MockCognition {
    pub fn new(script: Vec<Vec<Intent>>, final_answer: Option<String>) -> Self {
        MockCognition {
            script: script.into_iter(),
            final_answer,
        }
    }
}

impl Cognition for MockCognition {
    fn step(&mut self, _ctx: &CognitionContext<'_>) -> Result<CognitionStep> {
        match self.script.next() {
            Some(intents) => Ok(CognitionStep {
                intents,
                final_answer: None,
            }),
            None => Ok(CognitionStep {
                intents: vec![],
                final_answer: self.final_answer.take(),
            }),
        }
    }
}
