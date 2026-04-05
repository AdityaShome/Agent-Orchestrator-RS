use reqwest::Client;
use serde_json::json;

pub async fn call_llm(prompt: &str) -> String {
    let _ = dotenvy::dotenv();
    let client = Client::new();

    let res = client
        .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent")
        .query(&[("key", std::env::var("GEMINI_API_KEY").unwrap())])
        .json(&json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }]
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    res
}
