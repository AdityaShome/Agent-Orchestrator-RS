use crate::agents::aggregator::aggregator_prompt;
use crate::agents::planner::planner_prompt;
use crate::agents::worker::worker_prompt;
use crate::llm::provider::call_llm;
use crate::models::task::Plan;

fn extract_json_block(text: &str) -> Result<String, String> {
    // Prefer fenced JSON blocks, but fall back to the first object span.
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

fn parse_plan(text: &str) -> Result<Plan, String> {
    // Enforce the Plan schema, with a fallback to flatten object steps.
    let json = extract_json_block(&text)?;

    match serde_json::from_str::<Plan>(&json) {
        Ok(plan) => Ok(plan),
        Err(first_err) => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(steps) = value.get("steps").and_then(|s| s.as_array()) {
                    let mut flattened = Vec::new();
                    for step in steps {
                        if let Some(s) = step.as_str() {
                            flattened.push(s.to_string());
                        } else if let Some(obj) = step.as_object() {
                            if let Some(desc) = obj.get("description").and_then(|d| d.as_str()) {
                                flattened.push(desc.to_string());
                            }
                        }
                    }
                    if !flattened.is_empty() {
                        return Ok(Plan { steps: flattened });
                    }
                }
            }
            Err(first_err.to_string())
        }
    }
}

pub async fn run(task: &str) {
    // Planner phase: get JSON plan from the model.
    let plan_text = match call_llm(&planner_prompt(task)).await {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Planner call failed: {}", err);
            return;
        }
    };

    let plan = match parse_plan(&plan_text) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("Failed to parse plan JSON: {}", err);
            eprintln!("Raw response:\n{}", plan_text);
            return;
        }
    };

    // Debug logging: plan and step outputs help diagnose agent behavior.
    println!("Plan:");
    for (i, step) in plan.steps.iter().enumerate() {
        println!("{}. {}", i + 1, step);
    }

    let mut results = Vec::new();
    for step in &plan.steps {
        println!("\nRunning step: {}", step);
        let res = match call_llm(&worker_prompt(step)).await {
            Ok(text) => text,
            Err(err) => {
                eprintln!("Worker call failed: {}", err);
                return;
            }
        };
        println!("Step result:\n{}", res);
        results.push(res);
    }

    let final_output = match call_llm(&aggregator_prompt(&results)).await {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Aggregator call failed: {}", err);
            return;
        }
    };
    println!("\nFinal output:");
    println!("{}", final_output);
}
