mod parser;

use anyhow::{Context, Ok, Result, anyhow};
use clap::Parser as ClapParser;
use parser::parse_statement;
use rig::{completion::Prompt, providers::ollama};

#[derive(ClapParser, Debug)]
#[command(name = "my_cli", version, about, trailing_var_arg = true)]
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

    let (_, result) = parse_statement(&raw)
        .map_err(|e| anyhow!("{e:?}"))
        .with_context(|| format!("Failed to parse input: {raw:?}"))?;
    println!("Raw sentence: {}", raw);
    println!("Parsed sentence: {:#?}", result);

    let freeform_part = result
        .parts
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let prompt = format!("{} {}", &result.verb.name, freeform_part);

    let client = ollama::Client::new();
    let agent = client.agent(&result.vocative.name).build();
    let response = agent.prompt(prompt).await.expect("could not get results");

    println!("Ollama completion response: {:?}", response);

    Ok(())
}
