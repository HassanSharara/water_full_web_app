use yarte::Template;
use crate::models::helper::{AssetsHelper};
use crate::models::layout::MainLayout;

#[derive(Template)]
#[template(path = "UI/auth/auth.html.hbs")]
pub struct AuthUi {
    error_message:Option<String>
}

impl AuthUi {

    pub const fn new()->Self{
        Self{error_message:None}
    }
    pub const fn app_name()->&'static str{
        MainLayout::app_name()
    }
}

impl AssetsHelper for AuthUi {}
