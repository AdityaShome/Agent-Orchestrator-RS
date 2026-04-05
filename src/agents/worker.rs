pub fn worker_prompt(step: &str) -> String {
    format!("Execute this step:\n{}", step)
}
