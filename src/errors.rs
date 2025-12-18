use jsonrpsee::types::{error::INVALID_PARAMS_CODE, ErrorObjectOwned};

/// Errors related to transaction validation and processing.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransactionValidationError {
    /// The transaction failed to process.
    TransactionFailed,
    /// The transaction is missing from the request.
    TransactionMissing,
    /// The message is missing from the transaction.
    MessageMissing,
    /// An invalid account was provided.
    InvalidAccount,
}

impl From<TransactionValidationError> for &str {
    fn from(val: TransactionValidationError) -> Self {
        match val {
            TransactionValidationError::TransactionFailed => "txn_failed",
            TransactionValidationError::TransactionMissing => "txn_missing",
            TransactionValidationError::MessageMissing => "message_missing",
            TransactionValidationError::InvalidAccount => "invalid_pubkey",
        }
    }
}

/// Creates an invalid request error object with a specific reason.
pub fn invalid_request(reason: &str) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        INVALID_PARAMS_CODE,
        format!("Invalid Request: {reason}"),
        None::<String>,
    )
}
