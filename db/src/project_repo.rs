use sqlx::PgPool;
use domain::{Project};

use crate::models::ProjectRow;
 
pub struct ProjectRepository{
    pool:PgPool,
}
impl ProjectRepository {
    pub fn new(pool:PgPool)->Self{
        Self{pool}
    }
    pub async fn create(&self, project:&Project)->Result<(),sqlx::Error>{
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, subdomaine,active_deployment_id, created_at)
            VALUES($1,$2,$3,$4,$5)
            "#
        ).bind(project.id)
        .bind(&project.name)
        .bind(project.subdomain.value())
        .bind(project.active_deployment_id)
        .bind(project.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn find_by_subdomain(&self, subdomain:&str)->Result<Option<ProjectRow>,sqlx::Error>{
        sqlx::query_as::<_,ProjectRow>(
            r#"
            SELECT * FROM projects WHERE subdomain = $1
            "#
        )
            .bind(subdomain)
            .fetch_optional(&self.pool)
            .await
    }
    
}

