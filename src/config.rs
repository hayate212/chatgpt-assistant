use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gpt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    profiles: HashMap<String, Profile>,
}
impl Config {
    pub fn get_profile(&self, name: String) -> Option<&Profile> {
        self.profiles.get(name.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub messages: Vec<gpt::ChatMessage>,
}

impl Profile {
    pub fn new(messages: Vec<gpt::ChatMessage>) -> Profile {
        Profile { messages }
    }
    pub fn get_messages(&self) -> &Vec<gpt::ChatMessage> {
        &self.messages
    }
}

pub const DEFAULT_CONFIG_STR: &str = r#"profiles:
  default:
    messages: []
"#;
