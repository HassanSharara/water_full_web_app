use yarte::Template;
use db::models::user::UserDbModel;
use crate::models::helper::AssetsHelper;
use crate::models::sidebar::{Page, Sidebar};

#[derive(Template)]
#[template(path = "MainTemplate/Layout/main_layout.html.hbs")]
pub struct MainLayout<'a>{
    pub user:&'a UserDbModel,
    pub body_content: &'a str,
    pub sidebar:Sidebar,
}



impl<'a>  MainLayout<'a> {

    pub const fn default_sidebar()->Sidebar{
        Sidebar::default()
    }
    pub const fn  app_name()->&'static str{
      "RoyalBoard App"
    }
    pub fn new (body_content: &'a str,user_db_model: &'a UserDbModel)->MainLayout<'a>{
        MainLayout { body_content ,user:user_db_model,sidebar:Self::default_sidebar()}
    }
    pub fn new_with_page (body_content: &'a str,user_db_model: &'a UserDbModel,page: Page)->MainLayout<'a>{
        let mut sidebar = Sidebar::default();
        sidebar.current_page = page;
        MainLayout { body_content ,user:user_db_model,sidebar}
    }

}
impl<'a> AssetsHelper for MainLayout<'a> {}