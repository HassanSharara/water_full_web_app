pub mod home;
pub mod profile;
use std::borrow::Cow;
use std::net::SocketAddr;
use water_http::WaterController;
use yarte::Template;
use crate::models::auth::AuthUi;
use data_holder::{FrontendHolder, MainDataHolder, Session, UserDbModel};
use crate::models::sidebar::Page;
pub use home::HomeController;
pub use profile::ProfileController;

pub fn generate_auth_ui() -> String {
    let ui = AuthUi::new();
    ui.call().unwrap()
}

WaterController! {
    holder -> super::MainDataHolder,
    name -> FrontendController,
    functions -> {

        POST_logout -> logout -> logout(context) async {
            let holder: &mut super::FrontendHolder = context.holder.as_mut().unwrap().frontend.as_mut().unwrap();
            let session = &mut holder.session;
            if let Some(user) = session.user_model.as_ref() {
                 if super::Session::destroy_session(&session.session_id).await.is_err()

                || user.remove_remember_token().await.is_err() {
                _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::INTERNAL_SERVER_ERROR).await;
                 return;
                }
            _= context.redirect("login").await;
                return;
            }
            _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::BAD_REQUEST).await;
        }

        POST -> refresh_session -> refresh(context) async {
            let cookie: Option<super::Cow<str>> = context.get_from_headers("Cookie");
            let socket: &super::SocketAddr = context.get_peer_socket();
            let ip = socket.ip();
            if let Ok(session) = super::Session::refresh_session(cookie, ip).await {
                let cookie = session.generate_set_cookie_for_session();
                let refresh_cookie = session.generate_set_cookie_for_refresh_token();
                let mut sender = context.sender();
                sender.send_status_code(http::status_code::HttpStatusCode::OK);
                _= sender.set_header_ef("Set-Cookie", &cookie);
                _= sender.set_header_ef("Set-Cookie", &refresh_cookie);
                _= sender.send_data_as_final_response(http::ResponseData::Slice(&[])).await;
                return;
            }
            _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::EXPECTATION_FAILED).await;
        }

        GET -> login -> login(context) async {
           let mut user_model = None;
           let cookie = context.get_from_headers_as_bytes("Cookie");
            if let Some(cookie) = cookie &&
            let Some(remember_token ) = super::Session::extract_remember_token_cookie(
                unsafe{std::str::from_utf8_unchecked(cookie)}
            ) {
                if let Ok(Some(user)) = super::UserDbModel::get_by_remember_token(remember_token).await {
                    user_model = Some(user);
                }
            }
            let holder: &mut super::FrontendHolder = context.holder.as_mut().unwrap().frontend.as_mut().unwrap();
            let session = &mut holder.session;
            session.user_model = user_model;
            let cookie = session.generate_set_cookie_for_session();
            let refresh_cookie = session.generate_set_cookie_for_refresh_token();
            _= session.save().await;

            let is_user = session.user_model.is_some();

            let mut sender = context.sender();
            if is_user {
                sender.send_status_code(http::status_code::HttpStatusCode::SEE_OTHER);
                sender.set_header_ef("Location",super::Page::Home.get_route());
            }
            else {            sender.send_status_code(http::status_code::HttpStatusCode::OK); }

            _= sender.set_header_ef("Set-Cookie", &cookie);
            _= sender.set_header_ef("Set-Cookie", &refresh_cookie);

            if is_user {
                _= sender.send_data_as_final_response(http::ResponseData::Slice(&[])).await;
                return;
            }

            let html = super::generate_auth_ui();
            _= sender.send_data_as_final_response(http::ResponseData::Slice(html.as_bytes())).await;
        }

        POST -> loginPost -> login_post(context) async {
            use http::request::DynamicBodyMapTrait;
            let body = context.get_body_map().await;

            if let Ok(body) = body {
                let remember = body.get_as_encoded_string("remember").unwrap_or("".into()) == "on";
                let holder: &mut super::FrontendHolder = context.holder.as_mut().unwrap().frontend.as_mut().unwrap();
                let session = &mut holder.session;
                let email = body.get_as_encoded_string("email");
                let password = body.get_as_encoded_string("password");

                // FIXED: Standard tuple matching instead of unstable let-chains
                if let (Some(password), Some(email)) = (password, email) {
                    if let Ok(Some(mut user)) = super::UserDbModel::validate(&email, &password).await {
                        if remember {
                            session.generate_remember_token();
                            if user.save_remember_token(session.remember_token.as_ref().unwrap().to_string()).await.is_err() {
                                _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::INTERNAL_SERVER_ERROR).await;
                                return;
                            }
                        }
                        session.user_model = Some(user);

                        if remember {
                            let rt = session.generate_set_cookie_for_remember_token();
                            _= session.save().await;

                            let mut sender = context.sender();
                            sender.send_status_code(http::status_code::HttpStatusCode::SEE_OTHER);
                            sender.set_header_ef("Location", crate::models::sidebar::Page::Home.get_route().as_bytes());
                            sender.set_header_ef("Set-Cookie", rt);
                            _= sender.send_data_as_final_response(http::ResponseData::Slice(&[])).await;
                            return;
                        }

                        _= session.save().await;
                        _= context.redirect(crate::models::sidebar::Page::Home.get_route()).await;
                        return;
                    }

                    let jso = serde_json::json!({
                        "email": email,
                        "password": password,
                        "csrf_token": session.csrf_token,
                        "session_id": session.session_id
                    });
                    _= context.send_json(&jso).await;
                    return;
                }
            }
            _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::BAD_REQUEST).await;
        }
    },
    prefix -> ("frontend"),
    middleware -> (context {
        let path = context.path();
        if path.ends_with("/refresh_session") {
            return server::MiddlewareResult::Pass;
        }
        let cookie: Option<super::Cow<str>> = context.get_from_headers("Cookie");
        let socket: &super::SocketAddr = context.get_peer_socket();
        let ip = socket.ip();

        if let Ok(session) = super::Session::handle_session(cookie, ip).await {
            if session.user_model.is_some() {
                let path = context.path();
                if path.ends_with("/login") {
                    context.holder = Some(session.into());
                    _= context.redirect("./home").await;
                    return server::MiddlewareResult::Stop;
                }
            }
            context.holder = Some(session.into());
            return server::MiddlewareResult::Pass;
        }
        _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::INTERNAL_SERVER_ERROR).await;
        server::MiddlewareResult::Stop
    }),
    children -> ([
        HomeController,
        ProfileController
    ])
}