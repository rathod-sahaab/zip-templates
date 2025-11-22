//! Reference implementation of ZipTemplates algorithm (parse + render)
//!
//! - parse: splits template into `statics` and `placeholders` vectors
//! - render: resolves placeholder dot-paths against a `serde_json::Value` and zips/stitches the final output

use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ZipTemplate {
    pub statics: Vec<String>,
    pub placeholders: Vec<String>,
    pre_emptive_size: usize,
}

impl ZipTemplate {
    /// Parse a template into `statics` and `placeholders`.
    /// Placeholder syntax: `{{path.to.value}}` (trimmed).
    pub fn from(template: &str) -> Self {
        let re = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        let mut statics = Vec::new();
        let mut placeholders = Vec::new();
        let mut last = 0;
        for caps in re.captures_iter(template) {
            let m = caps.get(0).unwrap();
            statics.push(template[last..m.start()].to_string());
            placeholders.push(caps[1].trim().to_string());
            last = m.end();
        }
        statics.push(template[last..].to_string());

        if placeholders.len() < statics.len() {
            placeholders.push("".to_string());
        }

        ZipTemplate {
            statics,
            placeholders,
            pre_emptive_size: (template.len() as f32 * 1.5) as usize,
        }
    }

    /// Render this parsed template against a flat map of placeholder values.
    pub fn render(&self, flat: &std::collections::HashMap<String, String>) -> String {
        let mut out = String::with_capacity(self.pre_emptive_size);

        self.statics
            .iter()
            .zip(&self.placeholders)
            .for_each(|(s, placeholder)| {
                out.push_str(s);
                if let Some(data) = flat.get(placeholder) {
                    out.push_str(data);
                }
            });

        out
    }
}

use std::collections::HashMap;

/// Flattens a nested JSON object into a flat map with dot-separated keys.
pub fn flatten_json(value: &Value) -> HashMap<String, String> {
    fn helper(value: &Value, prefix: String, out: &mut HashMap<String, String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    let new_prefix = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };
                    helper(v, new_prefix, out);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_prefix = if prefix.is_empty() {
                        i.to_string()
                    } else {
                        format!("{}.{}", prefix, i)
                    };
                    helper(v, new_prefix, out);
                }
            }
            Value::Null => {
                out.insert(prefix, String::new());
            }
            _ => {
                out.insert(prefix, value.to_string().trim_matches('"').to_string());
            }
        }
    }
    let mut out = HashMap::new();
    helper(value, String::new(), &mut out);
    out
}

// (render moved into impl ZipTemplate)

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn basic_parse_render_flat() {
        let tpl = "Hi, {{user.name.first}} — balance: {{account.balance}} USD";
        let parsed = ZipTemplate::from(tpl);
        let mut flat = HashMap::new();
        flat.insert("user.name.first".to_string(), "Sam".to_string());
        flat.insert("account.balance".to_string(), "12.34".to_string());
        let out = parsed.render(&flat);
        assert_eq!(out, "Hi, Sam — balance: 12.34 USD");
    }

    #[test]
    fn missing_key_non_strict() {
        let tpl = "Hello, {{name}}!";
        let parsed = ZipTemplate::from(tpl);
        let flat = HashMap::new();
        let out = parsed.render(&flat);
        assert_eq!(out, "Hello, !");
    }

    #[test]
    fn multiple_placeholders() {
        let tpl = "{{a}},{{b}},{{c}}";
        let parsed = ZipTemplate::from(tpl);
        let mut flat = HashMap::new();
        flat.insert("a".to_string(), "1".to_string());
        flat.insert("b".to_string(), "2".to_string());
        flat.insert("c".to_string(), "3".to_string());
        let out = parsed.render(&flat);
        assert_eq!(out, "1,2,3");
    }

    #[test]
    fn empty_template() {
        let tpl = "";
        let parsed = ZipTemplate::from(tpl);
        let flat = HashMap::new();
        let out = parsed.render(&flat);
        assert_eq!(out, "");
    }

    #[test]
    fn only_static() {
        let tpl = "static text only";
        let parsed = ZipTemplate::from(tpl);
        let flat = HashMap::new();
        let out = parsed.render(&flat);
        assert_eq!(out, "static text only");
    }
}
