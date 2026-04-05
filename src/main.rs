mod agents;
mod llm;
mod models;

use agents::planner::planner_prompt;
use agents::aggregator::aggregator_prompt;
use agents::worker::worker_prompt;
use llm::provider::call_llm;
use models::task::Plan;
use std::io::{self, Write};

fn extract_text_from_response(raw: &str) -> Result<String, String> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("Invalid response JSON: {e}"))?;

    let text = v
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.get(0))
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| "Missing response text".to_string())?;

    Ok(text.to_string())
}

fn extract_json_block(text: &str) -> Result<String, String> {
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        let after = if after.starts_with("json") {
            &after[4..]
        } else {
            after
        };

        if let Some(end) = after.find("```") {
            let block = &after[..end];
            return Ok(block.trim().to_string());
        }
    }

    let start = text.find('{').ok_or_else(|| "No JSON object start found".to_string())?;
    let end = text.rfind('}').ok_or_else(|| "No JSON object end found".to_string())?;
    if end <= start {
        return Err("Invalid JSON object range".to_string());
    }
    Ok(text[start..=end].to_string())
}

#[tokio::main]
async fn main() {
    println!("Enter a prompt (empty line to exit):");

    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read input.");
            continue;
        }

        let prompt = input.trim();
        if prompt.is_empty() {
            break;
        }

        let response = call_llm(&planner_prompt(prompt)).await;

        let plan = match extract_text_from_response(&response)
            .and_then(|text| extract_json_block(&text))
            .and_then(|json| serde_json::from_str::<Plan>(&json).map_err(|e| e.to_string()))
        {
            Ok(plan) => plan,
            Err(err) => {
                eprintln!("Failed to parse plan JSON: {}", err);
                eprintln!("Raw response:\n{}", response);
                continue;
            }
        };

        println!("Plan steps:");
        for (i, step) in plan.steps.iter().enumerate() {
            println!("{}. {}", i + 1, step);
        }

        let mut worker_results = Vec::new();
        if let Some(first_step) = plan.steps.first() {
            let worker_response = call_llm(&worker_prompt(first_step)).await;
            println!("\nWorker response for step 1:\n{}", worker_response);
            worker_results.push(worker_response);
        }

        if !worker_results.is_empty() {
            let agg_response = call_llm(&aggregator_prompt(&worker_results)).await;
            println!("\nAggregator response:\n{}", agg_response);
        }
    }
}
