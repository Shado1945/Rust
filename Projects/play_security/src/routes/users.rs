use crate::middleware::auth::auth_middleware;
use crate::response::responses::Response;
use axum::{
    Json, Router,
    extract::{Path, State},
    middleware,
    routing::{delete, get, patch, post, put},
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::{error, info};

#[derive(Serialize)]
pub struct AllUserFetchResponse {
    pub code: u16,
    pub message: String,
    pub data: Vec<UserData>,
    pub total: u64,
}

#[derive(Serialize)]
pub struct SingleUserGetResponse {
    pub code: u16,
    pub message: String,
    pub data: Option<UserData>,
}

#[derive(Serialize)]
pub struct UserCrudResponse {
    pub code: u16,
    pub message: String,
    pub rows_affected: u64,
}

#[derive(Deserialize)]
pub struct MultipleUsersRequest {
    pub ids: Vec<i32>,
}
#[derive(Deserialize)]
pub struct UserPasswordChange {
    pub password: String,
    pub update_by: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UserData {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub username: String,
    pub name: String,
    pub surname: String,
    pub phone: String,
    pub email: String,
    #[serde(skip_deserializing)]
    pub create_date: chrono::NaiveDateTime,
    pub created_by: String,
    pub write_date: Option<chrono::NaiveDateTime>,
    pub update_by: Option<String>,
}

pub fn users_route(pool: PgPool) -> Router {
    Router::new()
        .route("/users", get(get_all_users))
        .route("/users", post(create_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", get(get_user))
        .route("/users/:id", delete(remove_user))
        .route("/users/update_pwd/:id", patch(update_password))
        .route("/users/delete_multiple", delete(remove_multiple_users))
        .with_state(pool.clone())
        .route_layer(middleware::from_fn_with_state(pool, auth_middleware))
}

async fn get_all_users(State(pool): State<PgPool>) -> Result<Json<AllUserFetchResponse>, Response> {
    let users: Vec<UserData> = match sqlx::query_as::<_, UserData>(GET_ALL_USERS)
        .fetch_all(&pool)
        .await
    {
        Ok(u) => u,
        Err(_) => {
            error!(
                "GET_ALL_USERS failed with error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };
    info!("GET_ALL_USERS: {:?}", users);
    if !users.is_empty() {
        Ok(Json(AllUserFetchResponse {
            code: Response::Success.status_code().as_u16(),
            message: Response::Success.message(),
            total: users.len() as u64,
            data: users,
        }))
    } else {
        error!(
            "GET_ALL_USERS failed with error: {:?}",
            Response::NoUserFound
        );
        Err(Response::NoUserFound)
    }
}

async fn get_user(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Json<SingleUserGetResponse>, Response> {
    let user: Option<UserData> = match sqlx::query_as::<_, UserData>(FETCH_SINGLE_USER)
        .bind(id)
        .fetch_optional(&pool)
        .await
    {
        Ok(u) => u,
        Err(_) => {
            error!("GET_USER failed with error: {:?}", Response::InternalError);
            return Err(Response::InternalError);
        }
    };
    info!("GET_USER: {:?}", user);
    if user.is_some() {
        Ok(Json(SingleUserGetResponse {
            code: Response::Success.status_code().as_u16(),
            message: Response::Success.message(),
            data: user,
        }))
    } else {
        error!("GET_USER failed with error: {:?}", Response::NoUserFound);
        Err(Response::NoUserFound)
    }
}

async fn remove_user(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
) -> Result<Json<UserCrudResponse>, Response> {
    let result = match sqlx::query(REMOVE_USER).bind(id).execute(&pool).await {
        Ok(u) => u,
        Err(_) => {
            error!(
                "REMOVE_USER: Record id: {id} failed with error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    if result.rows_affected() == 0 {
        error!(
            "REMOVE_USER: Record id: {id} failed with error: {:?}, rows affected 0",
            Response::NoUserFound
        );
        return Err(Response::NoUserFound);
    }
    info!(
        "REMOVE_USER: Record id: {id} successfully removed: {:?}",
        result
    );
    Ok(Json(UserCrudResponse {
        code: Response::Success.status_code().as_u16(),
        message: "User was successfully deleted".to_string(),
        rows_affected: result.rows_affected(),
    }))
}

async fn remove_multiple_users(
    State(pool): State<PgPool>,
    Json(payload): Json<MultipleUsersRequest>,
) -> Result<Json<UserCrudResponse>, Response> {
    if payload.ids.is_empty() {
        error!(
            "MULTI_REMOVE_USER: {:?}, payload empty",
            Response::BadRequest
        );
        return Err(Response::BadRequest);
    }

    let result = match sqlx::query(REMOVE_MULTIPLE_USERS)
        .bind(&payload.ids)
        .execute(&pool)
        .await
    {
        Ok(u) => u,
        Err(_) => {
            error!(
                "MULTI_REMOVE_USER: failed with error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    if result.rows_affected() == 0 {
        error!(
            "MULTI_REMOVE_USER: failed with error: {:?}, rows affected 0",
            Response::NoUserFound
        );
        return Err(Response::NoUserFound);
    }
    info!(
        "MULTI_REMOVE_USER: Users successfully removed: {:?}",
        result
    );
    Ok(Json(UserCrudResponse {
        code: Response::Success.status_code().as_u16(),
        message: "Successfully Deleted Users".to_string(),
        rows_affected: result.rows_affected(),
    }))
}

async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<UserData>,
) -> Result<Json<UserCrudResponse>, Response> {
    let UserData {
        username,
        name,
        surname,
        phone,
        email,
        created_by,
        ..
    } = payload;

    let plain_pwd: String = format!("{}#01!", &username);

    let hashed_pwd = match hash(plain_pwd, DEFAULT_COST) {
        Ok(r) => r,
        Err(_) => {
            error!(
                "CREATE_USER: Password Hashing failed with error {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    let result = match sqlx::query(CREATE_USER)
        .bind(&username)
        .bind(name)
        .bind(surname)
        .bind(phone)
        .bind(email)
        .bind(hashed_pwd)
        .bind(created_by)
        .execute(&pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            error!(
                "CREATE_USER: failed with error: {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };
    info!("CREATE USER: The user: {username} was successfully created");
    Ok(Json(UserCrudResponse {
        code: Response::Success.status_code().as_u16(),
        message: format!("User: {username} was successfully created"),
        rows_affected: result.rows_affected(),
    }))
}

async fn update_user(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
    Json(payload): Json<UserData>,
) -> Result<Json<UserCrudResponse>, Response> {
    let UserData {
        username,
        name,
        surname,
        phone,
        email,
        update_by,
        ..
    } = payload;

    let write_date = Utc::now().naive_utc();

    let result = match sqlx::query(UPDATE_USER)
        .bind(&username)
        .bind(name)
        .bind(surname)
        .bind(phone)
        .bind(email)
        .bind(update_by)
        .bind(write_date)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            error!(
                "UPDATE_USER: Record id: {id} failed with error:  {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    if result.rows_affected() == 0 {
        error!(
            "UPDATE_USER: Record id: {id} failed with error: {:?} rows affected 0",
            Response::InternalError
        );
        return Err(Response::NoUserFound);
    }

    info!("User: {username} updated successfully");
    Ok(Json(UserCrudResponse {
        code: Response::Success.status_code().as_u16(),
        message: format!("User: {username} updated successfully"),
        rows_affected: result.rows_affected(),
    }))
}

async fn update_password(
    Path(id): Path<i32>,
    State(pool): State<PgPool>,
    Json(payload): Json<UserPasswordChange>,
) -> Result<Json<UserCrudResponse>, Response> {
    let UserPasswordChange {
        password,
        update_by,
    } = payload;

    if password.trim().is_empty() {
        return Err(Response::BadRequest);
    }

    let write_date = Utc::now().naive_utc();
    let hashed_pwd = match hash(password, DEFAULT_COST) {
        Ok(r) => r,
        Err(_) => {
            error!(
                "UPDATE_PASSWORD: Password Hashing failed with error {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    let result = match sqlx::query(UPDATE_USER_PWD)
        .bind(hashed_pwd)
        .bind(update_by)
        .bind(write_date)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            error!(
                "UPDATE_PASSWOR: Record id {id} failed with error {:?}",
                Response::InternalError
            );
            return Err(Response::InternalError);
        }
    };

    info!("User password for record: {id} updated successfully");
    Ok(Json(UserCrudResponse {
        code: Response::Success.status_code().as_u16(),
        message: "Password updated successfully".to_string(),
        rows_affected: result.rows_affected(),
    }))
}

const GET_ALL_USERS: &str = "
SELECT id
    ,username
    ,name
    ,surname
    ,phone
    ,email
    ,create_date
    ,created_by
    ,write_date
    ,update_by
FROM users 
ORDER BY id
";

const FETCH_SINGLE_USER: &str = "
SELECT id
    ,username
    ,name
    ,surname
    ,phone
    ,email
    ,create_date
    ,created_by
    ,write_date
    ,update_by
FROM users 
WHERE id = $1
";

const REMOVE_USER: &str = "
DELETE FROM users
WHERE id = $1
";

const REMOVE_MULTIPLE_USERS: &str = "
DELETE FROM users
WHERE id = ANY($1)
";

const CREATE_USER: &str = "
INSERT INTO users (username, name, surname, phone, email, pwd, created_by)
VALUES($1, $2, $3, $4, $5, $6, $7)
";

const UPDATE_USER: &str = "
UPDATE users
SET username = $1
    ,name = $2
    ,surname = $3
    ,phone = $4
    ,email = $5
    ,update_by = $6
    ,write_date = $7
WHERE id = $8
";

const UPDATE_USER_PWD: &str = "
UPDATE users
SET pwd = $1
    ,update_by = $2
    ,write_date = $3
WHERE id = $4
";
