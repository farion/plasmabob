use serde_json::Value;

pub(crate) fn inner_array_value_to_csv_string(v: &Value) -> String {
    if let Value::Array(arr) = v {
        let parts: Vec<String> = arr
            .iter()
            .map(|item| match item {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            })
            .collect();
        parts.join(",")
    } else {
        String::new()
    }
}

pub(crate) fn format_array_short(arr: &[Value]) -> String {
    let parts: Vec<String> = arr
        .iter()
        .map(|v| match v {
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    format!("{}", f)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => format!("\"{}\"", s),
            other => serde_json::to_string(other).unwrap_or_default(),
        })
        .collect();
    format!("[{}]", parts.join(","))
}
