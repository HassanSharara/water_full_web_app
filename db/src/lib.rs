pub mod models;

use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool};
use sqlx::pool::PoolConnection;
use tokio::sync::OnceCell;

thread_local! {
    static DB_POOL: OnceCell<MySqlPool> = OnceCell::const_new();

}
// 2. Your internal initialization function (remains async)
async fn create_pool() -> Result<MySqlPool, sqlx::Error> {
    let mysql_url = dotenv::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL must be set");
    MySqlPoolOptions::new()
        .max_connections(2)
        .min_connections(1)
        .connect(&mysql_url)
        .await
}

/// 🌟 Automatically initializes the pool on the first call if it doesn't exist yet,
/// then returns a cheap handle clone that can be safely passed anywhere across threads.
pub async fn get_pool() -> Result<PoolConnection<MySql>, sqlx::Error> {
    let pool_opt = DB_POOL.with(|c| c.get().cloned());
    let pool = match pool_opt {
        Some(p) => p,
        None => {
            let new_pool = create_pool().await?;
            DB_POOL.with(|c| {
                let _ = c.set(new_pool.clone());
                c.get().unwrap().clone()
            })
        }
    };
    pool.acquire().await
}