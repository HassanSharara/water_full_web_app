
use yarte::Template;
use crate::models::helper::AssetsHelper;

#[derive(Template)]
#[template(path = "UI/Home/home.html.hbs")]
pub struct HomePageUi {}

impl HomePageUi {
    
    
    pub fn new() -> HomePageUi {
        HomePageUi {}
    }
}
impl AssetsHelper for HomePageUi {}