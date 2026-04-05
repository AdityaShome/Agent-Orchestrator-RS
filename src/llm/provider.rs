use reqwest::Client;
use serde_json::json;

struct LlmError {
    msg: String,
    retryable: bool,
}

fn is_retryable_status(status: reqwest::StatusCode, body: &str) -> bool {
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        return true;
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return true;
    }
    let body_lower = body.to_lowercase();
    body_lower.contains("resource_exhausted") || body_lower.contains("quota")
}

async fn call_gemini_content(prompt: &str) -> Result<String, LlmError> {
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
        .map_err(|e| LlmError {
            msg: e.to_string(),
            retryable: true,
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(LlmError {
            msg: format!("gemini status {}: {}", status, body),
            retryable: is_retryable_status(status, &body),
        });
    }

    let body = res.text().await.map_err(|e| LlmError {
        msg: e.to_string(),
        retryable: true,
    })?;

    let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| LlmError {
        msg: format!("gemini invalid json: {e}"),
        retryable: false,
    })?;

    let text = v
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| LlmError {
            msg: "gemini missing response text".to_string(),
            retryable: false,
        })?;

    Ok(text.to_string())
}

async fn call_groq_content(prompt: &str) -> Result<String, String> {
    let client = Client::new();
    let api_key = std::env::var("GROQ_API_KEY").map_err(|_| "GROQ_API_KEY not set".to_string())?;

    let res = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "llama-3.1-8b-instant",
            "messages": [{
                "role": "user",
                "content": prompt
            }]
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("groq status {}: {}", status, body));
    }

    let body = res.text().await.map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    let text = v
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "groq missing response text".to_string())?;

    Ok(text.to_string())
}

pub async fn call_llm(prompt: &str) -> Result<String, String> {
    let _ = dotenvy::dotenv();
    match call_gemini_content(prompt).await {
        Ok(text) => Ok(text),
        Err(err) if err.retryable => {
            eprintln!("Gemini failed, falling back to Groq: {}", err.msg);
            call_groq_content(prompt).await
        }
        Err(err) => Err(err.msg),
    }
}
