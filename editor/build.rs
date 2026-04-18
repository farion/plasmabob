use std::env;
use std::fs;
use std::path::Path;

fn map_inner_type_to_editor(inner: &str, field_name: &str) -> &'static str {
    let s = inner.trim();
    // Prefer array/vec patterns before scalar float detection because e.g.
    // "[f32; 2]" contains "f32" and would otherwise be classified as
    // a scalar number.
    if s.contains("[f32") || s.contains("[f64") {
        if field_name == "waypoints" {
            return "waypoints";
        }
        return "array<number>";
    }
    if s.contains("Vec<") {
        if s.contains("String") {
            return "array<string>";
        }
        if s.contains("[f32") || s.contains("[f64") {
            if field_name == "waypoints" {
                return "waypoints";
            }
            return "array<number>";
        }
        return "array<number>";
    }
    if s.contains("String") {
        return "string";
    }
    if s.contains("bool") {
        return "bool";
    }
    if s.contains("f32") || s.contains("f64") {
        return "number";
    }
    if s.contains("i8") || s.contains("i16") || s.contains("i32") || s.contains("i64")
        || s.contains("u8") || s.contains("u16") || s.contains("u32") || s.contains("u64")
    {
        return "int";
    }

    "json"
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let configs_dir = Path::new("../src/game/level/configs");
    let mut components: Vec<(String, Vec<(String, String)>)> = Vec::new();

    for entry in fs::read_dir(configs_dir).expect("read configs dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("stem")
            .to_string();

        // component name: remove trailing _config if present
        let comp_name = if let Some(stripped) = stem.strip_suffix("_config") {
            stripped.to_string()
        } else {
            stem.clone()
        };

        let content = fs::read_to_string(&path).expect("read file");

        // We'll use both the original content (to find serde renames located
        // in attributes) and a simplified content with attribute lines
        // removed so field: type parsing is easier.
        let mut content_no_attrs = content.clone();
        while let Some(start) = content_no_attrs.find("#[") {
            if let Some(end_rel) = content_no_attrs[start..].find(']') {
                let end = start + end_rel + 1;
                content_no_attrs.replace_range(start..end, "");
            } else {
                break;
            }
        }

        let orig_lines: Vec<&str> = content.lines().collect();
        let no_attr_lines: Vec<&str> = content_no_attrs.lines().collect();

        let mut attrs: Vec<(String, String)> = Vec::new();

        for (i, raw_line) in no_attr_lines.iter().enumerate() {
            let line = raw_line.trim();
            if !line.starts_with("pub") {
                continue;
            }
            // Must contain a colon to be a field declaration
            let colon_pos = if let Some(p) = line.find(':') { p } else { continue };

            // Determine the declared field name: the last token before ':' is
            // typically the field name (handles "pub", "pub(crate)", etc).
            let before = &line[..colon_pos];
            let tokens: Vec<&str> = before.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            let field_name = tokens.last().unwrap().trim().trim_end_matches(',').to_string();

            // Extract the type text to the end of the field (stop at ',' or end)
            let mut type_part = line[colon_pos + 1..].trim().to_string();
            // remove trailing commas
            if type_part.ends_with(',') {
                type_part.pop();
            }

            // If the type is wrapped with Option<...>, extract inner, otherwise use type_part
            let inner = if type_part.starts_with("Option<") {
                // find matching '>' for the first '<'
                let mut depth = 0usize;
                let mut end_idx = None;
                for (idx, ch) in type_part.chars().enumerate() {
                    if ch == '<' { depth += 1 } else if ch == '>' {
                        depth -= 1;
                        if depth == 0 {
                            end_idx = Some(idx);
                            break;
                        }
                    }
                }
                if let Some(e) = end_idx {
                    type_part["Option<".len()..e].trim().to_string()
                } else {
                    // fallback
                    type_part.clone()
                }
            } else {
                type_part.clone()
            };

            // Look for a serde(rename = "...") attribute in the original
            // source within a few lines above this field.
            let mut json_name = field_name.clone();
            let start_scan = if i >= 4 { i - 4 } else { 0 };
            for j in (start_scan..=i).rev() {
                if let Some(orig) = orig_lines.get(j) {
                    let s = orig.trim();
                    if s.contains("serde") && s.contains("rename") {
                        if let Some(q1) = s.find('"') {
                            if let Some(q2) = s[q1 + 1..].find('"') {
                                let extracted = &s[q1 + 1..q1 + 1 + q2];
                                json_name = extracted.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            let editor_type = map_inner_type_to_editor(&inner, &field_name).to_string();
            attrs.push((json_name, editor_type));
        }

        components.push((comp_name, attrs));
    }

    // Write generated file
    let mut out = String::new();
    out.push_str("// Generated file — do not edit. Produced by build.rs\n");
    out.push_str("pub fn component_attribute_type(component: &str, attribute: &str) -> Option<&'static str> {\n");
    out.push_str("    match component {\n");
    for (comp, attrs) in &components {
        out.push_str(&format!("        \"{}\" => match attribute {{\n", comp));
        for (name, typ) in attrs {
            out.push_str(&format!("            \"{}\" => Some(\"{}\"),\n", name, typ));
        }
        out.push_str("            _ => None,\n        },\n");
    }
    out.push_str("        _ => None,\n    }\n}\n\n");

    // component_declared_attributes
    out.push_str("pub fn component_declared_attributes(component: &str) -> &'static [(&'static str, &'static str)] {\n");
    out.push_str("    match component {\n");
    for (comp, attrs) in &components {
        out.push_str(&format!("        \"{}\" => &[\n", comp));
        for (name, typ) in attrs {
            out.push_str(&format!("            (\"{}\", \"{}\"),\n", name, typ));
        }
        out.push_str("        ],\n");
    }
    out.push_str("        _ => &[],\n    }\n}\n");

    let out_path = Path::new(&out_dir).join("component_attr_map.rs");
    fs::write(&out_path, out).expect("write generated");
}
