use water_http::WaterController;
use data_holder::{MainDataHolder,FrontendHolder};
use crate::models::{home::HomePageUi};
use super::categories::CategoriesController;




WaterController! {
    holder -> super::MainDataHolder,
    name -> HomeController,
    functions -> {
        GET_Home => home => home(context)async{
            use crate::models::helper::HtmlRenderer;
            let user = crate::get_user(context);
            let html = super::HomePageUi::new().generate_html_string(user);
             // println!("{:?}",route!("Home"))
            _= context.send_html_text(&html).await;
        }
    }

    children -> ([
        CategoriesController
    ]),
    middleware -> (context {
        let holder: &super::FrontendHolder = context.holder.as_ref().unwrap().frontend.as_ref().unwrap();
        let session = &holder.session;
        if session.user_model.is_none() {
            _= context.redirect("/frontend/login").await;
            return server::MiddlewareResult::Stop;
        }
        // 2. Rate limit check (e.g., 100 requests per 60 seconds)
       if let Ok(allowed) = session.check_rate_limit(100, 60).await {
           if !allowed {
             _= context.send_status_code_as_final_response(http::status_code::HttpStatusCode::TOO_MANY_REQUESTS).await;
            return server::MiddlewareResult::Stop;
        }
    }




        server::MiddlewareResult::Pass
    })
}