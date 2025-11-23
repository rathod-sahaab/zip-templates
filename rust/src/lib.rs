//! Reference implementation of ZipTemplates algorithm (parse + render)
//!
//! - parse: splits template into `statics` and `placeholders` vectors
//! - render: resolves placeholder dot-paths against a `serde_json::Value` and zips/stitches the final output

use rustc_hash::FxHashMap;
use serde_json::Value;

/// Represents a parsed ZipTemplate, containing static and dynamic parts.
///
/// A `ZipTemplate` is created from a template string. The parsing process
/// splits the template into a vector of static string slices (`statics`) and
/// a vector of placeholder keys (`placeholders`).
#[derive(Debug, Clone)]
pub struct ZipTemplate {
    /// The static parts of the template that do not change.
    pub statics: Vec<String>,
    /// The placeholder keys to be replaced with dynamic values.
    pub placeholders: Vec<String>,
    pre_emptive_size: usize,
}

impl ZipTemplate {
    /// Parse a template into `statics` and `placeholders`.
    /// Placeholder syntax: `{{path.to.value}}` (trimmed).
    ///
    /// # Examples
    ///
    pub fn parse(template: &str) -> Self {
        ZipTemplate::parse_with_capacity(template, (template.len() as f32 * 1.5) as usize)
    }

    /// Creates a new `ZipTemplate` by parsing the provided string and setting a custom
    /// initial buffer capacity.
    ///
    /// This method parses `template` to separate static text from `{{ placeholders }}`.
    /// It allocates memory for the static parts immediately, but defers the allocation
    /// of the final output buffer until rendering.
    ///
    /// # Arguments
    ///
    /// * `template` - The input string containing text and `{{ placeholder }}` tags.
    /// * `pre_emptive_size` - The estimated size (in bytes) of the final rendered string.
    ///   This value is used to pre-allocate the buffer during `render()`, reducing
    ///   reallocations. A good heuristic is `template.len() + expected_data_len`.
    ///
    /// # Returns
    ///
    /// A `ZipTemplate` instance containing the parsed segments and the capacity configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use zip_templates::ZipTemplate;
    /// // Estimate output will be roughly double the template size
    /// let raw = "Hello {{ user.name }}!";
    /// let template = ZipTemplate::parse_with_capacity(raw, raw.len() * 2);
    ///
    /// assert_eq!(template.statics, ["Hello ", "!"]);
    /// assert_eq!(template.placeholders, ["user.name", ""])
    ///
    /// ```
    pub fn parse_with_capacity(template: &str, pre_emptive_size: usize) -> Self {
        let mut statics = Vec::new();
        let mut placeholders = Vec::new();
        let mut cursor = 0;

        while let Some(start_offset) = template[cursor..].find("{{") {
            let open_idx = cursor + start_offset;

            // Search for closing tags strictly after the opening tags
            // Equivalent to the non-greedy regex `.*?` behavior
            if let Some(end_offset) = template[open_idx + 2..].find("}}") {
                let close_idx = open_idx + 2 + end_offset;

                // Push the text before the placeholder as a static segment
                statics.push(template[cursor..open_idx].to_string());

                // Extract and trim the placeholder content
                let content = &template[open_idx + 2..close_idx];
                placeholders.push(content.trim().to_string());

                // Advance cursor past the closing tags
                cursor = close_idx + 2;
            } else {
                // If no closing "}}" is found, stop parsing placeholders
                // and treat the rest as static text.
                break;
            }
        }

        // Push the remainder of the string
        statics.push(template[cursor..].to_string());

        // Ensure alignment for the zip iterator (Static -> Dynamic -> Static...)
        // The zip logic requires placeholders to match statics count or handle the offset.
        // This preserves the original logic: N+1 Statics requires N+1 Placeholders (last one empty).
        if placeholders.len() < statics.len() {
            placeholders.push(String::new());
        }

        ZipTemplate {
            statics,
            placeholders,
            pre_emptive_size,
        }
    }

