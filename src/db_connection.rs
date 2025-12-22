use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PoolError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use std::io::{stdout, Write};

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
    let mut sql_pool = match establish_connection() {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            std::process::exit(1);
        }
    };

    let mut sql_pooled_connection = match sql_pool_handler(&mut sql_pool) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to get database connection from pool: {}", e);
            std::process::exit(1);
        }
    };

    print!("Running diesel database migrations...\n");
    let _ = stdout().flush();

    match sql_pooled_connection.run_pending_migrations(MIGRATIONS) {
        Ok(output) => {
            print!("{output:?}\n");
            let _ = stdout().flush();
        }
        Err(e) => {
            eprintln!("Failed to run database migrations: {}", e);
            std::process::exit(1);
        }
    }
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
    Pool::builder()
        .max_size(get_pool_size_per_worker())
        .build(manager)
}

fn sql_pool_handler(pool: &Pool) -> Result<PooledConnection, PoolError> {
    pool.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_establish_connection() {
        let pool = establish_connection();
        assert!(
            pool.is_ok(),
            "Should successfully establish connection pool"
        );

        let pool = pool.unwrap();
        assert!(pool.max_size() > 0, "Pool should have positive max size");
    }

    #[test]
    fn test_run_sql_schema_migrations() {
        run_sql_schema_migrations();

        let pool = establish_connection().expect("Failed to establish connection");
        let mut conn = pool.get().expect("Failed to get connection from pool");

        use diesel::dsl::sql;
        use diesel::prelude::*;
        use diesel::sql_types::BigInt;

        let result: Result<i64, _> = sql::<BigInt>(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'items'",
        )
        .get_result(&mut conn);

        assert!(
            result.is_ok(),
            "Should be able to query database after migrations"
        );
        assert_eq!(
            result.unwrap(),
            1,
            "Items table should exist after migrations"
        );
    }

    #[test]
    fn test_init_pool() {
        let database_url: String = get_database();
        let pool = init_pool(&database_url);
        assert!(pool.is_ok(), "Should successfully initialize pool");

        let pool = pool.unwrap();
        assert_eq!(
            pool.max_size(),
            get_pool_size_per_worker(),
            "Pool max size should match configured pool size per worker"
        );
    }

    #[test]
    fn test_init_pool_with_custom_size() {
        std::env::set_var("LITTLE_LOOKUP_POOL_SIZE_PER_WORKER", "3");
        let database_url: String = get_database();
        let pool = init_pool(&database_url);
        assert!(pool.is_ok(), "Pool creation should succeed");

        let pool = pool.unwrap();
        assert_eq!(pool.max_size(), 3, "Pool should respect custom size");

        std::env::remove_var("LITTLE_LOOKUP_POOL_SIZE_PER_WORKER");
    }

    #[test]
    fn test_sql_pool_handler() {
        let pool = establish_connection();
        assert!(pool.is_ok(), "Should establish connection pool");

        if let Ok(pool_instance) = pool {
            let connection = sql_pool_handler(&pool_instance);
            assert!(connection.is_ok(), "Should get connection from pool");

            let mut conn = connection.unwrap();
            use diesel::dsl::sql;
            use diesel::prelude::*;
            use diesel::sql_types::Integer;

            let result: Result<i32, _> = sql::<Integer>("SELECT 1").get_result(&mut conn);
            assert!(
                result.is_ok(),
                "Should be able to execute queries on pooled connection"
            );
            assert_eq!(result.unwrap(), 1, "Query should return expected result");
        }
    }

    #[test]
    fn test_pool_reuse() {
        let pool = establish_connection().expect("Failed to establish connection");

        let conn1 = pool.get();
        assert!(conn1.is_ok(), "First connection should succeed");
        drop(conn1);

        let conn2 = pool.get();
        assert!(
            conn2.is_ok(),
            "Second connection should succeed after first is dropped"
        );
    }

    #[test]
    fn test_pool_multiple_connections() {
        let pool = establish_connection().expect("Failed to establish connection");

        let pool_size = get_pool_size_per_worker();
        let test_size = std::cmp::min(pool_size, 2);

        let mut connections = Vec::new();
        for i in 0..test_size {
            let conn = pool.get();
            assert!(
                conn.is_ok(),
                "Should be able to get connection {} from pool",
                i + 1
            );
            connections.push(conn.unwrap());
        }

        assert_eq!(
            connections.len() as u32,
            test_size,
            "Should be able to get {} connections",
            test_size
        );
    }
}
