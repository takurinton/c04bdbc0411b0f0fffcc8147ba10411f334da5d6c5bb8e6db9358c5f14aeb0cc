use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct ChatGPTMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatGPTChoice {
    message: ChatGPTMessage,
}

#[derive(Deserialize)]
struct ChatGPTResponse {
    choices: Vec<ChatGPTChoice>,
}

pub async fn fetch_chatgpt(content: String, prompts: Vec<String>) -> String {
    let client = reqwest::Client::new();
    let prompts = prompts
        .iter()
        .map(|p| format!(r#"{{ "role": "system", "content": "{}" }},"#, p))
        .collect::<Vec<String>>()
        .join("");
    let response = match client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not defined"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(format!(
            r#"{{ "model": "gpt-3.5-turbo", "messages": [{}{{ "role": "user", "content": "{}" }}] }}"#,
            prompts,
            content
        ))
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return "通信エラーが発生しました。".to_string();
        }
    };

    let body = match response.json::<ChatGPTResponse>().await {
        Ok(body) => body,
        Err(_) => {
            return "コンテンツの取得に失敗しました。".to_string();
        }
    };

    body.choices[0].message.content.clone()
}