use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;

pub async fn user_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(USER_TABLE).execute(pool).await?;

    let username = "admin".to_string();
    let name = "Admin".to_string();
    let surname = "Admin".to_string();
    let phone = "+27824505996".to_string();
    let email = "admin@gmail.com".to_string();
    let created_by = "admin".to_string();
    let plain_pwd: String = format!("{}#01", &username);
    let hash_pwd = match hash(plain_pwd, DEFAULT_COST) {
        Ok(p) => p,
        Err(e) => {
            return Err(sqlx::Error::Protocol(format!(
                "Password hashing failed: {e}"
            )));
        }
    };

    sqlx::query(INSERT_ADMIN_USER)
        .bind(username)
        .bind(name)
        .bind(surname)
        .bind(phone)
        .bind(email)
        .bind(hash_pwd)
        .bind(created_by)
        .execute(pool)
        .await?;
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
    update_by VARCHAR(50),
    active BOOLEAN DEFAULT TRUE
)
";

const INSERT_ADMIN_USER: &str = "
INSERT INTO users (username, name, surname, phone, email, pwd, created_by)
VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (username) DO NOTHING
";
