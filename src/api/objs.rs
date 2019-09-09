//! Objects related to requests and responses performed by the API.
use serde_derive::{Serialize, Deserialize};
use super::{Token, Balance, Transfer};
use rocket_contrib::json::JsonValue;

/* Login */
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub key: String
}

#[derive(Debug, Clone, Serialize)]
pub enum LoginError {
    /// Either username or password is wrong.
    WrongCombination,
    /// The current client is blocked from doing requests for the given period.
    Cooldown(std::time::Duration)
}

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
#[derive(Debug, Clone, Serialize)]
pub enum HistoryError {
    /// The given token is not valid.
    InvalidToken,
}
pub type HistoryResponse = Result<Vec<Transfer>, HistoryError>;


/* Deposit */
#[derive(Debug, Clone, Deserialize)]
pub struct DepositRequest {

}
#[derive(Debug, Clone, Serialize)]
pub enum DepositError {
    /// The given token is not valid.
    InvalidToken,
}
pub type DepositResponse = Result<(), DepositError>;

/* Registration */
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
	pub email:  String,
	pub name:   String,
	pub key:    String,
}
