pub fn planner_prompt(task: &str) -> String {
    format!(
        "Break task into steps. Return JSON only.\n\
Task: {}\n\
Rules:\n\
- Output MUST be a single JSON object.\n\
- \"steps\" MUST be an array of strings.\n\
- Do NOT use objects in the steps array.\n\
- Do NOT wrap in markdown or code fences.\n\
Format: {{\"steps\":[\"step 1\",\"step 2\"]}}",
        task
    )
}
