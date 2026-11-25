// use water_http::http::request::DynamicBodyMapTrait;
use water_http::WaterController;
use data_holder::{MainDataHolder};
use crate::models::categories::*;

WaterController! {
    holder -> super::MainDataHolder,
    name -> CategoriesController,
    functions -> {
        
        
        GET -> / -> index(context)async{
            let mut page = 0;
            crate::get_page(context,&mut page);
            let categories = db::models::categories::CategoryModel::get_categories_by_page(page).await;
            match categories {

            Ok((total,cat)) => {
                    let paginate = crate::models::categories::PaginatedCategories::new(
                        cat,
                        total as u32,
                        page,
                    );
                    use crate::models::helper::*;
                    // use crate::models::helper::html_list_model::SupportTableModel;
                    let user = crate::get_user(context);
                    let r = paginate.generate_html_string(user);
                    _= context.send_html_text(&r).await;
                    return
                }
            Err(e) => {
                 println!("{:?}",e);
                                _=context.send_str("invalid categories").await;
                    return
                }
            }

        }

        GET -> create -> create(context) async {
            use crate::models::helper::*;
            let session = &context.holder.as_ref().unwrap().frontend.as_ref().unwrap().session;
            let user = crate::get_user(context);
            let htm = super::CreateCategoryUi::new(None,session.csrf_token.clone()).generate_html_string(&user);
            _= context.send_html_text(&htm).await;
        }




        POST -> createCatPost -> create_post(context) async {

            extract_request_body! {
                context [
                    name -> "name",
                    isDeveloper -> "is_developer"
                ] else {
                    _= context.send_str("invalid input").await;
                }
            }

            let response  = format!("the name input is ( {:?} ) and the developer input is {:?}",name.unwrap(),isDeveloper);
           _= context.send_string_slice(&response).await;
        }
    }

    prefix -> ("categories")
}