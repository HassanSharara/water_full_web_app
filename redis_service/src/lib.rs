pub mod models;
pub mod macros;
use deadpool_redis::{Config, Pool, Runtime};
use std::sync::OnceLock;

static REDIS_POOL: OnceLock<Pool> = OnceLock::new();

pub fn init_global_pool() {
    let database_url = dotenv::var("redis_database_url").unwrap();
    let mut cfg = Config::default();
    cfg.url = Some(database_url);
    cfg.connection = None;
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool");
    if REDIS_POOL.set(pool).is_err() {
        println!("Warning: Redis pool was already initialized.");
    }
}

pub fn get_redis() -> &'static Pool {
    REDIS_POOL.get().expect("Redis pool is not initialized! Call init_global_pool first.")
}


use deadpool_redis::redis::AsyncCommands;

pub async fn run_redis_demo() {
    // 1. Grab an available connection from your global static pool
    let mut conn = get_redis()
        .get()
        .await
        .expect("Failed to secure a connection from the pool");

    let key = "user:session:100";
    let value = "active_authenticated_state";

    // 2. Save (SET) the key into Redis
    // We use the unit type `()` as the type hint because we just care if it succeeds or fails
    let _: () = conn.set(key, value)
        .await
        .expect("Failed to write data to Redis");

    println!("Successfully saved key: '{}'", key);

    // 3. Retrieve (GET) the key back out
    // We explicitly type-hint `String` so the driver knows how to parse the binary response
    let retrieved_value: String = conn.get(key)
        .await
        .expect("Failed to read data from Redis");

    println!("Successfully retrieved value: '{}'", retrieved_value);
}