use yarte::Template;
use crate::models::helper::{AssetsHelper};
use crate::models::layout::MainLayout;

#[derive(Template)]
#[template(path = "UI/auth/auth.html.hbs")]
pub struct AuthUi {
    pub error_message:Option<String>,
    pub csrf_token:String,
}

impl AuthUi {

    pub const fn new(csrf_token:String)->Self{
        Self{error_message:None,csrf_token}
    }
    pub const fn app_name()->&'static str{
        MainLayout::app_name()
    }
}

impl AssetsHelper for AuthUi {}
