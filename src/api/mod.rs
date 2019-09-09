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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    source:  String,
    target:  String,
    ammount: Balance
}

use super::settings::Auth;

use rocket_contrib::json::{Json, JsonValue};
use rocket::{Response, State};
use rocket::response::{self, Responder, content};
use rocket::request::{Request, FromRequest, Outcome};
use rocket::http::{Status, ContentType};
use crate::state;
use crate::db;
use crate::keyhash;

mod objs;
use objs::*;

pub enum JsonResponse {
    Success(JsonValue),
    Failure(JsonValue)
}

impl JsonResponse {
    fn error(msg: &str) -> JsonValue {
        json!({ "error": msg })
    }

    fn fail(msg: &str) -> Self {
        Self::Failure(Self::error(msg))
    }

    fn empty_success() -> Self {
        Self::Success(json!({}))
    }
}

impl<'r> Responder<'r> for JsonResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let status = match self {
            Self::Success(v) => JsonStatus { success: true,  value: v },
            Self::Failure(v) => JsonStatus { success: false, value: v }
        };
        serde_json::to_string(&status).map(|string| {
            content::Json(string).respond_to(req).unwrap()
        }).map_err(|e| {
            error!("JSON failed to serialize: {:?}", e);
            Status::InternalServerError
        })
    }
}

impl core::ops::Try for JsonResponse {
    type Ok = JsonValue;
    type Error = JsonValue;

    fn from_error(v: Self::Error) -> Self {
        Self::Failure(v)
    }

    fn from_ok(v: Self::Ok) -> Self {
        Self::Success(v)
    }

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::Success(v) => Ok(v),
            Self::Failure(v) => Err(v)
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonStatus {
    success: bool,

    #[serde(flatten)]
    value: JsonValue
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
pub fn login(server: State<state::Server>, param: Json<LoginRequest>) -> JsonResponse {
    use jwt::{Header, encode};

    let srv: &state::Server = &server;
    let mut conn = srv.db_conn.borrow();

    let valid = db::validate(&mut *conn, param.0.username.clone(), param.0.key)
        .map_err(|_| JsonResponse::error("internal server error"))?;
    if !valid {
        return JsonResponse::fail("invalid username or password");
    }

	let auth = &(*server).settings.auth;
    let admin = db::is_admin(&mut *conn, param.0.username.clone())
        .map_err(|_| JsonResponse::error("internal server error"))?;

    let token = encode(&Header::new(auth.algorithm), &Token {
        username: param.0.username,
        is_admin: admin
    }, auth.secret.as_bytes())
        .map_err(|_| JsonResponse::error("failed to sign token"))?;
    JsonResponse::Success(json!({ "token": token }))
}

#[post("/register", format = "json", data = "<param>")]
pub fn register(server: State<state::Server>, param: Json<RegisterRequest>)
	-> JsonResponse {
	
	let mut conn = (*server).db_conn.borrow();
	let param    = &(*param);

	let (keyhash, salt) = keyhash::generate(param.key.clone());

	info!("Creating an account for {}", param.email);
	let status = match db::create_account(
		&mut *conn,
		param.email.clone(),
		0,
		param.email.clone(),
		param.name.clone(),
		keyhash,
		salt
		){

		Ok(status) => status,
		Err(what) => {
			error!("Error when contacting Redis: {:?}", what);
			return JsonResponse::fail("database error");
		}
	};
	debug!("Account creation invoke for {} returned {:?}", param.email, status);

	match status.as_str() {
		"-KeyExists" => JsonResponse::fail("user already exists"),
		"+OK" => JsonResponse::empty_success(),
		s @ &_ => {
			error!("Invalid return from account creation invoke: {}", s);
			JsonResponse::fail("database error")
		}
	}
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
	routes![home, login, drop, register, transfer, history, deposit]
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid
}

fn decode_token(raw: &str, auth: &Auth) -> Option<Token> {
    use jwt::{Validation, decode};

    let validation = Validation {
        algorithms: vec![auth.algorithm],
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
