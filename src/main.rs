mod parser;

use clap::Parser as ClapParser;
use parser::parse_statement;
use rllm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, ChatRole},
};

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

fn main() {
    let cli = Cli::parse();
    let raw = cli.input.join(" ");

    if raw.trim().is_empty() {
        eprintln!("You need to specify who you are talking to. Available vocatives: foo, bar");
        std::process::exit(1);
    }

    let (_, result) = parse_statement(&raw).expect("Failed to parse statement");
    println!("Raw sentence: {}", raw);
    println!("Parsed sentence: {:#?}", result);

    let llm = LLMBuilder::new()
        .backend(LLMBackend::Ollama)
        .model(result.vocative.name)
        .build()
        .expect("Failed to create LLM backend");

    let messages = vec![ChatMessage {
        role: ChatRole::User,
        message_type: Default::default(),
        content: "Tell me that you love cats".into(),
    }];

    let _ = llm.chat(&messages);
}
