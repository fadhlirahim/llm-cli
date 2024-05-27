use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{stdin, stdout, Write};

const MAX_TOKENS: u32 = 1000;
const URL: &str = "https://api.openai.com/v1/chat/completions";
const MODEL: &str = "gpt-3.5-turbo";

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OAIChoices {
    message: Message,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String,
}

#[derive(Deserialize, Debug)]
struct OAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>,
}

#[derive(Serialize, Debug)]
struct OAIRequest {
    messages: Vec<Message>,
    max_tokens: u32,
    model: String,
}

async fn process_user_input(
    client: &Client<HttpsConnector<hyper::client::HttpConnector>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let oai_token: String = env::var("OPENAI_API_KEY").unwrap();
    let auth_header_val = format!("Bearer {}", oai_token);

    println!("> ");
    stdout().flush().unwrap();
    let mut user_text = String::new();

    stdin()
        .read_line(&mut user_text)
        .expect("Failed to read line");

    let system_message = Message {
        role: "system".to_string(),
        content: "Answer in a casual, conversational manner. With humor, if possible. When presenting code examples always start with ```, and end with ```.".to_string()
    };

    let user_message = Message {
        role: "user".to_string(),
        content: user_text.trim().to_string(),
    };

    println!("");

    let spinner = Spinner::new(&Spinners::Dots9, "Thinking...".into());

    let oai_request = OAIRequest {
        messages: vec![system_message, user_message],
        max_tokens: MAX_TOKENS,
        model: MODEL.to_string(),
    };

    let body = Body::from(serde_json::to_vec(&oai_request)?);

    let req = Request::post(URL)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, &auth_header_val)
        .body(body)
        .unwrap();

    let res = client.request(req).await?;

    let body = hyper::body::aggregate(res).await?;

    spinner.stop();

    println!("");

    match serde_json::from_reader::<_, OAIResponse>(body.reader()) {
        Ok(json) => Ok(json.choices[0].message.content.to_string()),
        Err(e) => Err(Box::new(e)),
    }
}

fn clear_screen() {
    print!("{esc}c", esc = 27 as char);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);

    clear_screen();

    loop {
        match process_user_input(&client).await {
            Ok(response) => println!("{}", response),
            Err(e) => println!("Error: {}", e),
        }
    }
}
