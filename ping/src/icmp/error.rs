use thiserror::Error;
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("Invaild size")]
    InvalidSize,

    #[error("Invalid packet")]
    InvalidPacket,
}
pub type DecodeResult<T> = Result<T, DecodeError>;
