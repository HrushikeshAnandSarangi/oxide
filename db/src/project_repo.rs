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
            INSERT INTO projects (id, name, subdomain, repo_url, install_command, build_command, run_command, root_directory, active_deployment_id, created_at)
            VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#
        ).bind(project.id)
        .bind(&project.name)
        .bind(project.subdomain.value())
        .bind(&project.repo_url)
        .bind(&project.install_command)
        .bind(&project.build_command)
        .bind(&project.run_command)
        .bind(&project.root_directory)
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

    pub async fn update_active_deployment(&self, project_id: &uuid::Uuid, deployment_id: &uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE projects SET active_deployment_id = $1 WHERE id = $2
            "#
        )
        .bind(deployment_id)
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_all(&self) -> Result<Vec<crate::models::ProjectRow>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::ProjectRow>(
            "SELECT * FROM projects ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &uuid::Uuid) -> Result<Option<crate::models::ProjectRow>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::ProjectRow>(
            "SELECT * FROM projects WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn add_env_var(&self, env_var: &domain::project::ProjectEnvVar) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO project_env_vars (id, project_id, key, value_encrypted, nonce, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(env_var.id)
        .bind(env_var.project_id)
        .bind(&env_var.key)
        .bind(&env_var.value_encrypted)
        .bind(&env_var.nonce)
        .bind(env_var.created_at)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_env_vars(&self, project_id: uuid::Uuid) -> Result<Vec<crate::models::ProjectEnvVarRow>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::ProjectEnvVarRow>(
            r#"
            SELECT * FROM project_env_vars WHERE project_id = $1
            "#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
    }
}

