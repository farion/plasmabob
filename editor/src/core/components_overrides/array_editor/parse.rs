pub(crate) fn parse_array_type_signature(type_str: &str) -> super::ParsedArrayType {
    let s = type_str.replace(' ', "").replace('\t', "").to_string();
    if let Some(start) = s.find('[') {
        if let Some(semi) = s.find(';') {
            if let Some(end) = s.find(']') {
                let after_semi = &s[semi + 1..end];
                if let Ok(n) = after_semi.parse::<usize>() {
                    let elem = &s[start + 1..semi];
                    let element_is_array = elem.starts_with('[') || elem.starts_with("Vec<");
                    let element_is_number = elem.contains("f32")
                        || elem.contains("f64")
                        || elem.contains("number")
                        || elem.contains("i32")
                        || elem.contains("i64");
                    let mut inner_fixed = None;
                    if elem.starts_with('[') {
                        if let Some(semi2) = elem.find(';') {
                            if let Some(end2) = elem.rfind(']') {
                                if let Ok(inner_n) = elem[semi2 + 1..end2].parse::<usize>() {
                                    inner_fixed = Some(inner_n);
                                }
                            }
                        }
                    }
                    return super::ParsedArrayType {
                        element_is_array,
                        element_is_number,
                        inner_fixed_len: inner_fixed,
                        outer_fixed_len: Some(n),
                    };
                }
            }
        }
    }

    if s.starts_with("Vec<") || s.starts_with("vec<") {
        if let Some(open) = s.find('<') {
            if let Some(close) = s.rfind('>') {
                let inner = &s[open + 1..close];
                if inner.starts_with('[') {
                    if let Some(semi) = inner.find(';') {
                        if let Some(end) = inner.find(']') {
                            if let Ok(n) = inner[semi + 1..end].parse::<usize>() {
                                let element_is_number = inner.contains("f32")
                                    || inner.contains("f64")
                                    || inner.contains("number")
                                    || inner.contains("i32")
                                    || inner.contains("i64");
                                return super::ParsedArrayType {
                                    element_is_array: true,
                                    element_is_number,
                                    inner_fixed_len: Some(n),
                                    outer_fixed_len: None,
                                };
                            }
                        }
                    }
                } else {
                    let element_is_number = inner.contains("f32")
                        || inner.contains("f64")
                        || inner.contains("number")
                        || inner.contains("i32")
                        || inner.contains("i64");
                    return super::ParsedArrayType {
                        element_is_array: false,
                        element_is_number,
                        inner_fixed_len: None,
                        outer_fixed_len: None,
                    };
                }
            }
        }
    }

    if s.starts_with("array<") {
        if let Some(open) = s.find('<') {
            if let Some(close) = s.rfind('>') {
                let inner = &s[open + 1..close];
                if let Some(semi) = inner.find(';') {
                    if let Ok(n) = inner[semi + 1..].parse::<usize>() {
                        let element_is_number = inner.contains("number") || inner.contains("f32");
                        return super::ParsedArrayType {
                            element_is_array: false,
                            element_is_number,
                            inner_fixed_len: None,
                            outer_fixed_len: Some(n),
                        };
                    }
                }
            }
        }
    }

    let element_is_number = s.contains("f32")
        || s.contains("f64")
        || s.contains("number")
        || s.contains("i32")
        || s.contains("i64");
    super::ParsedArrayType {
        element_is_array: false,
        element_is_number,
        inner_fixed_len: None,
        outer_fixed_len: None,
    }
}
