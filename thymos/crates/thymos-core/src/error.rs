use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serialization failure: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("invariant violated: {0}")]
    Invariant(String),

    #[error("authority void: {0}")]
    AuthorityVoid(String),

    #[error("precondition failed: {0}")]
    PreconditionFailed(String),

    #[error("postcondition failed: {0}")]
    PostconditionFailed(String),

    #[error("policy denied: {0}")]
    PolicyDenied(String),

    #[error("policy requires approval: {0}")]
    PolicyApprovalRequired(String),

    #[error("budget exhausted: {0}")]
    BudgetExhausted(String),

    #[error("unknown tool: {0}")]
    UnknownTool(String),

    #[error("type mismatch in tool contract '{tool}': {detail}")]
    ToolTypeMismatch { tool: String, detail: String },

    #[error("tool execution failed: {0}")]
    ToolExecution(String),

    #[error("ledger error: {0}")]
    Ledger(String),

    #[error("cas conflict on {kind}:{id} (expected v{expected}, found v{found})")]
    CasConflict {
        kind: String,
        id: String,
        expected: u64,
        found: u64,
    },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
