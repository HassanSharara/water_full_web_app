use water_http::WaterController;
use data_holder::{MainDataHolder,FrontendHolder};
use crate::models::{home::HomePageUi};





WaterController! {
    holder -> super::MainDataHolder,
    name -> HomeController,
    functions -> {
        GET_Home => home => home(context)async{
            use crate::models::helper::HtmlRenderer;
            let user = crate::get_user(context);
            let html = super::HomePageUi::new().generate_html_string(user);
            _= context.send_html_text(&html).await;
        }
    }

    middleware -> (context {
        let holder: &super::FrontendHolder = context.holder.as_ref().unwrap().frontend.as_ref().unwrap();
        let session = &holder.session;
        if session.user_model.is_none() {
            _= context.redirect("/frontend/login").await;
            return server::MiddlewareResult::Stop;
        }
        server::MiddlewareResult::Pass
    })
}