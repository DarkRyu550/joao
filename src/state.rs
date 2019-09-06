
use crate::settings::Settings;
use crate::pool::Pool;
use redis::Connection;

pub struct Server {
	pub settings: Settings,
	pub db_conn:  Pool<Connection>
}
impl Server {
}
