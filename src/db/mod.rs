pub const TRANSACTION_SCRIPT: &'static str = include_str!("transaction.lua");
pub const NEW_ACCOUNT_SCRIPT: &'static str = include_str!("new_account.lua");
pub const DEL_ACCOUNT_SCRIPT: &'static str = include_str!("del_account.lua");

pub const INITIAL_BALANCE: u32   = 500;
pub const MAX_RETRIES:     usize = 256;
pub const USERHASH_SIZE:   usize = 32;

mod names {
	pub fn uid_table() -> String {
		"uids".to_owned()
	}

	pub fn user_name(userhash: &str) -> String {
		format!("user:{}:name", userhash)
	}

	pub fn user_balance(userhash: &str) -> String {
		format!("user:{}:balance", userhash)
	}

	pub fn user_email(userhash: &str) -> String {
		format!("user:{}:email", userhash)
	}

	pub fn user_cooldown(userhash: &str) -> String {
		format!("user:{}:cd_lock", userhash)
	}

	pub fn user_history(userhash: &str) -> String {
		format!("user:{}:history", userhash)
	}

	pub fn user_keyhash(userhash: &str) -> String {
		format!("user:{}:keyhash", userhash)
	}

	pub fn user_salt(userhash: &str) -> String {
		format!("user:{}:salt", userhash)
	}

	pub fn user_tokens(userhash: &str) -> String {
		format!("user:{}:tokens", userhash)
	}

	pub fn user_username(userhash: &str) -> String {
		format!("user:{}:username", userhash)
	}

	pub fn user_admin(userhash: &str) -> String {
		format!("user:{}:admin", userhash)
	}
}

pub fn get_userhash(
	connection: &mut redis::Connection,
	username: &str) -> redis::RedisResult<String> {

	use redis::Commands;
	connection.hget(names::uid_table(), username)
}

#[derive(Debug)]
pub struct UserInfo {
    pub username: String,
    pub balance: u32,
    pub admin: bool
}

pub fn user_info(conn: &mut redis::Connection, username: &str)
    -> redis::RedisResult<UserInfo> {
    trace!("Attempting to get user info for {}", username);

    let userhash = get_userhash(conn, &username)?;
    
    use redis::Commands;

    Ok(UserInfo {
        username: username.to_owned(),
        balance: conn.get(names::user_balance(&userhash))?,
        admin: is_admin(conn, username.to_owned())?
    })
}

pub fn history(conn: &mut redis::Connection, username: &str)
    -> redis::RedisResult<Vec<String>> {
    trace!("Attempting to get history for user {}", username);

    let userhash = get_userhash(conn, &username)?;
    use redis::Commands;

    conn.lrange(names::user_history(&userhash), -20, -1)
}

#[derive(Debug)]
pub enum TransactionStatus {
	Success,
	NotEnoughFunds,
	InvalidFrom,
	InvalidTo,
    Cooldown
}

pub fn transaction(conn: &mut redis::Connection, from: &str, to: &str, 
          		   amount: u32) -> redis::RedisResult<TransactionStatus> {
    trace!("Attempting to transfer {} from {} to {}", amount, from, to);

	let fromhash = get_userhash(conn, &from)?;
	let tohash = get_userhash(conn, &to)?;

	let script = redis::Script::new(TRANSACTION_SCRIPT);
	let code: u32 = script
		.key(names::user_balance(&fromhash))
        .key(names::user_history(&fromhash))
        .key(names::user_cooldown(&fromhash))
		.key(names::user_balance(&tohash))
        .key(names::user_history(&tohash))
        .key(names::user_cooldown(&tohash))
		.arg(amount)
        .arg(from)
        .arg(to)
		.invoke(conn)?;
    let status = match code {
        0 => TransactionStatus::Success,
        1 => TransactionStatus::NotEnoughFunds,
        2 => TransactionStatus::InvalidFrom,
        3 => TransactionStatus::InvalidTo,
        4 => TransactionStatus::Cooldown,
        _ => panic!("Invalid status code returned")
    };
    Ok(status)
}

use crate::api::Balance;
pub fn create_account(
	connection: &mut redis::Connection,
	username:   String,
	email:      String,
	realname:   String,
	keyhash:    String,
	salt:       String) -> redis::RedisResult<String> {

	let script = redis::Script::new(NEW_ACCOUNT_SCRIPT);
	for _ in (0..MAX_RETRIES) {
		let userhash = (0..USERHASH_SIZE)
			.into_iter()
			.map(|_| rand::random::<u8>())
			.map(|n| format!("{:x}", n))
			.collect::<String>();

		info!("Creating an account for {} on hash {}", username, userhash);
		
		let result: String = script
			.key(names::user_name(&userhash))
			.key(names::user_email(&userhash))
			.key(names::user_keyhash(&userhash))
			.key(names::user_salt(&userhash))
			.key(names::user_cooldown(&userhash))
			.key(names::user_balance(&userhash))
			.key(names::uid_table())
            .key(names::user_username(&userhash))
			.arg(INITIAL_BALANCE)
			.arg(&email)
			.arg(&realname)
			.arg(&keyhash)
			.arg(&salt)
			.arg(&username)
			.arg(&userhash)
			.invoke(connection)?;

		if result.as_str() != "-Retry" {
			return Ok(result)
		} else {
			info!("Retrying account creation for user {}", username);
		}
	}
    Ok("-UnableToCreate".to_owned())
}

pub fn delete_account(
	connection: &mut redis::Connection,
	username:   String
	) -> redis::RedisResult<String> {
	
	let userhash = get_userhash(connection, &username)?;
	trace!("Deleting the account on userhash {}", userhash);

	let script = redis::Script::new(DEL_ACCOUNT_SCRIPT);
	script
		.key(names::user_email(&userhash))
		.key(names::user_name(&userhash))
		.key(names::user_history(&userhash))
		.key(names::user_cooldown(&userhash))
		.key(names::user_keyhash(&userhash))
		.key(names::user_salt(&userhash))
		.key(names::user_tokens(&userhash))
		.key(names::user_balance(&userhash))
		.key(names::user_username(&userhash))
		.key(names::uid_table())
		.invoke(connection)
}

pub fn validate(
	connection: &mut redis::Connection, 
	username: String,
	password: String) -> redis::RedisResult<bool> {

    trace!("Validating credentials for user {}", username);
	
	let userhash = match get_userhash(connection, &username) {
        Err(e) => return Ok(false),
        Ok(s)  => s
    };

    use redis::Commands;
    let hash: Option<String> = connection.get(names::user_keyhash(&userhash))?;
	let salt: Option<String> = connection.get(names::user_salt(&userhash))?;
	trace!("Trying to log in user with hash {}", userhash);
	use crate::keyhash;
	Ok(match (hash, salt) {
		(Some(hash), Some(salt)) => 
			keyhash::verify(password, hash, salt).unwrap_or(false),
		_ => false
	})
}

pub fn is_admin(conn: &mut redis::Connection, username: String) -> redis::RedisResult<bool> {
 	let userhash = get_userhash(conn, &username)?;

 	use redis::Commands;
    conn.exists(names::user_admin(&userhash))
}

pub fn deposit(conn: &mut redis::Connection, username: String, amount: u32)
    -> redis::RedisResult<()> {
    let userhash = get_userhash(conn, &username)?;

    use redis::Commands;
    conn.incr(names::user_balance(&userhash), amount)
}
