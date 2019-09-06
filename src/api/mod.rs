use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    username: String,
    is_admin: bool
}

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

use super::settings::Auth;

use rocket_contrib::json::Json;
use rocket::{Response, State};
use rocket::request::{Request, FromRequest, Outcome};
use rocket::http::{Status, ContentType};
use crate::state;
use crate::db;

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
pub fn login(server: State<state::Server>,
             param: Json<LoginRequest>) -> LoginResponse {
    use jwt::{Header, encode};

    let srv: &state::Server = &server;
    let mut conn = srv.db_conn.borrow();

    let valid = db::validate(&mut *conn, param.0.username.clone(), param.0.key)
        .map_err(|_| json!({ "success": false, "error": "internal server error" }))?;
    if !valid {
        return Err(json!({ "success": false, "error": "invalid username or password" }));
    }

    let admin = db::is_admin(&mut *conn, param.0.username.clone())
        .map_err(|_| json!({ "success": false, "error": "internal server error" }))?;

    let auth = &srv.settings.auth;
    let token = encode(&Header::new(auth.alg), &Token {
        username: param.0.username,
        is_admin: admin
    }, auth.secret.as_bytes())
        .map_err(|_| json!({ "success": false, "error": "failed to sign token" }))?;
    Ok(json!({ "success": true, "token": token }))
}

#[post("/drop", format = "json", data = "<param>")]
pub fn drop(token: Token, param: Json<DropRequest>) -> Json<DropResponse> {
    unimplemented!()
}

#[post("/transfer", format = "json", data = "<param>")]
pub fn transfer(token: Token, param: Json<TransferRequest>) -> Json<TransferResponse> {
    unimplemented!()
}

#[get("/history")]
pub fn history(token: Token) -> Json<HistoryResponse> {
    unimplemented!()
}

#[post("/reg/deposit", format = "json", data = "<param>")]
pub fn deposit(token: Token, param: Json<DepositRequest>) -> Json<DepositResponse> {
    unimplemented!()
}

pub fn routes() -> Vec<rocket::Route> {
	routes![home, login, drop, transfer, history, deposit]
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid
}

fn decode_token(raw: &str, auth: &Auth) -> Option<Token> {
    use jwt::{Validation, decode};

    let validation = Validation {
        algorithms: vec![auth.alg],
        validate_exp: false,
        ..Default::default()
    };

    decode::<Token>(raw, auth.secret.as_bytes(), &validation)
        .ok()
        .map(|data| data.claims)
}

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = TokenError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("authorization").collect();
        if keys.len() == 0 {
            return Outcome::Failure((Status::Unauthorized, TokenError::Missing));
        }
        let server = request.guard::<State<state::Server>>().expect("Unable to obtain state for auth");
        let auth = &server.settings.auth;
        match decode_token(keys[0], auth) {
            None => Outcome::Failure((Status::Unauthorized, TokenError::Invalid)),
            Some(token) => Outcome::Success(token)
        }
    }
}
