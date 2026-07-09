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

    let config = ServerConfigurations::bind("0.0.0.0",8084);
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
    children -> ([FrontendController])
}