mod engine;
mod parser;
mod verbs;

use anyhow::{Ok, Result};
use clap::Parser as ClapParser;
use engine::{extract_description, parse};
use minijinja::context;
use rig::{completion::Prompt, providers::ollama};

#[derive(ClapParser, Debug)]
#[command(name = "lakonik", version, about, trailing_var_arg = true)]
struct Cli {
    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Everything after flags is the prompt definition
    #[arg(num_args = 0..)]
    input: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let raw = cli.input.join(" ");

    let result = parse(&raw);
    // let json = serde_json::to_string(&result).expect("Failed to serialize result to JSON");

    let description = extract_description(&result);

    let environment = verbs::build_environment();
    let context = context! {
        description,
    };
    let template = environment
        .get_template(&format!("verbs/{}", &result.verb.name))
        .unwrap();
    let prompt = template.render(context).unwrap();

    println!("Prompt: {}", prompt);

    let client = ollama::Client::new();
    let agent = client.agent(&result.vocative.name).build();
    let response = agent.prompt(prompt).await.expect("could not get results");

    println!("Ollama completion response: {:?}", response);

    Ok(())
}
