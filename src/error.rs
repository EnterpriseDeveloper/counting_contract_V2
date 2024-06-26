use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized - only {owner} can it")]
    Unauthorized { owner: String },

    #[error("Migration invalid contract: {0}")]
    InvalidName(String),

    #[error("Migration from unsupported version: {0}")]
    InvalidVersion(String),
}
