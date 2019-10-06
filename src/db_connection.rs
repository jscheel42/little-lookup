use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager, PoolError };

pub type SLPool = Pool<ConnectionManager<SqliteConnection>>;
pub type SLConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

fn init_pool(database_url: &str) -> Result<SLPool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> SLPool {
    let key = "LITTLE_LOOKUP_DATABASE";
    let database_url = match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("default.db"),
    };
    init_pool(&database_url).expect("Failed to create pool")
}
