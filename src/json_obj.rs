use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HelpResponse {
    pub events: HashMap<String, String>,
    pub functions: HashMap<String, String>,
    pub types: HashMap<String, String>
}