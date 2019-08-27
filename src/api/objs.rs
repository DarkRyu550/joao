//! Objects related to requests and responses performed by the API.
use serde_derive::{Serialize, Deserialize};
use super::{Token, Balance, Transfer};

/* Login */
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    username: String,
    key: String
}

#[derive(Debug, Clone, Serialize)]
pub enum LoginError {
    /// The given username is invalid.
    MalformedUsername,
    /// Either username or password is wrong.
    WrongCombination,
    /// The current client is blocked from doing requests for the given period.
    Cooldown(std::time::Duration)
}
pub type LoginResponse = Result<Token, LoginError>;

/* Drop */
#[derive(Debug, Clone, Deserialize)]
pub struct DropRequest {
    username: String,
    token: Token
}
#[derive(Debug, Clone, Serialize)]
pub enum DropError {
    /// The given token is not valid.
    InvalidToken,
    /// The current client is blocked from doing requests for the given period.
    Cooldown(std::time::Duration)
}
pub type DropResponse = Result<(), DropError>;

/* Transfer */
#[derive(Debug, Clone, Deserialize)]
pub struct TransferRequest {
    
}
#[derive(Debug, Clone, Serialize)]
pub enum TransferError {
    /// The given token is not valid.
    InvalidToken,
    /// The given target user is not valid.
    InvalidTarget,
    /// The user is lacking in balance for this tranfer by this much.
    InsufficientBalance(Balance),
    /// The target can't receive the request as it has too much money.
    TooMuchMoney,
}
pub type TransferResponse = Result<(), TransferError>;


/* History */
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryRequest {

}
pub enum HistoryError {
    /// The given token is not valid.
    InvalidToken,
}
pub type HistoryResponse = Result<Vec<Tranfer>, HistoryError>;
