use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PoolError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

pub type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

use crate::util::{get_database, get_pool_size_per_worker};

// diesel_migrations::embed_migrations!();
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_connection() -> Pool {
    let database_url = get_database();

    let mut sql_pool = init_pool(&database_url).expect("Failed to create pool");

    // Create a pooled connection which we'll use to run migrations
    let mut sql_pooled_connection = sql_pool_handler(&mut sql_pool).unwrap();
    prepare_database(&mut sql_pooled_connection);

    sql_pool
}

fn init_pool(database_url: &str) -> Result<Pool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(get_pool_size_per_worker()).build(manager)
}

fn prepare_database(connection: &mut PgConnection) {
    // embedded_migrations::run_with_output(connection, &mut std::io::stdout()).unwrap()
    connection.run_pending_migrations(MIGRATIONS).unwrap();
}

fn sql_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    let sql_pooled_connection = pool.get().unwrap();
    Ok(sql_pooled_connection)
}
