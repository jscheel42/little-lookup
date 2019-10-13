use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PoolError};

pub type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

use crate::util::{get_database, get_pool_size_per_worker};

diesel_migrations::embed_migrations!();

pub fn establish_connection() -> Pool {
    let database_url = get_database();

    let sql_pool = init_pool(&database_url).expect("Failed to create pool");
    let sql_pooled_connection = sql_pool_handler(&sql_pool).unwrap();
    prepare_database(&sql_pooled_connection);
    sql_pool
}

fn init_pool(database_url: &str) -> Result<Pool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(get_pool_size_per_worker()).build(manager)
}

fn prepare_database(connection: &PgConnection) {
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).unwrap()
}

fn sql_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    let sql_pool = pool.get().unwrap();
    Ok(sql_pool)
}
