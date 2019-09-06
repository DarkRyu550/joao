pub const TRANSACTION_SCRIPT: &'static str = include_str!("transaction.lua");

#[derive(Debug)]
pub enum TransactionStatus {
    Success,
    NotEnoughFunds,
    InvalidFrom,
    InvalidTo
}

pub fn bank_transaction(
	conn: &mut redis::Connection, 
	from: &str, 
	to: &str, 
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
pub struct LockGuard<'a> {
	conn: &'a mut redis::Connection,
}

pub fn create_account(
	username: String,
	realname: Option<String>,
	) -> redis::RedisResult<AccountCreationStatus> {

	let pipe = redis::pipe()
		.atomic()
		.cmd("SETNX").arg("new").arg(123)
		.cmd("SETNX").arg("new").arg(321)
		.
}
