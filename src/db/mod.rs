pub const NEW_ACCOUNT_SCRIPT: &'static str = include_str!("new_account.lua");
pub const TRANSACTION_SCRIPT: &'static str = include_str!("transaction.lua");
pub const INITIAL_BALANCE: u32             = 500;

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

#[derive(Debug)]
pub enum AccountCreationStatus {
    Success,
    UserExists,
    HashFailure
}

pub fn create_account(conn: &mut redis::Connection, username: String, password: String)
    -> redis::RedisResult<AccountCreationStatus> {
    use bcrypt::{DEFAULT_COST, hash};
    let password_hash = match hash(password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return Ok(AccountCreationStatus::HashFailure)
    };

    let result: u32 = redis::Script::new(NEW_ACCOUNT_SCRIPT)
        .key(format!("users:{}:password", username))
        .key(format!("users:{}:balance", username))
        .arg(password_hash)
        .arg(INITIAL_BALANCE)
        .invoke(conn)?;
    match result {
        0 => return Ok(AccountCreationStatus::Success),
        1 => return Ok(AccountCreationStatus::UserExists),
        _ => panic!("Invalid status code returned")
    }
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
