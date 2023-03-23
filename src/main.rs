mod config;
mod gpt;

use std::io::{self, Write};

use async_recursion::async_recursion;
use clap::Parser;
use dotenv;
use gpt::ChatMessage;
use termion::{
    color, cursor::DetectCursorPos, event::Key, input::TermRead, raw::IntoRawMode, style,
};

const RED: color::Fg<color::Red> = color::Fg(color::Red);
const BLUE: color::Fg<color::Blue> = color::Fg(color::Blue);
const GREEN: color::Fg<color::Green> = color::Fg(color::Green);
const RESET: style::Reset = style::Reset;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short = 'p', long)]
    profile: Option<String>,
    #[arg(short = 'o', long)]
    oneshot: bool,
    #[arg(short = 's', long)]
    system_messages: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let arg_profile = args.profile.unwrap_or("default".to_owned());
    let arg_oneshot = args.oneshot;
    let arg_system_messages = args.system_messages.unwrap_or(vec![]);

    let config_dir = dirs::home_dir().unwrap().join(".chatgpt-assistant/");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }
    let env_file = config_dir.join(".env");
    if !env_file.exists() {
        let mut file = std::fs::File::create(&env_file).unwrap();
        let secret_key = read_line("SECRET KEY: ");
        let org_id = read_line("Organization ID: ");
        file.write_all(format!("API_KEY={}\nAPI_ORG={}", secret_key, org_id).as_bytes())
            .unwrap();
    }
    dotenv::from_path(&env_file).unwrap();

    let config_file_path = config_dir.join("config.yaml");
    if !config_file_path.exists() {
        let mut file = std::fs::File::create(&config_file_path).unwrap();
        file.write_all(config::DEFAULT_CONFIG_STR.as_bytes())
            .unwrap()
    }
    let loaded_config = serde_yaml::from_str::<config::Config>(
        std::fs::read_to_string(config_file_path).unwrap().as_str(),
    )
    .unwrap();

    let profile = loaded_config.get_profile(arg_profile);
    if profile.is_none() {
        println!("profile not found");
        return Ok(());
    }

    let mut messages = profile.unwrap().get_messages().clone();
    for system_message in arg_system_messages {
        messages.push(gpt::ChatMessage::new(system_message, "system".to_owned()));
    }
    let gpt_client = gpt::GptClient::new(dotenv::var("API_KEY").unwrap());

    println!("Starting conversation with ChatGPT.");
    println!("Please type 'quit' to end the conversation.");
    if arg_oneshot {
        println!("{}{}{}", RED, "ONESHOT MODE", RESET)
    }

    talk(&gpt_client, messages, arg_oneshot).await?;

    Ok(())
}

#[async_recursion]
async fn talk(
    gpt_client: &gpt::GptClient,
    before_messages: Vec<gpt::ChatMessage>,
    oneshot: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut messages = before_messages.clone();
    loop {
        let message = match read_chatmessage() {
            None => return Ok(()),
            Some(c) => c,
        };
        println!();
        messages.push(message.clone());

        if message.get_role() == "user" {
            break;
        }
    }

    let message = match gpt_client.chat_completions(&messages).await {
        Err(_) => return Ok(()),
        Ok(res) => {
            let choice = res.get_choices().first().unwrap();
            let message = choice.get_message();
            message.clone()
        }
    };

    println!("{}{}{}", BLUE, message.get_content().trim(), RESET);

    if oneshot {
        return Ok(());
    }

    messages.push(message.clone());
    talk(gpt_client, messages, oneshot).await?;

    Ok(())
}

enum RoleType {
    User,
    System,
}
struct Role {
    t: RoleType,
}
impl Role {
    fn new() -> Self {
        Role { t: RoleType::User }
    }
    fn get_role_string(&self) -> String {
        match self.t {
            RoleType::User => "user".to_owned(),
            RoleType::System => "system".to_owned(),
        }
    }
    fn toggle(&mut self) {
        self.t = match self.t {
            RoleType::User => RoleType::System,
            RoleType::System => RoleType::User,
        };
    }
}

fn read_chatmessage() -> Option<ChatMessage> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let mut str = String::new();
    let mut role = Role::new();

    let get_inline = |r: &Role| -> String {
        let color = match r.t {
            RoleType::System => format!("{}", RED),
            RoleType::User => format!("{}", GREEN),
        };
        format!("{}[{}] {}", color, r.get_role_string(), RESET)
    };

    write!(stdout, "{}", get_inline(&role)).unwrap();
    stdout.flush().unwrap();

    for c in stdin.keys() {
        str = match c.unwrap() {
            Key::Char('\n') => break,
            Key::Char('\t') => {
                role.toggle();
                str
            }
            Key::Char(c) => format!("{}{}", str, c),
            Key::Backspace => {
                if str.len() > 0 {
                    let mut str = str.clone();
                    str.pop();
                    str
                } else {
                    str
                }
            }
            Key::Ctrl('c') => return None,
            _ => str,
        };
        let (_, y) = stdout.cursor_pos().unwrap();
        write!(
            stdout,
            "{}{}{}{}",
            termion::clear::CurrentLine,
            termion::cursor::Goto(0, y),
            get_inline(&role),
            str
        )
        .unwrap();
        stdout.flush().unwrap();
    }

    Some(gpt::ChatMessage::new(str, role.get_role_string()))
}

fn read_line(inline_message: &str) -> String {
    print!("{}", inline_message);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    // remove whitespace
    line.trim().to_string()
}
