use serde_derive::{Serialize, Deserialize};

/// Token object, using 4098 bytes for paranoia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token(Vec<u8>);

/// Repesents an ammount of money.
/// Limited to u32 due to Redis and Lua 5.1
pub type Balance = u32;

/// Represents a money transfer between two users
/// /// Represents a money transfer between two users..
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    source:  String,
    target:  String,
    ammount: Balance
}

use rocket_contrib::json::Json;
use rocket::Response;
use rocket::http::{Status, ContentType};

mod objs;
use objs::*;

#[get("/")]
pub fn home<'a>() -> Response<'a> {
	use std::io::Cursor;
	Response::build()
		.status(Status::Found)
		.header(ContentType::HTML)
		.raw_header("Location", "https://www.youtube.com/watch?v=NF26ZyZRJbU")
		.sized_body(Cursor::new(include_str!("home.html")))
		.finalize()
}

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

#[post("/reg/deposit", format = "json", data = "<param>")]
pub fn deposit(param: Json<DepositRequest>) -> Json<DepositResponse> {
    unimplemented!()
}

pub fn routes() -> Vec<rocket::Route> {
	routes![home, login, drop, transfer, history, deposit]
}


