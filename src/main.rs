use std::io::Write;

use dotenv;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use termion::{color, style};

#[derive(Debug, Serialize, Deserialize)]
struct MessagesConfigFile {
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatResponse {
    id: String,
    object: String,
    created: i32,
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Choice {
    index: i32,
    message: ChatMessage,
    finish_reason: String,
}

const CHAT_GPTMODEL: &str = "gpt-3.5-turbo";

const BLUE: color::Fg<color::Blue> = color::Fg(color::Blue);
const RESET: style::Reset = style::Reset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::home_dir().unwrap().join(".chatgpt-assistant/");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }
    let config_file = config_dir.join(".config");
    if !config_file.exists() {
        let mut file = std::fs::File::create(&config_file).unwrap();
        let secret_key = read_line("SECRET KEY: ");
        let org_id = read_line("Organization ID: ");
        file.write_all(format!("API_KEY={}\nAPI_ORG={}", secret_key, org_id).as_bytes())
            .unwrap();
    }
    dotenv::from_path(&config_file).unwrap();

    let messages_file_path = config_dir.join("messages.json");
    if !messages_file_path.exists() {
        let mut file = std::fs::File::create(&messages_file_path).unwrap();
        let config = MessagesConfigFile {
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "これは対話をするためのルールです。文章のはじめには必ず 🤖 の絵文字をつけて返答してください。".to_string(),
                },  
                ChatMessage {
                    role: "system".to_string(),
                    content: "また、絵文字のあとにスペースを1文字分入れましょう".to_string(),
                }
            ],
        };
        file.write_all(serde_json::to_string(&config).unwrap().as_bytes())
            .unwrap()
    }
    let mut messages = serde_json::from_str::<MessagesConfigFile>(
        std::fs::read_to_string(messages_file_path)
            .unwrap()
            .as_str(),
    )
    .unwrap()
    .messages;

    let client = Client::new();
    let api_key = dotenv::var("API_KEY").unwrap();

    println!("ChatGPT との対話を開始します");
    println!("終了する際は \'quit\' と入力してください");
    loop {
        let line = read_line(">> ");
        if line == "quit" {
            break;
        }
        messages.push(ChatMessage {
            role: String::from("user"),
            content: line.to_string(),
        });
        let body = ChatRequest {
            model: CHAT_GPTMODEL.to_string(),
            messages: messages.clone(),
        };

        let res = client
            .post("https://api.openai.com/v1/chat/completions")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;
        let choice = res.choices.first().unwrap();

        println!("{}{}{}", BLUE, choice.message.content, RESET);

        messages.push(choice.message.clone());
    }

    Ok(())
}

fn read_line(inline_message: &str) -> String {
    print!("{}", inline_message);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    // remove whitespace
    line.trim().to_string()
}
