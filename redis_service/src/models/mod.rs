use std::borrow::Cow;
use std::net::IpAddr;
use redis::AsyncCommands;
use redis_macros::{FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

pub use db::models::user::UserDbModel;

pub fn generate_random_session() -> String {
    format!("session_{}", uuid::Uuid::new_v4())
}

pub fn generate_random_csrf() -> String {
    format!("csrf_{}", uuid::Uuid::new_v4())
}
pub fn generate_remember_token() -> String {
    format!("remember_{}", uuid::Uuid::new_v4())
}

pub fn generate_refresh_token() -> String {
    format!("refresh_{}", uuid::Uuid::new_v4())
}

const TTL: i64 = 1800;

#[derive(FromRedisValue, ToRedisArgs, Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub refresh_token: String,
    pub csrf_token: String,
    pub remote_ip: IpAddr,
    pub remember_token: Option<String>,
    pub user_model: Option<UserDbModel>,
    pub cached: bool,
    #[serde(default)]
    pub is_dying: bool,
}

impl Session {
    pub fn generate_set_cookie_for_session(&self) -> String {
        format!("session_id={}; Path=/frontend; HttpOnly; SameSite=Strict; Secure; Max-Age=900", self.session_id)
    }

    pub fn generate_set_cookie_for_refresh_token(&self) -> String {
        format!("refresh_token={}; Path=/frontend; HttpOnly; SameSite=Strict; Secure; Max-Age=900", self.refresh_token)
    } pub fn generate_set_cookie_for_remember_token(&self) -> String {
        format!("remember_token={}; Path=/frontend/login; HttpOnly; SameSite=Strict; Secure; Max-Age=80000", self.remember_token.as_ref().unwrap())
    }

    pub fn generate_logout_cookie_for_session() -> String {
        "session_id=; Path=/frontend; HttpOnly; SameSite=Strict; Secure; Max-Age=0".to_string()
    }

    pub fn generate_logout_cookie_for_refresh_token() -> String {
        "refresh_token=; Path=/frontend; HttpOnly; SameSite=Strict; Secure; Max-Age=0".to_string()
    }

    pub fn generate_remember_token(&mut self){
        self.remember_token = Some(generate_remember_token());
    }
    pub fn new(remote_ip: IpAddr, csrf_token: String, refresh_token: String, session_id: String) -> Session {
        Session {
            session_id,
            csrf_token,
            refresh_token,
            remote_ip,
            remember_token:None,
            user_model: None,
            cached: false,
            is_dying: false,
        }
    }
    #[inline(always)]
    pub fn extract_remember_token_cookie(cookie_header: &str) -> Option<&str> {
        let mut current = cookie_header;
        while !current.is_empty() {
            let (pair, remainder) = match current.find(';') {
                Some(idx) => (&current[..idx], &current[idx + 1..]),
                None => (current, ""),
            };
            current = remainder;
            let pair = pair.trim_start();

            if pair.starts_with("remember_token=") {
                return  Some(&pair["remember_token=".len()..])
            }
        }
        None
    }

    pub fn default(remote_ip: IpAddr) -> Session {
        Self::new(remote_ip, generate_random_csrf(), generate_refresh_token(), generate_random_session())
    }

    async fn get_session_from_cache(id: &str, ip_addr: IpAddr) -> Result<Option<Session>, ()> {
        let pool = crate::get_redis();
        let mut connection = pool.get().await.map_err(|_| ())?;
        let session: Option<Session> = connection.get(id).await.map_err(|_| ())?;
        if let Some(mut session) = session {
            session.cached = true;
            if session.remote_ip == ip_addr {
                let _: bool = redis::AsyncTypedCommands::expire(&mut connection, id, TTL).await.map_err(|_| ())?;
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    pub async fn handle_session(cookie: Option<Cow<'_, str>>, ip: IpAddr) -> Result<Session, ()> {
        if let Some(cookie) = cookie {
            if let Some(sid) = extract_session_id_fast(&cookie) {
                if let Some(session) = Self::get_session_from_cache(sid, ip).await? {
                    return Ok(session);
                }
            }
        }
        let new_session = Self::default(ip);
        Self::save_to_cache(&new_session).await?;
        Ok(new_session)
    }

    pub async fn refresh_session(cookie: Option<Cow<'_, str>>, ip: IpAddr) -> Result<Session, ()> {
        if let Some(cookie) = cookie {
            if let (Some(session_id), Some(refresh_token)) = extract_auth_cookies(&cookie) {
                let pool = crate::get_redis();
                let mut connection = pool.get().await.map_err(|_| ())?;
                let session: Option<Session> = connection.get(session_id).await.map_err(|_| ())?;

                if let Some(old_session) = session {
                    if old_session.refresh_token == refresh_token && old_session.remote_ip == ip {
                        let _: bool = AsyncCommands::expire(&mut connection, session_id, 50)
                            .await
                            .map_err(|_| ())?;

                        let mut new_session = Session::default(ip);
                        new_session.user_model = old_session.user_model;

                        Self::save_to_cache(&new_session).await?;
                        return Ok(new_session);
                    }
                }
            }
        }
        Err(())
    }

    pub async fn save(&self) {
        let _ = Self::save_to_cache(self).await;
    }

    async fn save_to_cache(session: &Session) -> Result<(), ()> {
        let pool = crate::get_redis();
        let mut connection = pool.get().await.map_err(|_| ())?;
        let _: () = connection
            .set_ex(&session.session_id, session, TTL as u64)
            .await
            .map_err(|_| ())?;
        Ok(())
    }

    pub async fn destroy_session(session_id: &str) -> Result<(), ()> {
        let pool = crate::get_redis();
        let mut connection = pool.get().await.map_err(|_| ())?;
        let _: i32 = AsyncCommands::del(&mut connection, session_id).await.map_err(|_| ())?;
        Ok(())
    }
}

#[inline(always)]
fn extract_session_id_fast(cookie_header: &str) -> Option<&str> {
    let mut current = cookie_header;
    while !current.is_empty() {
        let (pair, remainder) = match current.find(';') {
            Some(idx) => (&current[..idx], &current[idx + 1..]),
            None => (current, ""),
        };
        current = remainder;
        let pair = pair.trim_start();
        if pair.starts_with("session_id=") {
            return Some(&pair["session_id=".len()..]);
        }
    }
    None
}

#[inline(always)]
fn extract_auth_cookies(cookie_header: &str) -> (Option<&str>, Option<&str>) {
    let mut session_id = None;
    let mut refresh_token = None;
    let mut current = cookie_header;

    while !current.is_empty() {
        let (pair, remainder) = match current.find(';') {
            Some(idx) => (&current[..idx], &current[idx + 1..]),
            None => (current, ""),
        };
        current = remainder;
        let pair = pair.trim_start();

        if pair.starts_with("session_id=") {
            session_id = Some(&pair["session_id=".len()..]);
        } else if pair.starts_with("refresh_token=") {
            refresh_token = Some(&pair["refresh_token=".len()..]);
        }

        if session_id.is_some() && refresh_token.is_some() {
            break;
        }
    }
    (session_id, refresh_token)
}
