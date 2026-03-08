pub mod models;
pub mod project_repo;
pub mod deployment_repo;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url:&str)->Result<PgPool,sqlx::Error>{
    PgPoolOptions::new()
    .max_connections(10)
    .connect(database_url)
    .await
}