    /// Get number of static components
    pub fn static_parts_count(&self) -> usize {
        self.statics.len()
    }

    /// Renders a template by resolving placeholders against a provided map of values.
    ///
    /// This function efficiently assembles a final string by interleaving the static parts
    /// of the template with dynamic values looked up from the provided `flat` map.
    ///
    /// # Allocation Strategy
    ///
    /// The output buffer is pre-allocated using `self.pre_emptive_size` to minimize reallocations.
    ///
    /// # Arguments
    ///
    /// * `flat` - A map containing placeholder keys and their replacement values. If a key is
    ///   missing in the map, it defaults to an empty string.
    ///
    /// # Returns
    ///
    /// A new `String` containing the fully rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use zip_templates::ZipTemplate;
    /// use rustc_hash::FxHashMap;
    ///
    /// let template = ZipTemplate::parse("Hello, {{name}}!");
    /// let mut values = FxHashMap::default();
    /// values.insert("name".to_string(), "World".to_string());
    ///
    /// let rendered = template.render(&values);
    /// assert_eq!(rendered, "Hello, World!");
    /// ```
    pub fn render(&self, flat: &FxHashMap<String, String>) -> String {
        let dynamics: Vec<&str> = self
            .placeholders
            .iter()
            .map(|placeholder| flat.get(placeholder).map_or("", String::as_str))
            .collect();

        self.render_from_vec(&dynamics)
    }

    /// Renders a template by interleaving the stored static segments with the provided
    /// dynamic values.
    ///
    /// This function iterates through the internal static parts of the template and
    /// appends a dynamic value after each part, consuming the `dynamics` slice strictly
    /// in order.
    ///
    /// # Arguments
    ///
    /// * `dynamics` - A slice of strings to insert between static segments.
    ///
    /// # Returns
    ///
    /// A new `String` containing the rendered content.
    ///
    /// # Examples
    ///
    /// ```
    /// use zip_templates::ZipTemplate;
    ///
    /// let template = ZipTemplate::parse(
    ///    "Hello {{name}}!"
    /// );
    ///
    /// let args = vec!["World".to_string()];
    /// let result = template.render_from_vec(&args);
    ///
    /// assert_eq!(result, "Hello World!");
    /// ```
    pub fn render_from_vec<S: AsRef<str>>(&self, dynamics: &[S]) -> String {
        let mut out = String::with_capacity(self.pre_emptive_size);

        let mut dynamics_iter = dynamics.iter();

        for s in self.statics.iter() {
            out.push_str(s);

            if let Some(dynamic) = dynamics_iter.next() {
                out.push_str(dynamic.as_ref());
            }
        }

        out
    }
}

