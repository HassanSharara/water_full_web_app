use std::borrow::Cow;
use std::fmt::Write;

// If this lives in the same crate as your form module, delete this copy
// and `use` the existing `esc_into` instead.
#[inline]
fn esc_into(out: &mut String, s: &str) {
    if !s.bytes().any(|b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\'')) {
        out.push_str(s);
        return;
    }
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
}

// ───────────────────────── Building blocks ─────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum Variant {
    Primary,
    Success,
    Danger,
}

impl Variant {
    fn btn_class(self) -> &'static str {
        match self {
            Variant::Primary => "btn btn-primary",
            Variant::Success => "btn btn-success",
            Variant::Danger => "btn btn-danger",
        }
    }

    fn badge_class(self) -> &'static str {
        match self {
            Variant::Primary => "badge badge-primary",
            Variant::Success => "badge badge-success",
            Variant::Danger => "badge badge-danger",
        }
    }
}

/// One `<td>`. Everything is escaped on output.
#[derive(Debug, Clone)]
pub enum Cell<'a> {
    Empty,
    Text(Cow<'a, str>),
    Image {
        src: Option<Cow<'a, str>>,
    },
    Badge {
        text: Cow<'a, str>,
        variant: Variant,
    },
    Button {
        href: Cow<'a, str>,
        label: Cow<'a, str>,
        variant: Variant,
    },
}

