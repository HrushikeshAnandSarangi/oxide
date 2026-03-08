use domain::Deployment;
use sqlx::{PgPool, Result};
use domain::DeploymentStatus;
use uuid::Uuid;
pub struct DeploymentRepository{
    pool:PgPool,
}

impl DeploymentRepository {
    pub fn new(pool:PgPool)->Self{
        Self{pool}
    }
    pub async fn create(&self,deployment:&Deployment)->Result<(),sqlx::Error>{
        sqlx::query(
            r#"
            INSERT INTO DeploymentRepository(id,project_id,version,artifact_path,docker_image,container_id,status,created_at)
            VALUES($1,$2,$3,$4,$5,$6,$7,$8)
            "#
        )
            .bind(deployment.id)
            .bind(deployment.project_id)
            .bind(deployment.version.clone())
            .bind(deployment.artifact_path.clone())
            .bind(deployment.docker_image.clone())
            .bind(deployment.container_id.clone())
            .bind(format!("{:?}",deployment.status))
            .bind(deployment.created_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_status(&self,id:&Uuid,status:DeploymentStatus)->Result<(),sqlx::Error>{
        sqlx::query(
            r#"
            UPDATE deployments
            SET status = $1
            WHERE id = $2
            "#
        )
            .bind(format!("{:?}",status))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
        
    }

    pub async fn get_running_deployments(&self)->Result<Vec<(String,u16)>,sqlx::Error>{
        use sqlx::Row;
        let records=sqlx::query(
        r#"
        SELECT p.subdomain, d.container_port
        FROM deployments d
        JOIN projects p ON d.project_id= p.id 
        WHERE d.status = 'Running' AND d.container_port IS NOT NULL
        "#
        )
            .fetch_all(&self.pool)
            .await?;

        let routes=records.into_iter().map(|r|{
            let subdomain: String = r.get("subdomain");
            // PostgreSQL integer is usually mapped to i32 in sqlx
            let container_port: i32 = r.get("container_port");
            (subdomain, container_port as u16)
        }).collect();
        Ok(routes)
    }
}
