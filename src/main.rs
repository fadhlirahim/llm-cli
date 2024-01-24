use hyper::body::Buf;
use hyper::{header, Body, Client, Request};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{stdin, stdout, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String
}

#[derive(Deserialize, Debug)]
struct OAIChoices {
    message: Message,
    index: u8,
    logprobs: Option<u8>,
    finish_reason: String
}

#[derive(Deserialize, Debug)]
struct OAIResponse {
    id: Option<String>,
    object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>
}

#[derive(Serialize, Debug)]
struct OAIRequest {
    messages: Vec<Message>,
    max_tokens: u32,
    model: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let max_tokens = 100;
    let https = HttpsConnector::new();
    let client = Client::builder().build(https);
    let uri = "https://api.openai.com/v1/chat/completions";
    let oai_token: String = env::var("OPENAI_TOKEN").unwrap();
    let auth_header_val = format!("Bearer {}", oai_token);

    println!("{esc}c", esc = 27 as char);

    loop {
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
            content: user_text.trim().to_string()
        };

        println!("");

        let spinner = Spinner::new(&Spinners::Dots9, "Thinking...".into());

        let oai_request = OAIRequest {
            messages: vec![system_message, user_message],
            max_tokens: max_tokens,
            model: "gpt-3.5-turbo".to_string(),
        };

        let body = Body::from(serde_json::to_vec(&oai_request)?);

        let req = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, &auth_header_val)
            .body(body)
            .unwrap();

        let res = client.request(req).await?;

        let body = hyper::body::aggregate(res).await?;

        let json: OAIResponse = serde_json::from_reader(body.reader())?;

        spinner.stop();

        println!("");

        println!("{}", json.choices[0].message.content);
    }
}
