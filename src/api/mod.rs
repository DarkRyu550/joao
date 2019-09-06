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
use rocket::{Response, State};
use rocket::request::{Request, FromRequest, Outcome};
use rocket::http::{Status, ContentType};
use crate::state;
use crate::db;

mod objs;
use objs::*;

enum ActorError {
	UnknownUser,
	InvalidSignature,
}

struct Actor;
impl FromRequest for Actor {
	type Error = ActorError;
	fn from_request(req: &Request) -> Outcome<Self, Self::Error> {
		req.guard::<State<state::Server>>()
			.and_then(|state| {
				let conn = (*state).db_conn.borrow();
				match db::validate(&mut *conn) {

				}
			});
	}
}

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
pub fn drop(user: User) -> Json<DropResponse> {
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


