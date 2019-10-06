use diesel::sqlite::SqliteConnection;
use diesel::r2d2::{ Pool, PooledConnection, ConnectionManager, PoolError };

pub type SLPool = Pool<ConnectionManager<SqliteConnection>>;
pub type SLPooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

diesel_migrations::embed_migrations!();

pub fn establish_connection() -> SLPool {
    let key = "LITTLE_LOOKUP_DATABASE";
    let database_url = match std::env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("default.db"),
    };
    let sl_pool = init_pool(&database_url).expect("Failed to create pool");
    let sl_pooled_connection = sl_pool_handler(&sl_pool).unwrap();
    prepare_database(&sl_pooled_connection);
    sl_pool
}

fn init_pool(database_url: &str) -> Result<SLPool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

fn prepare_database(connection: &SqliteConnection) {
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).unwrap()
}

fn sl_pool_handler(pool: &SLPool) -> Result<SLPooledConnection, PoolError> {
    let sl_pool = pool.get().unwrap();
    Ok(sl_pool)
}
