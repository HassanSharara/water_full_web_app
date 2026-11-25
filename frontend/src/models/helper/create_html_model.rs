use std::borrow::Cow;
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum InputType<'a> {
    Text {
        value: Cow<'a, str>,
        placeholder: Option<Cow<'a, str>>,
    },
    Email {
        value: Cow<'a, str>,
        placeholder: Option<Cow<'a, str>>,
    },
    Password {
        value: Cow<'a, str>,
        placeholder: Option<Cow<'a, str>>,
    },
    Hidden {
        value: Cow<'a, str>,
    },
    Date {
        value: Option<Cow<'a, str>>,
    },
    Number {
        value: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    TextArea {
        value: Cow<'a, str>,
        rows: Option<u32>,
        placeholder: Option<Cow<'a, str>>,
    },
    File {
        accept: Option<Cow<'a, str>>,
        multiple: bool,
    },
    Checkbox {
        checked: bool,
    },
    Radio {
        selected_id: Option<Cow<'a, str>>,
        options: Vec<SelectOption<'a>>,
    },
    Select {
        selected_ids: Vec<Cow<'a, str>>,
        options: Vec<SelectOption<'a>>,
        multiple: bool,
    },
}

#[derive(Debug, Clone)]
pub struct SelectOption<'a> {
    pub id: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> SelectOption<'a> {
    pub fn is_selected(&self, current_id: &Option<Cow<'a, str>>) -> bool {
        match current_id {
            Some(id) => self.id == *id,
            None => false,
        }
    }

    pub fn is_selected_multi(&self, current_ids: &[Cow<'a, str>]) -> bool {
        current_ids.iter().any(|id| id == &self.id)
    }
}

#[derive(Debug, Clone)]
pub struct SingleInputModel<'a> {
    pub input_type: InputType<'a>,
    pub title: Cow<'a, str>,
    pub name: Cow<'a, str>,
    pub required: bool,
    pub disabled: bool,
}

