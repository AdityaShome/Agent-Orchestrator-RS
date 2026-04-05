use crate::agents::aggregator::aggregator_prompt;
use crate::agents::planner::planner_prompt;
use crate::agents::worker::worker_prompt;
use crate::llm::provider::call_llm;
use crate::models::task::Plan;

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

fn parse_plan(raw: &str) -> Result<Plan, String> {
    extract_text_from_response(raw)
        .and_then(|text| extract_json_block(&text))
        .and_then(|json| serde_json::from_str::<Plan>(&json).map_err(|e| e.to_string()))
}

pub async fn run(task: &str) {
    let plan_raw = call_llm(&planner_prompt(task)).await;

    let plan = match parse_plan(&plan_raw) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("Failed to parse plan JSON: {}", err);
            eprintln!("Raw response:\n{}", plan_raw);
            return;
        }
    };

    let mut results = Vec::new();
    for step in &plan.steps {
        let res = call_llm(&worker_prompt(step)).await;
        results.push(res);
    }

    let final_output = call_llm(&aggregator_prompt(&results)).await;
    println!("{}", final_output);
}
