use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("data store disconnected")]
    SolanaPubsubClientError(#[from] solana_client::pubsub_client::PubsubClientError),
    #[error("unknown error")]
    Unknown,
    #[error("Account doesn't exists")]
    AccountDoesntExist,
    #[error("Error while decoding account")]
    AccountDecodingError,
}
