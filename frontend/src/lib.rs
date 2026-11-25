use std::borrow::Cow;
use water_http::server::HttpContext;
use data_holder::Session;
use db::models::user::UserDbModel;

pub mod models;
pub mod controllers;

pub fn get_user<'a,const H:usize,const Q:usize>(context:&'a HttpContext<'a,data_holder::MainDataHolder,H,Q>)-> &'a UserDbModel{
    // let session =& context.holder.as_ref().unwrap().frontend.as_ref().unwrap().session;
    let session = get_session(context);
    session.user_model.as_ref().unwrap()
}

pub fn get_session<'a,const H:usize,const Q:usize>(context:&'a HttpContext<'a,data_holder::MainDataHolder,H,Q>)-> &'a Session{
    & context.holder.as_ref().unwrap().frontend.as_ref().unwrap().session
}

fn get_page<'a,const H:usize,const Q:usize>(context:&'a HttpContext<'a,data_holder::MainDataHolder,H,Q>,page:&mut u32){
    let p = context.get_from_path_query("page").unwrap_or(Cow::Borrowed("1"));
    use std::str::FromStr;
    * page = usize::from_str(p.as_ref()).unwrap_or(0) as u32;
}