impl<'a> Cell<'a> {
    fn render_into(&self, out: &mut String) {
        out.push_str("<td>");
        match self {
            Cell::Empty => {}
            Cell::Text(t) => esc_into(out, t),
            Cell::Image { src } => {
                if let Some(src) = src {
                    out.push_str(r#"<br><img class="img-responsive imgcover" src=""#);
                    esc_into(out, src);
                    out.push_str(r#""><br>"#);
                }
            }
            Cell::Badge { text, variant } => {
                out.push_str(r#"<div class=""#);
                out.push_str(variant.badge_class());
                out.push_str(r#"">"#);
                esc_into(out, text);
                out.push_str("</div>");
            }
            Cell::Button { href, label, variant } => {
                out.push_str(r#"<a href=""#);
                esc_into(out, href);
                out.push_str(r#"" class=""#);
                out.push_str(variant.btn_class());
                out.push_str(r#"">"#);
                esc_into(out, label);
                out.push_str("</a>");
            }
        }
        out.push_str("</td>");
    }
}

/// The optional button in the card header (e.g. "create sub-category").
#[derive(Debug, Clone)]
pub struct HeaderAction<'a> {
    pub label: Cow<'a, str>,
    pub href: Cow<'a, str>,
}

/// Equivalent of Laravel's `$model->links()`.
#[derive(Debug, Clone)]
pub struct Pagination<'a> {
    pub current_page: u32,
    pub last_page: u32,
    /// e.g. "/categories" or "/categories?id=3" – `page=N` is appended.
    pub base_url: Cow<'a, str>,
}

impl<'a> Pagination<'a> {
    fn item(&self, out: &mut String, label: &str, page: u32, disabled: bool, active: bool) {
        out.push_str(r#"<li class="page-item"#);
        if disabled {
            out.push_str(" disabled");
        }
        if active {
            out.push_str(" active");
        }
        out.push_str(r#""><a class="page-link" href=""#);
        if disabled {
            out.push('#');
        } else {
            esc_into(out, &self.base_url);
            out.push(if self.base_url.contains('?') { '&' } else { '?' });
            let _ = write!(out, "page={}", page);
        }
        out.push_str(r#"">"#);
        esc_into(out, label);
        out.push_str("</a></li>");
    }

    fn render_into(&self, out: &mut String) {
        if self.last_page <= 1 {
            return;
        }
        let cur = self.current_page.clamp(1, self.last_page);
        let from = cur.saturating_sub(2).max(1);
        let to = (cur + 2).min(self.last_page);

        out.push_str(r#"<nav><ul class="pagination">"#);
        self.item(out, "«", cur.saturating_sub(1), cur <= 1, false);
        for p in from..=to {
            let label = p.to_string();
            self.item(out, &label, p, false, p == cur);
        }
        self.item(out, "»", cur + 1, cur >= self.last_page, false);
        out.push_str("</ul></nav>");
    }
}

// ───────────────────────── The trait (used by the template) ─────────────────────────

pub trait SupportTableModel {
    fn get_page_title(&self) -> &'static str;
    fn get_columns(&self) -> Vec<Cow<'_, str>>;
    fn get_rows(&self) -> Vec<Vec<Cell<'_>>>;

    fn get_header_action(&self) -> Option<HeaderAction<'_>> {
        None
    }

    fn get_pagination(&self) -> Option<Pagination<'_>> ;

    /// `<th>…</th>` for every column (goes inside `<tr>` in the template).
    fn get_columns_html(&self) -> String {
        let columns = self.get_columns();
        let mut out = String::with_capacity(columns.iter().map(|c| c.len() + 16).sum());
        for c in &columns {
            out.push_str("<th>");
            esc_into(&mut out, c);
            out.push_str("</th>");
        }
        out
    }

    /// `<tr>…</tr>` for every row (goes inside `<tbody>` in the template).
    fn get_rows_html(&self) -> String {
        let rows = self.get_rows();
        let mut out = String::with_capacity(rows.len() * 600);
        for row in &rows {
            out.push_str("<tr>");
            for cell in row {
                cell.render_into(&mut out);
            }
            out.push_str("</tr>");
        }
        out
    }

    /// Empty string when there is no action button.
    fn get_header_action_html(&self) -> String {
        let mut out = String::new();
        if let Some(a) = self.get_header_action() {
            out.push_str(r#"<div class="card-header-action"><a href=""#);
            esc_into(&mut out, &a.href);
            out.push_str(r#"" class="btn btn-primary">"#);
            esc_into(&mut out, &a.label);
            out.push_str("</a></div>");
        }
        out
    }

    /// Empty string when there is only one page.
    fn get_pagination_html(&self) -> String {
        let mut out = String::new();
        if let Some(p) = self.get_pagination() {
            p.render_into(&mut out);
        }
        out
    }
}

// ───────────────────────── Example: categories ─────────────────────────

pub struct Category {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub image_path: Option<String>,
    pub active: bool,
}

pub struct CategoryListPage {
    pub categories: Vec<Category>,
    pub parent_id: Option<u64>,
    pub page: u32,
    pub per_page: u32,
    pub last_page: u32,
    pub upload_path: String,
}

impl SupportTableModel for CategoryListPage {
    fn get_page_title(&self) -> &'static str {
        "جميع التخصصات"
    }

    fn get_columns(&self) -> Vec<Cow<'_, str>> {
        [
            "التسلسل", "اسم التخصص", "وصف التخصص", "صورة التخصص", "الحالة",
            "الاقسام", "امتحانات", "تعديل", "حذف",
        ]
            .into_iter()
            .map(Cow::Borrowed)
            .collect()
    }

    fn get_rows(&self) -> Vec<Vec<Cell<'_>>> {
        let offset = (self.page.saturating_sub(1) * self.per_page) as usize;

        self.categories
            .iter()
            .enumerate()
            .map(|(i, c)| {
                vec![
                    Cell::Text(Cow::Owned((offset + i + 1).to_string())),
                    Cell::Text(Cow::Borrowed(&c.name)),
                    Cell::Text(Cow::Borrowed(&c.description)),
                    Cell::Image {
                        src: c
                            .image_path
                            .as_ref()
                            .map(|p| Cow::Owned(format!("{}/{}", self.upload_path, p))),
                    },
                    if c.active {
                        Cell::Badge { text: "نشط".into(), variant: Variant::Success }
                    } else {
                        Cell::Badge { text: "غير نشط".into(), variant: Variant::Danger }
                    },
                    Cell::Button {
                        href: Cow::Owned(format!("/categories/{}/sections", c.id)),
                        label: "الاقسام".into(),
                        variant: Variant::Primary,
                    },
                    Cell::Button {
                        href: Cow::Owned(format!("/categories/{}/exams", c.id)),
                        label: "تقسيم الامتحانات".into(),
                        variant: Variant::Primary,
                    },
                    Cell::Button {
                        href: Cow::Owned(format!("/categories/{}/edit", c.id)),
                        label: "تعديل".into(),
                        variant: Variant::Primary,
                    },
                    Cell::Button {
                        href: Cow::Owned(format!("/categories/{}/delete", c.id)),
                        label: "حذف".into(),
                        variant: Variant::Danger,
                    },
                ]
            })
            .collect()
    }

    fn get_header_action(&self) -> Option<HeaderAction<'_>> {
        self.parent_id.map(|id| HeaderAction {
            label: "انشاء قسم فرعي".into(),
            href: Cow::Owned(format!("/categories/create?id={}", id)),
        })
    }

    fn get_pagination(&self) -> Option<Pagination<'_>> {
        let base_url = match self.parent_id {
            Some(id) => format!("/subcategories/{}", id),
            None => "/categories".to_string(),
        };
        Some(Pagination {
            current_page: self.page,
            last_page: self.last_page,
            base_url: Cow::Owned(base_url),
        })
    }
}

// use std::borrow::Cow;
// use std::fmt::Write;
//
// // If this lives in the same crate as your form module, delete this copy
// // and `use` the existing `esc_into` instead.
// #[inline]
// fn esc_into(out: &mut String, s: &str) {
//     if !s.bytes().any(|b| matches!(b, b'&' | b'<' | b'>' | b'"' | b'\'')) {
//         out.push_str(s);
//         return;
//     }
//     for c in s.chars() {
//         match c {
//             '&' => out.push_str("&amp;"),
//             '<' => out.push_str("&lt;"),
//             '>' => out.push_str("&gt;"),
//             '"' => out.push_str("&quot;"),
//             '\'' => out.push_str("&#39;"),
//             _ => out.push(c),
//         }
//     }
// }
//
// // ───────────────────────── Building blocks ─────────────────────────
//
// #[derive(Debug, Clone, Copy)]
// pub enum Variant {
//     Primary,
//     Success,
//     Danger,
// }
//
// impl Variant {
//     fn btn_class(self) -> &'static str {
//         match self {
//             Variant::Primary => "btn btn-primary",
//             Variant::Success => "btn btn-success",
//             Variant::Danger => "btn btn-danger",
//         }
//     }
//
//     fn badge_class(self) -> &'static str {
//         match self {
//             Variant::Primary => "badge badge-primary",
//             Variant::Success => "badge badge-success",
//             Variant::Danger => "badge badge-danger",
//         }
//     }
// }
//
// /// One `<td>`. Everything is escaped on output.
// #[derive(Debug, Clone)]
// pub enum Cell<'a> {
//     Empty,
//     Text(Cow<'a, str>),
//     Image {
//         src: Option<Cow<'a, str>>,
//     },
//     Badge {
//         text: Cow<'a, str>,
//         variant: Variant,
//     },
//     Button {
//         href: Cow<'a, str>,
//         label: Cow<'a, str>,
//         variant: Variant,
//     },
// }
//
// impl<'a> Cell<'a> {
//     fn render_into(&self, out: &mut String) {
//         out.push_str("<td>");
//         match self {
//             Cell::Empty => {}
//             Cell::Text(t) => esc_into(out, t),
//             Cell::Image { src } => {
//                 if let Some(src) = src {
//                     out.push_str(r#"<br><img class="img-responsive imgcover" src=""#);
//                     esc_into(out, src);
//                     out.push_str(r#""><br>"#);
//                 }
//             }
//             Cell::Badge { text, variant } => {
//                 out.push_str(r#"<div class=""#);
//                 out.push_str(variant.badge_class());
//                 out.push_str(r#"">"#);
//                 esc_into(out, text);
//                 out.push_str("</div>");
//             }
//             Cell::Button { href, label, variant } => {
//                 out.push_str(r#"<a href=""#);
//                 esc_into(out, href);
//                 out.push_str(r#"" class=""#);
//                 out.push_str(variant.btn_class());
//                 out.push_str(r#"">"#);
//                 esc_into(out, label);
//                 out.push_str("</a>");
//             }
//         }
//         out.push_str("</td>");
//     }
// }
//
// /// Equivalent of Laravel's `$model->links()`.
// #[derive(Debug, Clone)]
// pub struct Pagination<'a> {
//     pub current_page: u32,
//     pub last_page: u32,
//     /// e.g. "/categories" or "/categories?id=3" – `page=N` is appended.
//     pub base_url: Cow<'a, str>,
// }
//
// impl<'a> Pagination<'a> {
//     fn item(&self, out: &mut String, label: &str, page: u32, disabled: bool, active: bool) {
//         out.push_str(r#"<li class="page-item"#);
//         if disabled {
//             out.push_str(" disabled");
//         }
//         if active {
//             out.push_str(" active");
//         }
//         out.push_str(r#""><a class="page-link" href=""#);
//         if disabled {
//             out.push('#');
//         } else {
//             esc_into(out, &self.base_url);
//             out.push(if self.base_url.contains('?') { '&' } else { '?' });
//             let _ = write!(out, "page={}", page);
//         }
//         out.push_str(r#"">"#);
//         esc_into(out, label);
//         out.push_str("</a></li>");
//     }
//
//     fn render_into(&self, out: &mut String) {
//         if self.last_page <= 1 {
//             return;
//         }
//         let cur = self.current_page.clamp(1, self.last_page);
//         let from = cur.saturating_sub(2).max(1);
//         let to = (cur + 2).min(self.last_page);
//
//         out.push_str(r#"<nav><ul class="pagination">"#);
//         self.item(out, "«", cur.saturating_sub(1), cur <= 1, false);
//         for p in from..=to {
//             let label = p.to_string();
//             self.item(out, &label, p, false, p == cur);
//         }
//         self.item(out, "»", cur + 1, cur >= self.last_page, false);
//         out.push_str("</ul></nav>");
//     }
// }
//
// // ───────────────────────── The table page ─────────────────────────
//
// #[derive(Debug, Clone)]
// pub struct HeaderAction<'a> {
//     pub label: Cow<'a, str>,
//     pub href: Cow<'a, str>,
// }
//
// #[derive(Debug, Clone)]
// pub struct TableModel<'a> {
//     pub title: Cow<'a, str>,
//     pub header_action: Option<HeaderAction<'a>>,
//     pub columns: Vec<Cow<'a, str>>,
//     pub rows: Vec<Vec<Cell<'a>>>,
//     pub pagination: Option<Pagination<'a>>,
// }
//
// impl<'a> TableModel<'a> {
//     fn estimated_size(&self) -> usize {
//         512 + self.columns.len() * 32 + self.rows.len() * self.columns.len().max(1) * 80
//     }
//
//     pub fn render(&self) -> String {
//         let mut out = String::with_capacity(self.estimated_size());
//         self.render_into(&mut out);
//         out
//     }
//
//     pub fn render_into(&self, out: &mut String) {
//         out.push_str(r#"<div class="section-body"><div class="row"><div class="col-12"><div class="card">"#);
//
//         // card header
//         out.push_str(r#"<div class="card-header"><h4>"#);
//         esc_into(out, &self.title);
//         out.push_str("</h4>");
//         if let Some(a) = &self.header_action {
//             out.push_str(r#"<div class="card-header-action"><a href=""#);
//             esc_into(out, &a.href);
//             out.push_str(r#"" class="btn btn-primary">"#);
//             esc_into(out, &a.label);
//             out.push_str("</a></div>");
//         }
//         out.push_str("</div>");
//
//         // table
//         out.push_str(r#"<div class="card-body p-0"><div class="table-responsive"><table class="table table-striped"><thead><tr>"#);
//         for c in &self.columns {
//             out.push_str("<th>");
//             esc_into(out, c);
//             out.push_str("</th>");
//         }
//         out.push_str("</tr></thead><tbody>");
//
//         for row in &self.rows {
//             out.push_str("<tr>");
//             for cell in row {
//                 cell.render_into(out);
//             }
//             out.push_str("</tr>");
//         }
//         out.push_str("</tbody></table></div></div></div></div></div>");
//
//         if let Some(p) = &self.pagination {
//             p.render_into(out);
//         }
//         out.push_str("</div>");
//     }
// }
//
// pub trait SupportTableModel {
//     fn get_page_title(&self) -> &'static str;
//     fn get_table(&self) -> TableModel<'_>;
//
//     fn get_table_html(&self) -> String {
//         self.get_table().render()
//     }
// }
//
//
//
// // pub struct CategoryListPage {
// //     pub categories: Vec<Category>,
// //     pub parent_id: Option<u64>,
// //     pub page: u32,
// //     pub per_page: u32,
// //     pub last_page: u32,
// //     pub upload_path: String,
// // }
// //
// // impl SupportTableModel for CategoryListPage {
// //     fn get_page_title(&self) -> &'static str {
// //         "جميع التخصصات"
// //     }
// //
// //     fn get_table(&self) -> TableModel<'_> {
// //         let offset = (self.page.saturating_sub(1) * self.per_page) as usize;
// //
// //         let rows = self
// //             .categories
// //             .iter()
// //             .enumerate()
// //             .map(|(i, c)| {
// //                 vec![
// //                     Cell::Text(Cow::Owned((offset + i + 1).to_string())),
// //                     Cell::Text(Cow::Borrowed(&c.name)),
// //                     Cell::Text(Cow::Borrowed(&c.description)),
// //                     Cell::Image {
// //                         src: c
// //                             .image_path
// //                             .as_ref()
// //                             .map(|p| Cow::Owned(format!("{}/{}", self.upload_path, p))),
// //                     },
// //                     if c.active {
// //                         Cell::Badge { text: "نشط".into(), variant: Variant::Success }
// //                     } else {
// //                         Cell::Badge { text: "غير نشط".into(), variant: Variant::Danger }
// //                     },
// //                     // sub-categories: hidden when the category already has subjects
// //                     if c.has_subjects {
// //                         Cell::Empty
// //                     } else {
// //                         Cell::Button {
// //                             href: Cow::Owned(format!("/subcategories/{}", c.id)),
// //                             label: "التفرعات".into(),
// //                             variant: Variant::Primary,
// //                         }
// //                     },
// //                     // subjects & lessons: text hint when it has children
// //                     if c.has_children {
// //                         Cell::Text("اذهب الى التفرعات".into())
// //                     } else {
// //                         Cell::Button {
// //                             href: Cow::Owned(format!("/categories/{}/subjects-and-lessons", c.id)),
// //                             label: "المواد والدروس".into(),
// //                             variant: Variant::Primary,
// //                         }
// //                     },
// //                     Cell::Button {
// //                         href: Cow::Owned(format!("/categories/{}/edit", c.id)),
// //                         label: "تعديل".into(),
// //                         variant: Variant::Primary,
// //                     },
// //                     Cell::Button {
// //                         href: Cow::Owned(format!("/categories/{}/delete", c.id)),
// //                         label: "حذف".into(),
// //                         variant: Variant::Danger,
// //                     },
// //                 ]
// //             })
// //             .collect();
// //
// //         let base_url = match self.parent_id {
// //             Some(id) => format!("/subcategories/{}", id),
// //             None => "/categories".to_string(),
// //         };
// //
// //         TableModel {
// //             title: Cow::Borrowed(self.get_page_title()),
// //             header_action: self.parent_id.map(|id| HeaderAction {
// //                 label: "انشاء قسم فرعي".into(),
// //                 href: Cow::Owned(format!("/categories/create?id={}", id)),
// //             }),
// //             columns: [
// //                 "التسلسل", "اسم التخصص", "وصف التخصص", "صورة التخصص", "الحالة",
// //                 "التفرعات", "المواد والدروس", "تعديل", "حذف",
// //             ]
// //                 .into_iter()
// //                 .map(Cow::Borrowed)
// //                 .collect(),
// //             rows,
// //             pagination: Some(Pagination {
// //                 current_page: self.page,
// //                 last_page: self.last_page,
// //                 base_url: Cow::Owned(base_url),
// //             }),
// //         }
// //     }
// // }