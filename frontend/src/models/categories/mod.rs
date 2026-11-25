use std::borrow::Cow;
use yarte::Template;
use db::models::categories::{CategoryModel, DEFAULT_PAGINATION_LIMIT};
use crate::models::helper::{AssetsHelper};
use crate::models::helper::{create_html_model::{InputType, SingleInputModel, SupportCreateHtmlModel},
                            html_list_model::SupportTableModel
};
use crate::models::helper::html_list_model::{Cell, Pagination, Variant};
use crate::models::sidebar::Page;

// #[derive(Template)]
// #[template(path = "UI/Categories/categories_ui.hbs")]
// pub struct  CategoriesUiModel;


#[derive(Template)]
#[template(path = "UI/Categories/categories_ui.hbs")]
pub struct PaginatedCategories<'a> {
    categories: Vec<CategoryModel>,
    total: u32,
    pagination:Pagination<'a>
}

impl <'a> PaginatedCategories<'a> {
    pub fn new(categories:Vec<CategoryModel>,total:u32,current_page:u32) -> Self {
        let base_url = Cow::Borrowed(Page::Categories.get_route());
        let last_page = total / (DEFAULT_PAGINATION_LIMIT ) ;
        let pagination = Pagination {
            current_page,
            last_page,
            base_url
        };
         PaginatedCategories {
            categories,
            total,
            pagination
        }
    }

    pub fn get_total_records(&self)->u32 {self.total}
}

impl <'a> SupportTableModel for PaginatedCategories<'a> {
    fn get_page_title(&self) -> &'static str {
        " الاقسام"
    }


    fn get_columns(&self) -> Vec<Cow<'_, str>> {
        [
            Cow::Borrowed("التسلسل"),
            Cow::Borrowed("اسم القسم"),
            Cow::Borrowed("الوصف"),
            Cow::Borrowed("التفرع"),
        ].to_vec()
    }

    fn get_rows(&self) -> Vec<Vec<Cell<'_>>> {
        self.categories.iter()
            .enumerate()
            .map(|(i,c)|
                vec![
                    Cell::Text(Cow::Owned((i + 1).to_string())),
                    Cell::Text(Cow::Borrowed(&c.name)),
                    Cell::Text(Cow::Borrowed(&(c.description.as_ref().map(|s| s.as_ref()).unwrap_or("")))),
                    Cell::Button {
                        label:Cow::Borrowed("التفرعات"),
                        href:Cow::Owned(
                            format!("{}",Page::Categories.get_route())
                        ),
                        variant: Variant::Primary,
                    }
                ]
            )
            .collect()
    }

    fn get_pagination(&self) -> Option<Pagination<'_>> {
        Some(self.pagination.clone())
    }
}




#[derive(Template)]
#[template(path = "UI/Helpers/create.hbs")]
pub struct CreateCategoryUi {
    pub category: Option<CategoryModel>,
    pub csrf_token: String,
}
impl  CreateCategoryUi {
    pub fn new(category: Option<CategoryModel>,csrf_token:String) -> Self {
        Self { category,csrf_token }
    }
}
impl AssetsHelper for CreateCategoryUi {}

impl SupportCreateHtmlModel for CreateCategoryUi {
    fn get_page_title(&self) -> &'static str {
        "انشاء قسم"
    }

    fn get_action_route(&self) -> &str {
        Page::CreateCategoryPost.get_route()
    }

    fn get_inputs(&self) -> Vec<SingleInputModel<'_>> {
        vec![
            SingleInputModel {
            input_type: InputType::Text {
                value: if let Some(category) = &self.category {
                    Cow::Borrowed(category.name.as_ref())
                } else {
                    Cow::Borrowed("")
                },
                placeholder: Some(Cow::Owned("".to_owned())),
            },
            title: Cow::Borrowed("الاسم"),
            name: Cow::Borrowed("name"),
            required: false,
            disabled: false,
        },
            SingleInputModel {
                input_type:InputType::Checkbox {
                    checked: false,
                },
                title: Cow::Borrowed("هل انت المطور ؟"),
                name: Cow::Borrowed("is_developer"),
                required: true,
                disabled: false,
            },
        ]
    }

    fn get_csrf_token(&self) -> Option<&str> {
        Some(&self.csrf_token)
    }
}