mod ast;
mod engine;
mod hir;
mod lsp;
mod templates;

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use engine::run_prompt_builder;
use lsp::run_lsp_server;

#[derive(Parser, Debug)]
#[command(name = "lakonik", version, about, trailing_var_arg = true)]
struct Cli {
    /// Choose a subcommand: `eval` or `lsp`
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Evaluate a prompt and output JSON
    Eval {
        /// Verbose mode
        #[arg(short, long)]
        verbose: bool,

        /// Everything after flags is the prompt definition
        #[arg(num_args = 0..)]
        input: Vec<String>,
    },
    /// Language server protocol placeholder: prints "hello world"
    Lsp {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Eval { verbose, input } => {
            cmd_eval(*verbose, input).await?;
        }
        Commands::Lsp {} => {
            cmd_lsp().await;
        }
    }

    Ok(())
}

async fn cmd_eval(verbose: bool, input: &[String]) -> Result<()> {
    if verbose {
        eprintln!("Running in verbose mode...");
    }
    let raw = input.join(" ");
    let prompt_builder_result = run_prompt_builder(&raw);
    let json =
        serde_json::to_string(&prompt_builder_result).expect("Failed to serialize result to JSON");

    println!("{}", json);
    Ok(())
}

async fn cmd_lsp() {
    run_lsp_server().await;
}