/// Writes `s`, HTML-escaped, directly into `out`.
/// No intermediate String is allocated — this is the key difference
/// from the previous `esc(s) -> String` version, which allocated
/// once per call and then had to be copied into `out` anyway.
#[inline]
fn esc_into(out: &mut String, s: &str) {
    // Fast path: if there's nothing to escape, push the whole
    // slice in one go instead of char-by-char.
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

impl<'a> SingleInputModel<'a> {
    /// A rough size estimate so the caller can pre-reserve capacity
    /// and avoid buffer reallocations while writing.
    fn estimated_size(&self) -> usize {
        let base = self.title.len() + self.name.len() + 128;
        match &self.input_type {
            InputType::Select { options, .. } | InputType::Radio { options, .. } => {
                base + options.iter().map(|o| o.id.len() + o.value.len() + 96).sum::<usize>()
            }
            _ => base,
        }
    }

    /// Renders this single input's full markup (label + control)
    /// as one HTML string.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.estimated_size());
        self.render_into(&mut out);
        out
    }

    /// Writes this input's markup directly into an existing buffer.
    /// This is the version actually used internally so multiple
    /// inputs share one growing buffer instead of each allocating
    /// their own String that then gets copied into a parent buffer.
    pub fn render_into(&self, out: &mut String) {
        let required_attr = if self.required { " required" } else { "" };
        let disabled_attr = if self.disabled { " disabled" } else { "" };
        let self_labeled = matches!(
            self.input_type,
            InputType::Checkbox { .. } | InputType::Radio { .. }
        );

        out.push_str(r#"<div class="form-group mb-4">"#);

        if !self_labeled {
            out.push_str(r#"<label class="form-label font-weight-bold">"#);
            esc_into(out, &self.title);
            if self.required {
                out.push_str(r#"<span class="text-danger">*</span>"#);
            }
            out.push_str("</label>");
        }

        match &self.input_type {
            InputType::Text { value, placeholder } => {
                out.push_str(r#"<input type="text" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                esc_into(out, value);
                out.push_str(r#"" placeholder=""#);
                esc_into(out, placeholder.as_deref().unwrap_or(""));
                out.push('"');
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::Email { value, placeholder } => {
                out.push_str(r#"<input type="email" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                esc_into(out, value);
                out.push_str(r#"" placeholder=""#);
                esc_into(out, placeholder.as_deref().unwrap_or(""));
                out.push('"');
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::Password { value, placeholder } => {
                out.push_str(r#"<input type="password" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                esc_into(out, value);
                out.push_str(r#"" placeholder=""#);
                esc_into(out, placeholder.as_deref().unwrap_or(""));
                out.push('"');
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::Hidden { value } => {
                out.push_str(r#"<input type="hidden" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                esc_into(out, value);
                out.push_str(r#"">"#);
            }
            InputType::Date { value } => {
                out.push_str(r#"<input type="date" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                esc_into(out, value.as_deref().unwrap_or(""));
                out.push('"');
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::Number { value, min, max, step } => {
                out.push_str(r#"<input type="number" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" value=""#);
                let _ = write!(out, "{}", value.unwrap_or(0.0));
                out.push('"');
                if let Some(v) = min {
                    let _ = write!(out, r#" min="{}""#, v);
                }
                if let Some(v) = max {
                    let _ = write!(out, r#" max="{}""#, v);
                }
                if let Some(v) = step {
                    let _ = write!(out, r#" step="{}""#, v);
                }
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::TextArea { value, placeholder, .. } => {
                out.push_str(r#"<textarea name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" class="form-control" placeholder=""#);
                esc_into(out, placeholder.as_deref().unwrap_or(""));
                out.push('"');
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push('>');
                esc_into(out, value);
                out.push_str("</textarea>");
            }
            InputType::File { accept, multiple } => {
                out.push_str(r#"<input type="file" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" accept=""#);
                esc_into(out, accept.as_deref().unwrap_or(""));
                out.push('"');
                if *multiple {
                    out.push_str(" multiple");
                }
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push_str(r#" class="form-control">"#);
            }
            InputType::Checkbox { checked } => {
                out.push_str(r#"<div class="custom-control custom-checkbox"><input type="checkbox" name=""#);
                esc_into(out, &self.name);
                out.push_str(r#"" id="chk_"#);
                esc_into(out, &self.name);
                out.push_str(r#"" class="custom-control-input""#);
                if *checked {
                    out.push_str(" checked");
                }
                out.push_str(disabled_attr);
                out.push_str(r#"><label class="custom-control-label" for="chk_"#);
                esc_into(out, &self.name);
                out.push_str(r#"">"#);
                esc_into(out, &self.title);
                out.push_str("</label></div>");
            }
            InputType::Select { selected_ids, options, multiple } => {
                out.push_str(r#"<select name=""#);
                esc_into(out, &self.name);
                out.push('"');
                out.push_str(r#" class="form-control""#);
                if *multiple {
                    out.push_str(" multiple");
                }
                out.push_str(required_attr);
                out.push_str(disabled_attr);
                out.push('>');
                for o in options {
                    out.push_str(r#"<option value=""#);
                    esc_into(out, &o.id);
                    out.push('"');
                    if o.is_selected_multi(selected_ids) {
                        out.push_str(" selected");
                    }
                    out.push('>');
                    esc_into(out, &o.value);
                    out.push_str("</option>");
                }
                out.push_str("</select>");
            }
            InputType::Radio { selected_id, options } => {
                out.push_str(r#"<div><span class="d-block mb-2 font-weight-bold">"#);
                esc_into(out, &self.title);
                out.push_str("</span>");
                for (i, o) in options.iter().enumerate() {
                    out.push_str(r#"<div class="custom-control custom-radio"><input type="radio" id="rad_"#);
                    esc_into(out, &self.name);
                    out.push('_');
                    let _ = write!(out, "{}", i);
                    out.push_str(r#"" name=""#);
                    esc_into(out, &self.name);
                    out.push_str(r#"" value=""#);
                    esc_into(out, &o.id);
                    out.push('"');
                    if o.is_selected(selected_id) {
                        out.push_str(" checked");
                    }
                    out.push_str(disabled_attr);
                    out.push_str(r#" class="custom-control-input"><label class="custom-control-label" for="rad_"#);
                    esc_into(out, &self.name);
                    out.push('_');
                    let _ = write!(out, "{}", i);
                    out.push_str(r#"">"#);
                    esc_into(out, &o.value);
                    out.push_str("</label></div>");
                }
                out.push_str("</div>");
            }
        }

        out.push_str("</div>");
    }
}

pub trait SupportCreateHtmlModel {
    fn get_page_title(&self) -> &'static str;
    fn get_action_route(&self) -> &str;
    fn get_inputs(&self) -> Vec<SingleInputModel<'_>>;
    
    fn get_csrf_token(&self) -> Option<&str>;

    /// All inputs pre-rendered and concatenated into one HTML string.
    /// Every input writes directly into the same buffer — one
    /// allocation (with a capacity estimate), no per-input Strings
    /// that then need to be copied and dropped.
    fn get_inputs_html(&self) -> String {
        let inputs = self.get_inputs();
        let capacity: usize = inputs.iter().map(|m| m.estimated_size()).sum();
        let mut out = String::with_capacity(capacity);
        for m in &inputs {
            m.render_into(&mut out);
        }
        out
    }
}