use diesel::pg::PgConnection;
use diesel::r2d2::{ ConnectionManager, PoolError };

pub type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

diesel_migrations::embed_migrations!();

pub fn establish_connection() -> Pool {
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

fn init_pool(database_url: &str) -> Result<Pool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(5).build(manager)
}

fn prepare_database(connection: &PgConnection) {
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).unwrap()
}

fn sl_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    let sl_pool = pool.get().unwrap();
    Ok(sl_pool)
}
