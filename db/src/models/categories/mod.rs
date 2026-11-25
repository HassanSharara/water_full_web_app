use serde::{Deserialize, Serialize};
use sqlx::{ FromRow, MySql};


pub const DEFAULT_PAGINATION_LIMIT:u32 = 1;
#[derive(Debug,FromRow,Serialize,Deserialize,Clone)]
pub struct CategoryModel {
    pub id :i32,
    pub name: String,
    pub description: Option<String>,
    pub t:i32,
    pub price:Option<f32>,
    pub father_id :Option<String>,
}










impl CategoryModel {



    pub async fn get_categories_by_page(mut page:u32)->Result<(i64,Vec<CategoryModel>),sqlx::Error>{
        if page >= 1 { page -= 1;}
        let mut pool = crate::get_pool().await.expect("could not get pool connection");
        let page = page ;
        let q = sqlx::query_as::<MySql,CategoryModel>("SELECT * FROM category WHERE status = 1 ORDER BY created_at DESC LIMIT ? OFFSET ?")
            .bind(&DEFAULT_PAGINATION_LIMIT)
            .bind(&(page))
            .fetch_all(&mut *pool)
            .await?;
        let count:i64 = sqlx::query_scalar("SELECT COUNT(id) as count FROM category where status = 1 ")
            .fetch_one(&mut *pool)
            .await?;
        Ok((count,q))
    }

}