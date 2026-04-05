pub fn aggregator_prompt(results: &Vec<String>) -> String {
    format!("Combine results into final answer:\n{:?}", results)
}
