use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ColliderConfig {
    // Previously had typed fields (offset, is_trigger, rectangle_half_extents,
    // circle_radius) but those were not used by runtime code. Keep a
    // catch-all map so unknown keys in JSON are accepted without
    // generating dead-code warnings for unused named fields.
    #[serde(flatten)]
    pub _extra: HashMap<String, Value>,
}