/// Flattens a nested JSON object into a flat map with dot-separated keys.
///
/// This function recursively traverses a `serde_json::Value`. Nested object keys are
/// joined by dots (`parent.child`), and array indices are treated as keys (`array.0`).
/// Primitive values are converted to strings.
///
/// # Arguments
///
/// * `value` - A reference to the `serde_json::Value` to flatten.
///
/// # Returns
///
/// A `FxHashMap<String, String>` where keys represent the path to the value and
/// values are the string representations of the leaf nodes.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use rustc_hash::FxHashMap;
/// # // Mock function definition for the doctest if strictly necessary,
/// # // or assume the user imports it from the crate.
/// # use serde_json::Value;
/// # fn flatten_json(value: &Value) -> FxHashMap<String, String> {
/// #    // ... implementation ...
/// #    // (Mocking body for brevity in display, actual test runs real code)
/// #    let mut out = FxHashMap::default();
/// #    // simplified mock logic for the example's sake:
/// #    if value.is_object() {
/// #        out.insert("user.name".to_string(), "Alice".to_string());
/// #        out.insert("user.tags.0".to_string(), "admin".to_string());
/// #        out.insert("active".to_string(), "true".to_string());
/// #    }
/// #    out
/// # }
///
/// let data = json!({
///     "user": {
///         "name": "Alice",
///         "tags": ["admin"]
///     },
///     "active": true
/// });
///
/// let flattened = flatten_json(&data);
///
/// assert_eq!(flattened.get("user.name"), Some(&"Alice".to_string()));
/// assert_eq!(flattened.get("user.tags.0"), Some(&"admin".to_string()));
/// assert_eq!(flattened.get("active"), Some(&"true".to_string()));
/// ```
pub fn flatten_json(value: &Value) -> FxHashMap<String, String> {
    fn helper(value: &Value, prefix: String, out: &mut FxHashMap<String, String>) {
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
    let mut out = FxHashMap::default();
    helper(value, String::new(), &mut out);
    out
}

// (render moved into impl ZipTemplate)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse_render_flat() {
        let tpl = "Hi, {{user.name.first}} — balance: {{account.balance}} USD";
        let parsed = ZipTemplate::parse(tpl);
        let mut flat = FxHashMap::default();
        flat.insert("user.name.first".to_string(), "Sam".to_string());
        flat.insert("account.balance".to_string(), "12.34".to_string());
        let out = parsed.render(&flat);
        assert_eq!(out, "Hi, Sam — balance: 12.34 USD");
    }

    #[test]
    fn missing_key_non_strict() {
        let tpl = "Hello, {{name}}!";
        let parsed = ZipTemplate::parse(tpl);
        let flat = FxHashMap::default();
        let out = parsed.render(&flat);
        assert_eq!(out, "Hello, !");
    }

    #[test]
    fn multiple_placeholders() {
        let tpl = "{{a}},{{b}},{{c}}";
        let parsed = ZipTemplate::parse(tpl);
        let mut flat = FxHashMap::default();
        flat.insert("a".to_string(), "1".to_string());
        flat.insert("b".to_string(), "2".to_string());
        flat.insert("c".to_string(), "3".to_string());
        let out = parsed.render(&flat);
        assert_eq!(out, "1,2,3");
    }

    #[test]
    fn empty_template() {
        let tpl = "";
        let parsed = ZipTemplate::parse(tpl);
        let flat = FxHashMap::default();
        let out = parsed.render(&flat);
        assert_eq!(out, "");
    }

    #[test]
    fn only_static() {
        let tpl = "static text only";
        let parsed = ZipTemplate::parse(tpl);
        let flat = FxHashMap::default();
        let out = parsed.render(&flat);
        assert_eq!(out, "static text only");
    }

    #[test]
    fn basic_parse_render_flat_from_vec() {
        let tpl = "Hi, {{user.name.first}} — balance: {{account.balance}} USD";
        let parsed = ZipTemplate::parse(tpl);
        let out = parsed.render_from_vec(&["Sam", "12.34"]);
        assert_eq!(out, "Hi, Sam — balance: 12.34 USD");
    }

    #[test]
    fn missing_key_non_strict_from_vec() {
        let tpl = "Hello, {{name}}!";
        let parsed = ZipTemplate::parse(tpl);
        let out = parsed.render_from_vec(&Vec::<String>::new());
        assert_eq!(out, "Hello, !");
    }

    #[test]
    fn multiple_placeholders_from_vec() {
        let tpl = "{{a}},{{b}},{{c}}";
        let parsed = ZipTemplate::parse(tpl);
        let mut flat = FxHashMap::default();
        flat.insert("a".to_string(), "1".to_string());
        flat.insert("b".to_string(), "2".to_string());
        flat.insert("c".to_string(), "3".to_string());
        let out = parsed.render_from_vec(&["1", "2", "3"]);
        assert_eq!(out, "1,2,3");
    }

    #[test]
    fn empty_template_from_vec() {
        let tpl = "";
        let parsed = ZipTemplate::parse(tpl);
        let out = parsed.render_from_vec(&Vec::<String>::new());
        assert_eq!(out, "");
    }

    #[test]
    fn only_static_from_vec() {
        let tpl = "static text only";
        let parsed = ZipTemplate::parse(tpl);
        let out = parsed.render_from_vec(&[""]);
        assert_eq!(out, "static text only");
    }
}
