use std::env;
use std::fs;
use std::path::Path;

fn map_inner_type_to_editor(inner: &str, field_name: &str) -> &'static str {
    let s = inner.trim();
    if s.contains("String") {
        return "string";
    }
    if s.contains("bool") {
        return "bool";
    }
    if s.contains("f32") || s.contains("f64") {
        return "number";
    }
    if s.contains("i32") || s.contains("i64") || s.contains("u32") || s.contains("u64") {
        return "int";
    }
    if s.contains("[f32") {
        // If field is named waypoints assume waypoint editor
        if field_name == "waypoints" {
            return "waypoints";
        }
        return "array<number>";
    }
    if s.starts_with("Vec<") {
        if s.contains("String") {
            return "array<string>";
        }
        if s.contains("[f32") {
            if field_name == "waypoints" {
                return "waypoints";
            }
            return "array<number>";
        }
        return "array<number>";
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
        let mut attrs: Vec<(String, String)> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if !line.starts_with("pub") {
                continue;
            }
            if !line.contains("Option<") {
                continue;
            }

            // Extract field name and inner type crudely
            if let Some(colon_pos) = line.find(':') {
                let before = &line[..colon_pos];
                let parts: Vec<&str> = before.split_whitespace().collect();
                if parts.len() >= 2 {
                    let field_name = parts[1].trim().trim_end_matches(',').to_string();
                    if let Some(opt_start) = line.find("Option<") {
                        if let Some(gt_pos) = line[opt_start..].find('>') {
                            let inner = &line[opt_start + "Option<".len()..opt_start + gt_pos];
                            let editor_type = map_inner_type_to_editor(inner, &field_name).to_string();
                            attrs.push((field_name, editor_type));
                        }
                    }
                }
            }
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
