use water_http::server::HttpContext;
use db::models::user::UserDbModel;

pub mod models;
pub mod controllers;

pub fn get_user<'a,const H:usize,const Q:usize>(context:&'a HttpContext<'a,data_holder::MainDataHolder,H,Q>)-> &'a UserDbModel{
    let session =& context.holder.as_ref().unwrap().frontend.as_ref().unwrap().session;
    session.user_model.as_ref().unwrap()
}