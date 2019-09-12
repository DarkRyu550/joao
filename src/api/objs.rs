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

/* Drop */
#[derive(Debug, Clone, Deserialize)]
pub struct DropRequest {
    username: String,
    token: Token
}

/* Transfer */
#[derive(Debug, Clone, Deserialize)]
pub struct TransferRequest {
    pub to: String,
    pub amount: u32
}

#[derive(Debug, Clone, Deserialize)]
pub struct WithdrawRequest {
    pub amount: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HistoryEntry {
    pub from: String,
    pub to: String,
    pub amount: u32
}

/* Deposit */
#[derive(Debug, Clone, Deserialize)]
pub struct DepositRequest {
    pub username: String,
    pub amount: u32
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminWithdrawRequest {
    pub username: String,
    pub amount: u32
}

/* Registration */
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
	pub username:  String,
	pub name:   String,
	pub key:    String,
}
