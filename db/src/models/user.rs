use sqlx::{ FromRow, MySql};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use crate::models::GeneralDbModelsTrait;
// use redis_service::{FromRedisValue, ToRedisArgs,};

#[derive(Debug, FromRow, Serialize, Deserialize,Clone)]
pub struct UserDbModel {
    pub id: u32,
    pub name: Option<String>,
    pub email: String,
    pub remember_token: Option<String>,
    pub phone: Option<String>,
    #[sqlx(json)]
    pub roles: Option<Vec<String>>
}

// Internal structural helper to grab the hash from the DB
// without cluttering your public JSON UserDbModel struct
#[derive(FromRow,Debug)]
struct AuthRow {
    #[sqlx(flatten)]
    user: UserDbModel,
    password_hash: String,
}

impl UserDbModel {


    pub async fn save_remember_token(&mut self,token:String)->Result<Option<MySqlRow>, sqlx::Error>{
        let mut pool = crate::get_pool().await.expect("could not get pool connection");
        let r = sqlx::query("UPDATE users SET remember_token = ? WHERE id = ?")
            .bind(&token)
            .bind(&self.id)
            .fetch_optional(&mut *pool)
            .await?;
        self.remember_token = Some(token);

        Ok(r)
    }

    pub async fn remove_remember_token(&self)->Result<Option<MySqlRow>, sqlx::Error>{
        let mut pool = crate::get_pool().await.expect("could not get pool connection");
        let r = sqlx::query("UPDATE users SET remember_token = NULL WHERE id = ?")
            .bind(&self.id)
            .fetch_optional(&mut *pool)
            .await?;

        Ok(r)
    }

    pub async fn get_by_remember_token(remember_token:&str)->Result<Option<UserDbModel>, sqlx::Error>{
        let mut pool = crate::get_pool().await.expect("could not get pool connection");
        let auth_data: Option<AuthRow> = sqlx::query_as::<MySql, AuthRow>(
            "SELECT *, password AS password_hash FROM users WHERE remember_token = ?"
        )
            .bind(remember_token)
            .fetch_optional(&mut *pool) // 👈 Use the pool handle directly, NOT a checked out connection!
            .await?;
        if let Some(auth_data) = auth_data {
            return  Ok(Some(auth_data.user))
        }
        Ok(None)
    }
    pub async fn validate(username: &str, password: &str) -> Result<Option<UserDbModel>, sqlx::Error> {
        let mut pool = crate::get_pool().await.expect("could not get pool connection");
        let auth_data: Option<AuthRow> = sqlx::query_as::<MySql, AuthRow>(
            "SELECT *, password AS password_hash FROM users WHERE email = ?"
        )
            .bind(&username)
            .fetch_optional(&mut *pool) // 👈 Use the pool handle directly, NOT a checked out connection!
            .await?;
        // 3. Verify
        if let Some(data) = auth_data {
            if bcrypt::verify(password,&data.password_hash).is_ok() {
                return Ok(Some(data.user));
            }
        }

        Ok(None)
    }}

impl GeneralDbModelsTrait for UserDbModel {
    type Model = Self;

     fn table_name() -> &'static str {
        "users"
    }
}