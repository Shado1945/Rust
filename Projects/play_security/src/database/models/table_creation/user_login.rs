use sqlx::PgPool;

pub async fn user_login_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(USER_LOGIN_TABLE).execute(pool).await?;
    Ok(())
}

const USER_LOGIN_TABLE: &str = "
CREATE TABLE IF NOT EXISTS user_login (
    username VARCHAR(50) PRIMARY KEY,
    token VARCHAR(255),
    created_datetime TIMESTAMP NOT NULL,
    expire_datetime TIMESTAMP NOT NULL
)
";
