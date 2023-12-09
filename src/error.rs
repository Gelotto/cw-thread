use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("NotAuthorized: {reason:?}")]
    NotAuthorized { reason: String },

    #[error("NodeNotFound: node {node_id:?} not found")]
    NodeNotFound { node_id: u32 },

    #[error("ValidationError: {reason:?}")]
    ValidationError { reason: String },
}

impl From<ContractError> for StdError {
    fn from(err: ContractError) -> Self {
        StdError::generic_err(err.to_string())
    }
}
