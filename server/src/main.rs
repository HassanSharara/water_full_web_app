use water_http::{WaterController, InitControllersRoot, RunServer};
use water_http::server::ServerConfigurations;
pub use frontend::{controllers::FrontendController};
pub use data_holder::MainDataHolder;


InitControllersRoot! {
           name:ROOT,
           holder_type:MainDataHolder,
           headers_length:50,
           queries_length:12
}

fn main() {

    redis_service::init_global_pool();
    let port = 8084;
    let config = ServerConfigurations::bind("0.0.0.0",port);
    println!("listening on port {port}");
    RunServer!(
       config,
        ROOT,
        MainController
    )

}


WaterController! {
    holder -> super::MainDataHolder,
    name -> MainController,

    functions -> {

        get -> public/{path}/>> -> public(context) async {
            let path = format!("frontend/public/{}",path);
            if path.contains("../") || path.contains("./") || !path.contains(".") {
                _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::BAD_REQUEST).await;
            }
            response!(context file -> &path);
        }

        GET => "favicon.ico" => favicon(context) async {
            let path = "frontend/public/MainTemplate/favicon.png";
            response!(context file -> &path);
        }
    }
    children -> ([FrontendController]),
    middleware -> ( context {
        use redis_service::models::Session;
        // 1. Bypass static asset routes so heavy asset loading doesn't exhaust the quota
        let path = context.path();
        if path.starts_with("/public/") || path == "/favicon.ico" {
            return server::MiddlewareResult::Pass;
        }

        // 2. Extract Client IP
        let client_ip = context.get_peer_socket().ip();

        // 3. Resolve Session or construct temporary IP-based Session
        // If a session was already loaded into the context/holder, use it:
        let is_allowed = if let Some(holder) = context.holder.as_ref() {
            if let Some(frontend) = holder.frontend.as_ref() {
                // Authenticated or existing session -> uses session.check_rate_limit()
                // Limits by user_model.id if logged in, or IP if guest
                frontend.session.check_rate_limit(120, 60).await.unwrap_or(true)
            } else {
                // Fallback IP rate limit if frontend holder isn't initialized yet
                let temp_session = Session::default(client_ip);
                temp_session.check_rate_limit(60, 60).await.unwrap_or(true)
            }
        } else {
            let temp_session = Session::default(client_ip);
            temp_session.check_rate_limit(60, 60).await.unwrap_or(true)
        };

        if !is_allowed {
            _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::TOO_MANY_REQUESTS).await;
            return server::MiddlewareResult::Stop;
        }

        server::MiddlewareResult::Pass
    })
}