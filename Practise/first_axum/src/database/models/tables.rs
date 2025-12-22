use crate::database::models::table_creation::users::user_table;
use sqlx::PgPool;

pub struct Tables;
impl Tables {
    pub async fn initialize_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
        user_table(pool)
            .await
            .expect("Failed to create users table");
        Ok(())
    }
}
