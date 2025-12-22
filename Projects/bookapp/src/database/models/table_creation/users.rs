use sqlx::PgPool;

pub async fn user_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(USER_TABLE).execute(pool).await?;
    Ok(())
}

const USER_TABLE: &str = "
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    surname VARCHAR(70) NOT NULL,
    phone VARCHAR(20) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    pwd VARCHAR(255) NOT NULL,
    create_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(50) NOT NULL,
    write_date TIMESTAMP,
    update_by VARCHAR(50)
)
";
