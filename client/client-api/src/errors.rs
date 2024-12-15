use std::error::Error as StdError;
use std::io;
use thiserror::Error;

use solana_rpc_client_api::client_error::ErrorKind as RpcClientErrorKind;
use solana_rpc_client_api::request::RpcError;
use solana_sdk::pubkey::Pubkey;

pub use solana_banks_client::BanksClientError;
pub use solana_rpc_client_api::client_error::Error as RpcClientError;
pub use solana_sdk::address_lookup_table::error::AddressLookupError;
pub use solana_sdk::message::CompileError;
pub use solana_sdk::signer::SignerError;
pub use solana_sdk::transaction::TransactionError;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("An account {0} already exists")]
    AccountAlreadyExists(Pubkey),
    #[error("An account {0} was not found")]
    AccountNotFound(Pubkey),
    #[error("Failed to deserialize the account {0}")]
    AccountDidNotDeserialize(Pubkey),
    #[error("Failed to serialize the account {0}")]
    AccountDidNotSerialize(Pubkey),
    #[error(transparent)]
    CompileError(#[from] CompileError),
    #[error(transparent)]
    SigningError(#[from] SignerError),
    #[error(transparent)]
    AddressLookupError(#[from] AddressLookupError),
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error(transparent)]
    ClientSpecific(#[from] ClientSpecificError),
    #[error("domain specific error: {0}")]
    DomainSpecific(Box<dyn StdError + Send + Sync>),
}

impl ClientError {
    pub fn get_transaction_error(&self) -> Option<TransactionError> {
        match self {
            Self::TransactionError(e) => Some(e.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientSpecificError {
    #[error("banks client specific error: {0}")]
    BanksClient(#[from] BanksClientSpecificError),
    #[error("rpc client specific error: {0}")]
    RpcClient(#[from] RpcClientSpecificError),
}

#[derive(Debug, Error)]
pub enum BanksClientSpecificError {
    #[error("client error: {0}")]
    ClientError(&'static str),
    #[error(transparent)]
    Io(io::Error),
    #[error(transparent)]
    RpcError(tarpc::client::RpcError),
}

impl From<BanksClientSpecificError> for ClientError {
    fn from(error: BanksClientSpecificError) -> Self {
        ClientSpecificError::from(error).into()
    }
}

impl From<BanksClientError> for ClientError {
    fn from(error: BanksClientError) -> Self {
        match error {
            BanksClientError::ClientError(e) => BanksClientSpecificError::ClientError(e).into(),
            BanksClientError::Io(e) => BanksClientSpecificError::Io(e).into(),
            BanksClientError::RpcError(e) => BanksClientSpecificError::RpcError(e).into(),
            BanksClientError::TransactionError(e) => Self::TransactionError(e),
            BanksClientError::SimulationError { .. } => unreachable!("preflight"),
        }
    }
}

#[derive(Debug, Error)]
pub enum RpcClientSpecificError {
    #[error(transparent)]
    Io(io::Error),
    #[error(transparent)]
    Reqwest(reqwest::Error),
    #[error(transparent)]
    RpcError(RpcError),
    #[error(transparent)]
    SerdeJson(serde_json::error::Error),
    #[error("Custom: {0}")]
    Custom(String),
}

impl From<RpcClientSpecificError> for ClientError {
    fn from(error: RpcClientSpecificError) -> Self {
        ClientSpecificError::from(error).into()
    }
}

impl From<RpcClientError> for ClientError {
    fn from(error: RpcClientError) -> Self {
        match error.kind {
            RpcClientErrorKind::Io(e) => RpcClientSpecificError::Io(e).into(),
            RpcClientErrorKind::Reqwest(e) => RpcClientSpecificError::Reqwest(e).into(),
            RpcClientErrorKind::RpcError(e) => RpcClientSpecificError::RpcError(e).into(),
            RpcClientErrorKind::SerdeJson(e) => RpcClientSpecificError::SerdeJson(e).into(),
            RpcClientErrorKind::SigningError(e) => Self::SigningError(e),
            RpcClientErrorKind::TransactionError(e) => Self::TransactionError(e),
            RpcClientErrorKind::Custom(e) => RpcClientSpecificError::Custom(e).into(),
        }
    }
}

pub type ClientResult<T> = Result<T, ClientError>;
