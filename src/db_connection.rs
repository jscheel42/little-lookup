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

    let sql_pool = init_pool(&database_url).expect("Failed to create pool");
    let sql_pooled_connection = sql_pool_handler(&sql_pool).unwrap();
    prepare_database(&sql_pooled_connection);
    sql_pool
}

fn init_pool(database_url: &str) -> Result<Pool, PoolError> {
    let pool_size_key = "LITTLE_LOOKUP_POOL_SIZE_PER_WORKER";
    let pool_size_num = match std::env::var(pool_size_key) {
        Ok(val) => val.parse::<u32>().unwrap(),
        Err(_) => 5
    };

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(pool_size_num).build(manager)
}

fn prepare_database(connection: &PgConnection) {
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).unwrap()
}

fn sql_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    let sql_pool = pool.get().unwrap();
    Ok(sql_pool)
}
