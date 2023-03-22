use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    role: String,
    content: String,
}
impl ChatMessage {
    pub fn new(content: String, role: String) -> Self {
        ChatMessage { content, role }
    }
    pub fn get_content(&self) -> &str {
        &self.content
    }
    pub fn get_role(&self) -> &str {
        &self.role
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    id: String,
    object: String,
    created: i32,
    choices: Vec<Choice>,
}
impl ChatResponse {
    pub fn get_choices(&self) -> &Vec<Choice> {
        &self.choices
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choice {
    index: i32,
    message: ChatMessage,
    finish_reason: String,
}

impl Choice {
    pub fn get_message(&self) -> &ChatMessage {
        &self.message
    }
}

pub const CHAT_GPTMODEL: &str = "gpt-3.5-turbo";

pub struct GptClient {
    api_key: String,
    model: String,
    client: Client,
}
impl GptClient {
    pub fn new(api_key: String) -> GptClient {
        GptClient {
            api_key,
            model: CHAT_GPTMODEL.to_string(),
            client: Client::new(),
        }
    }

    pub async fn chat_completions(
        &self,
        messages: &Vec<ChatMessage>,
    ) -> Result<ChatResponse, Box<dyn std::error::Error>> {
        let body = ChatRequest {
            model: self.model.to_string(),
            messages: messages.clone(),
        };

        let res = &self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;
        Ok(res.clone())
    }
}
