use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    username: String,
    is_admin: bool,
}

/// Repesents an ammount of money.
/// Limited to u32 due to Redis and Lua 5.1
pub type Balance = u32;

/// Represents a money transfer between two users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    source: String,
    target: String,
    ammount: Balance,
}

use super::settings::Auth;

use crate::db;
use crate::keyhash;
use crate::state;
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, content, Responder};
use rocket::{Response, State};
use rocket_contrib::json::{Json, JsonValue};

mod objs;
use objs::*;

pub enum JsonResponse {
    Success(JsonValue),
    Failure(JsonValue),
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
            Self::Success(v) => JsonStatus {
                success: true,
                value: v,
            },
            Self::Failure(v) => JsonStatus {
                success: false,
                value: v,
            },
        };
        serde_json::to_string(&status)
            .map(|string| content::Json(string).respond_to(req).unwrap())
            .map_err(|e| {
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
            Self::Failure(v) => Err(v),
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonStatus {
    success: bool,

    #[serde(flatten)]
    value: JsonValue,
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

#[get("/info")]
pub fn info(server: State<state::Server>, token: Token) -> JsonResponse {
    let mut conn = (*server).db_conn.borrow();
    let info = db::user_info(&mut conn, &token.username).map_err(|e| {
        eprintln!("Error getting user info for {}: {}", &token.username, e);
        return JsonResponse::error("internal server error");
    })?;
    JsonResponse::Success(json!({
        "realname": info.realname,
        "username": info.username,
        "balance": info.balance,
        "is_admin": info.is_admin
    }))
}

#[post("/login", format = "json", data = "<param>")]
pub fn login(server: State<state::Server>, param: Json<LoginRequest>) -> JsonResponse {
    use jwt::{encode, Header};

    let srv: &state::Server = &server;
    let mut conn = srv.db_conn.borrow();

    let valid = db::validate(&mut *conn, param.0.username.clone(), param.0.key).map_err(|e| {
        error!("Error validating login credentials: {}", e);
        JsonResponse::error("internal server error")
    })?;
    if !valid {
        return JsonResponse::fail("invalid username or password");
    }

    let auth = &(*server).settings.auth;
    let admin = db::is_admin(&mut *conn, param.0.username.clone()).map_err(|e| {
        error!("Error verifying admin status: {}", e);
        JsonResponse::error("internal server error")
    })?;

    let token = encode(
        &Header::new(auth.algorithm),
        &Token {
            username: param.0.username,
            is_admin: admin,
        },
        auth.secret.as_bytes(),
    )
    .map_err(|_| JsonResponse::error("failed to sign token"))?;
    JsonResponse::Success(json!({ "token": token }))
}

#[post("/register", format = "json", data = "<param>")]
pub fn register(server: State<state::Server>, param: Json<RegisterRequest>) -> JsonResponse {
    let mut conn = (*server).db_conn.borrow();
    let param = &(*param);

    let (keyhash, salt) = keyhash::generate(param.key.clone());

    info!("Creating an account for {}", param.username);
    let status = match db::create_account(
        &mut *conn,
        param.username.clone(),
        param.username.clone(),
        param.name.clone(),
        keyhash,
        salt,
    ) {
        Ok(status) => status,
        Err(what) => {
            error!("Error when contacting Redis: {:?}", what);
            return JsonResponse::fail("internal server error");
        }
    };
    debug!(
        "Account creation invoke for {} returned {:?}",
        param.username, status
    );

    match status.as_str() {
        "-KeyExists" => JsonResponse::fail("user already exists"),
        "+OK" => {
            use jwt::{encode, Header};

            let auth = &(*server).settings.auth;
            let token = encode(
                &Header::new(auth.algorithm),
                &Token {
                    username: param.username.clone(),
                    is_admin: false,
                },
                auth.secret.as_bytes(),
            )
            .map_err(|_| JsonResponse::error("failed to sign token"))?;
            JsonResponse::Success(json!({ "token": token }))
        }
        s @ &_ => {
            error!("Invalid return from account creation invoke: {}", s);
            JsonResponse::fail("internal server error")
        }
    }
}

#[post("/drop", format = "json", data = "<param>")]
pub fn drop(token: Token, param: Json<DropRequest>) -> JsonResponse {
    unimplemented!()
}

#[post("/transfer", format = "json", data = "<param>")]
pub fn transfer(
    server: State<state::Server>,
    token: Token,
    param: Json<TransferRequest>,
) -> JsonResponse {
    let mut conn = (*server).db_conn.borrow();

    if token.username == param.0.to {
        return JsonResponse::fail("you cannot make transfers to yourself");
    }

    let r =
        db::transaction(&mut conn, &token.username, &param.0.to, param.0.amount).map_err(|e| {
            eprintln!("Transaction error: {}", e);
            return JsonResponse::error("internal server error");
        })?;

    use db::TransactionStatus;

    match r {
        TransactionStatus::Success => JsonResponse::empty_success(),
        TransactionStatus::NotEnoughFunds => JsonResponse::fail("not enough funds"),
        TransactionStatus::InvalidFrom => {
            JsonResponse::fail("invalid source user (we're as confused as you right now)")
        }
        TransactionStatus::InvalidTo => JsonResponse::fail("invalid destination user"),
        TransactionStatus::Cooldown => {
            JsonResponse::fail("please wait before performing this action")
        }
    }
}

#[get("/history")]
pub fn history(server: State<state::Server>, token: Token) -> JsonResponse {
    let mut conn = (*server).db_conn.borrow();
    let h = db::history(&mut conn, &token.username).map_err(|e| {
        eprintln!("Error getting history: {}", e);
        return JsonResponse::error("internal server error");
    })?;

    let res: Vec<_> = h
        .into_iter()
        .map(|e| serde_json::from_str::<HistoryEntry>(&e))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            eprintln!("Error deserializing history entries: {}", e);
            return JsonResponse::error("internal server error");
        })?;
    JsonResponse::Success(json!({ "history": res }))
}

#[post("/withdraw", format = "json", data = "<param>")]
pub fn withdraw(
    server: State<state::Server>,
    token: Token,
    param: Json<WithdrawRequest>,
) -> JsonResponse {
    let mut conn = (*server).db_conn.borrow();
    let success = db::withdraw(&mut conn, token.username, param.0.amount).map_err(|e| {
        eprintln!("Error withdrawing: {}", e);
        return JsonResponse::error("internal server error");
    })?;
    if !success {
        return JsonResponse::fail("you don't have enough funds");
    }
    JsonResponse::empty_success()
}

#[post("/admin/deposit", format = "json", data = "<param>")]
pub fn deposit(
    server: State<state::Server>,
    token: Token,
    param: Json<DepositRequest>,
) -> JsonResponse {
    if !token.is_admin {
        return JsonResponse::fail("you are not an admin");
    }
    let mut conn = (*server).db_conn.borrow();
    db::deposit(&mut conn, param.0.username, param.0.amount).map_err(|e| {
        eprintln!("Error depositing money: {}", e);
        return JsonResponse::error("internal server error");
    })?;
    JsonResponse::empty_success()
}

#[post("/admin/withdraw", format = "json", data = "<param>")]
pub fn admin_withdraw(
    server: State<state::Server>,
    token: Token,
    param: Json<AdminWithdrawRequest>,
) -> JsonResponse {
    if !token.is_admin {
        return JsonResponse::fail("you are not an admin");
    }
    let mut conn = (*server).db_conn.borrow();
    let success = db::withdraw(&mut conn, param.0.username, param.0.amount).map_err(|e| {
        eprintln!("Error withdrawing money: {}", e);
        return JsonResponse::error("internal server error");
    })?;
    if !success {
        return JsonResponse::fail("not enough funds");
    }
    JsonResponse::empty_success()
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        home,
        info,
        login,
        drop,
        register,
        transfer,
        withdraw,
        history,
        deposit,
        admin_withdraw
    ]
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Invalid,
}

fn decode_token(raw: &str, auth: &Auth) -> Option<Token> {
    use jwt::{decode, Validation};

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
        let server = request
            .guard::<State<state::Server>>()
            .expect("Unable to obtain state for auth");
        let auth = &server.settings.auth;
        match decode_token(keys[0], auth) {
            None => Outcome::Failure((Status::Unauthorized, TokenError::Invalid)),
            Some(token) => Outcome::Success(token),
        }
    }
}
