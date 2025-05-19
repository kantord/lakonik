mod parser;

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
async fn main() {
    let cli = Cli::parse();
    let raw = cli.input.join(" ");

    if raw.trim().is_empty() {
        eprintln!("You need to specify who you are talking to. Available vocatives: foo, bar");
        std::process::exit(1);
    }

    let (_, result) = parse_statement(&raw).expect("Failed to parse statement");
    println!("Raw sentence: {}", raw);
    println!("Parsed sentence: {:#?}", result);

    let client = ollama::Client::new();
    let agent = client.agent(&result.vocative.name).build();
    let response = agent
        .prompt(&result.verb.name)
        .await
        .expect("could not get results");

    println!("Ollama completion response: {:?}", response);
}
