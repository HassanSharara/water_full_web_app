
pub use redis_service::models::{Session,UserDbModel};
pub type MainDataHolder = _DataHolder;

pub struct _DataHolder {
   pub frontend:Option<FrontendHolder>
}


pub struct FrontendHolder {
   pub session:Session,
}
impl FrontendHolder {
   pub fn user(&self) -> &UserDbModel{
      self.session.user_model.as_ref().unwrap()
   }
}

impl Into<MainDataHolder> for Session {
   fn into(self) -> MainDataHolder {
      MainDataHolder {
         frontend:Some(
            FrontendHolder {
               session:self,
            }
         )
      }
   }
}
