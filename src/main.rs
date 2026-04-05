mod agents;
mod llm;
mod models;
mod orchestrator;

use std::io::{self, Write};

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

        orchestrator::run(prompt).await;
    }
}
