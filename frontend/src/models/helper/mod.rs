use std::collections::HashMap;
use yarte::Template;
use db::models::user::UserDbModel;
use crate::models::layout::MainLayout;
use crate::models::sidebar::Page;

pub trait AssetsHelper {
     fn asset_path(&self,path:&str)->String{
         format!("/public/MainTemplate/{path}")
     }
     fn route(&self,key:&str)->String{
        self.route_with(key,None)
    } 
    fn route_with(&self,key:&str,map:Option<HashMap<&str,&str>>)->String{
        water_http::get_route(key, map).expect(key)
    }

}

pub trait HtmlRenderer {
    
    fn generate_html_string(&self,u:&UserDbModel) -> String;
    fn generate_html_string_with_active(&self,u:&UserDbModel,page:Page) -> String;
}
impl <T:Template> HtmlRenderer for T {

     fn generate_html_string(&self,u:&UserDbModel)->String{
         let x = self.call().unwrap();
         let main = MainLayout::new(&x,u);
         main.call().unwrap()
    }

    fn generate_html_string_with_active(&self, u: &UserDbModel, page: Page) -> String {
        let x = self.call().unwrap();
        let main = MainLayout::new_with_page(&x,u,page);
        main.call().unwrap()
    }
}


// pub trait MainLayoutBuilder {
//
//     fn build_layout<'a>(&self)->MainLayout<'a>;
// }