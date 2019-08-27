use serde_derive::{Serialize, Deserialize};

/// Token object, using 4098 bytes for paranoia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token([u8; 4098]);

/// Repesents an ammount of money.
/// Limited to u32 due to Redis and Lua 5.1
pub type Balance = u32;

/// Represents a money transfer between two users
/// /// Represents a money transfer between two users..
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tranfer {
    source:  String,
    target:  String,
    ammount: Balance
}

use rocket_contrib::Json;
use objs::*;

#[post("/login", format = "json", data = "<param>")]
pub fn login(param: Json<LoginRequest>) -> Json<LoginResponse> {
    unimplemented!()
}

#[post("/drop", format = "json", data = "<param>")]
pub fn drop(param: Json<DropRequest>) -> Json<DropResponse> {
    unimplemented!()
}

#[post("/transfer", format = "json", data = "<param>")]
pub fn transfer(param: Json<TransferRequest>) -> Json<TransferResponse> {
    unimplemented!()
}

#[get("/history", format = "json", data = "<param>")]
pub fn history(param: Json<HistoryRequest>) -> Json<HistoryResponse> {
    unimplemented!()
}

#[post("/reg/deposit"), format = "json", data = "<param>"]
pub fn deposit(param: Json<DepositRequest>) -> Json<DepositResponse> {
    unimplemented!()
}



