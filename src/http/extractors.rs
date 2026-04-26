use sqlx::{PgPool, Transaction, Postgres};
use uuid::Uuid;
use axum::async_trait;

pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

impl TenantContext {
    /// This is the ONLY way we query the database. It enforces Row-Level Security.
    pub async fn begin_rls_transaction<'a>(
        &self,
        pool: &'a PgPool,
    ) -> Result<Transaction<'a, Postgres>, sqlx::Error> {
        
        // 1. Check out a connection and begin a transaction
        let mut tx = pool.begin().await?;

        // 2. Set the RLS context using SET LOCAL.
        // LOCAL guarantees that the moment the transaction commits or rolls back,
        // the session variable is immediately wiped, protecting the connection pool.
        let query = format!(
            "SET LOCAL app.current_tenant_id = '{}';",
            self.tenant_id
        );
        
        sqlx::query(&query).execute(&mut *tx).await?;

        // 3. (Optional) You can also set the user_id if you want to use it in triggers
        // let user_query = format!("SET LOCAL app.current_user_id = '{}';", self.user_id);
        // sqlx::query(&user_query).execute(&mut *tx).await?;

        Ok(tx)
    }
}