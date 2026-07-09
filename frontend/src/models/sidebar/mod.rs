

 pub struct Sidebar {
     pub current_page:Page,
     pub items:&'static [SidebarItem]
 }


 impl Sidebar {

     pub const fn default()->Sidebar{
         Sidebar {
             current_page:Page::Home,
             items:&[
                 SidebarItem{
                     page:Page::Home,
                     icon:"monitor",
                     title:"Dashboard",
                     children:&[]
                 },
                 
                  SidebarItem{
                     page:Page::Categories,
                     icon:"list",
                     title:"الاقسام",
                     children:&[

                          SidebarItem {
                              page:Page::Categories,
                              icon:"",
                              title:"جميع الاقسام",
                              children:&[]
                          },
                          SidebarItem {
                              page:Page::CreateCategory,
                              icon:"",
                              title:"انشاء قسم",
                              children:&[]
                          }
                     ]
                 },
                 
                 
             ]
         }
     }
 }
 pub struct SidebarItem {
     pub page:Page,
     pub icon:&'static str,
     pub title:&'static str,
     pub children:&'static [SidebarItem],
 }


 impl SidebarItem {

     pub fn active(&self,page:&Page)->&'static str{
        if  self.page == *page { return "active"}
         ""
     }
 }


#[derive(PartialEq)]
 pub enum Page {
     Home,
     Categories,
     CreateCategory,
 }
 
 impl Page {
     
     pub const fn get_route(&self)->&'static str{
         match self {
             Page::Home => {"/frontend/home"},
             Page::Categories => {"/frontend/categories"},
             Page::CreateCategory => {"/frontend/createCategory"}
         }
     }
 }