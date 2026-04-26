use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Represents the securely authenticated user making the HTTP request.
/// This is injected into the request by the JWT Middleware.
#[derive(Clone, Debug)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
}

impl TenantContext {
    /// Opens an ACID database transaction and strictly binds it to the current Tenant.
    /// This is the ONLY way handlers should access the database.
    pub async fn begin_rls_transaction<'a>(
        &self,
        pool: &'a PgPool,
    ) -> Result<Transaction<'a, Postgres>, sqlx::Error> {
        
        // 1. Checkout a connection and start the transaction
        let mut tx = pool.begin().await?;

        // 2. Set the Tenant ID for Row-Level Security
        let tenant_query = format!("SET LOCAL app.current_tenant_id = '{}';", self.tenant_id);
        sqlx::query(&tenant_query).execute(&mut *tx).await?;

        // 3. Set the User ID (Useful for database audit triggers that record 'created_by')
        let user_query = format!("SET LOCAL app.current_user_id = '{}';", self.user_id);
        sqlx::query(&user_query).execute(&mut *tx).await?;

        Ok(tx)
    }
}