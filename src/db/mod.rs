pub const TRANSACTION_SCRIPT: &'static str = include_str!("transaction.lua");
pub const NEW_ACCOUNT_SCRIPT: &'static str = include_str!("new_account.lua");
pub const DEL_ACCOUNT_SCRIPT: &'static str = include_str!("del_account.lua");
pub const INITIAL_BALANCE: u32             = 500;

mod name {
	pub fn user_name(userhash: &str) -> String {
		format!("{}:name", userhash)
	}

	pub fn user_email(userhash: &str) -> String {
		format!("{}:email", userhash)
	}

	pub fn user_cooler(userhash: &str) -> String {
		format!("{}:cd_lock", userhash)
	}

	pub fn user_history(userhash: &str) -> String {
		format!("{}:history", userhash)
	}

	pub fn user_keyhash(userhash: &str) -> String {
		format!("{}:keyhash", userhash)
	}

	pub fn user_salt(userhash: &str) -> String {
		format!("{}:salt", userhash)
	}

	pub fn user_tokens(userhash: &str) -> String {
		format!("{}:tokens", userhash)
	}
}

#[derive(Debug)]
pub enum TransactionStatus {
    Success,
    NotEnoughFunds,
    InvalidFrom,
    InvalidTo
}

pub fn bank_transaction(conn: &mut redis::Connection, from: &str, to: &str, 
                        amount: u32) -> redis::RedisResult<TransactionStatus> {

    let script = redis::Script::new(TRANSACTION_SCRIPT);
    let code: u32 = script.key(from).key(to).arg(amount).invoke(conn)?;
    let status = match code {
        0 => TransactionStatus::Success,
        1 => TransactionStatus::NotEnoughFunds,
        2 => TransactionStatus::InvalidFrom,
        3 => TransactionStatus::InvalidTo,
        _ => panic!("Invalid status code returned")
    };
    Ok(status)
}

use crate::api::Balance;
pub fn create_account(
	connection: &mut redis::Connection,
	userhash:   String,
	sbalance:   Balance,
	email:      String,
	realname:   String,
	keyhash:    String,
	salt:       String) -> redis::RedisResult<String> {

	trace!("Creating a new account on userhash {}", userhash);
	let script = redis::Script::new(NEW_ACCOUNT_SCRIPT);
	Ok(script
		.key(name::user_name(&userhash))
		.key(name::user_email(&userhash))
		.key(name::user_keyhash(&userhash))
		.key(name::user_salt(&userhash))
		.key(name::user_cooler(&userhash))
		.key(userhash)
		.arg(sbalance)
		.arg(email)
		.arg(realname)
		.arg(keyhash)
		.arg(salt)
		.invoke(connection)?)
}

pub fn delete_account(
	connection: &mut redis::Connection,
	userhash:   String
	) -> redis::RedisResult<String> {
	
	let script = redis::Script::new(DEL_ACCOUNT_SCRIPT);
	Ok(script
		.key(name::user_email(&userhash))
		.key(name::user_name(&userhash))
		.key(name::user_history(&userhash))
		.key(name::user_cooler(&userhash))
		.key(name::user_keyhash(&userhash))
		.key(name::user_salt(&userhash))
		.key(name::user_tokens(&userhash))
		.key(userhash)
		.invoke(connection)?)
}

pub fn validate(conn: &mut redis::Connection, username: String,
                password: String) -> redis::RedisResult<bool> {
    use redis::Commands;
    use bcrypt::verify;

    let hash: Option<String> = conn.get(format!("users:{}:password", username))?;
    
    Ok(hash.as_ref().and_then(|h| verify(password, h).ok()).unwrap_or(false))
}

pub fn is_admin(conn: &mut redis::Connection, username: String) -> redis::RedisResult<bool> {
    use redis::Commands;
    conn.exists(format!("users:{}:admin", username))
}


