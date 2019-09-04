
use crate::settings::Settings;
use crate::pool::Pool;
use redis::Connection;

struct Server {
	settings: Settings,
	db_conn:  Pool<Connection>
}
impl Server {
}
