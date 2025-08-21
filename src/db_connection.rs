use std::io::{Write, stdout};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PoolError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

pub type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection = diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>;

use crate::util::{get_database, get_pool_size_per_worker};

// diesel_migrations::embed_migrations!();
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_connection() -> Result<Pool, PoolError> {
    let database_url = get_database();
    let sql_pool = init_pool(&database_url);
    sql_pool
}

pub fn run_sql_schema_migrations() {
    let mut sql_pool = establish_connection().unwrap();
    let mut sql_pooled_connection = sql_pool_handler(&mut sql_pool).unwrap(); 

    print!("Running diesel database migrations...\n");
    stdout().flush().unwrap();
    
    let output = sql_pooled_connection.run_pending_migrations(MIGRATIONS).unwrap();
    print!("{output:?}\n");
    stdout().flush().unwrap();
}

// Ensure migrations run before any tests that need the database schema
#[ctor::ctor]
fn init_schema() {
    if cfg!(test) {
        run_sql_schema_migrations();
    }
}

fn init_pool(database_url: &str) -> Result<Pool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().max_size(get_pool_size_per_worker()).build(manager)
}    

fn sql_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    let sql_pooled_connection = pool.get().unwrap();
    Ok(sql_pooled_connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_establish_connection() {
        let pool = establish_connection();
        assert!(pool.is_ok());
    }

    #[test]
    fn test_run_sql_schema_migrations() {
        run_sql_schema_migrations();
        // Add assertions here to verify the success of the migrations
    }

    #[test]
    fn test_init_pool() {
        let database_url: String = get_database();
        let pool = init_pool(&database_url);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_sql_pool_handler() {
        let pool = establish_connection();
        let connection = sql_pool_handler(&pool.unwrap());
        assert!(connection.is_ok());
    }
}
