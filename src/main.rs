use clap::Parser; 
use nom::{
    IResult,
    Parser as NomParser, 
    branch::alt,
    bytes::complete::tag,
    character::complete::space1,
    combinator::eof,
};
use anyhow::Result;

/// Names the entity you are talking to
#[derive(Debug)]
enum Vocative {
    Foo,
    Bar,
}

/// Verbs are actions/capabilities the entity is expected to perform
#[derive(Debug)]
enum Verb {
    Lorem,
    Ipsum,
}

/// The fully-parsed sentence. Describes a prompt.
#[derive(Debug)]
struct Sentence {
    vocative: Vocative,
    verb: Verb,
}

impl std::fmt::Display for Sentence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.vocative, self.verb)
    }
}

#[derive(Parser, Debug)]
#[command(name = "my_cli", version, about, trailing_var_arg = true)]
struct Cli {
    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Everything after flags is the prompt definition
    #[arg(num_args = 0..)]
    input: Vec<String>,
}

fn parse_sentence(input: &str) -> IResult<&str, Sentence> {
    let (rest, (voc_s, _, verb_s)) = (
        alt((tag("foo"), tag("bar"))),
        space1,
        alt((tag("lorem"), tag("ipsum"))),
    )
        .parse(input)?;
    let (_rest2, _) = eof(rest)?;

    let voc = match voc_s {
        "foo" => Vocative::Foo,
        "bar" => Vocative::Bar,
        _ => unreachable!(),
    };
    let verb = match verb_s {
        "lorem" => Verb::Lorem,
        "ipsum" => Verb::Ipsum,
        _ => unreachable!(),
    };

    Ok((
        rest,
        Sentence {
            vocative: voc,
            verb,
        },
    ))
}

fn main() {
    let cli = Cli::parse();
    let raw = cli.input.join(" ");

    if raw.trim().is_empty() {
        eprintln!("You need to specify who you are talking to. Available vocatives: foo, bar");
        std::process::exit(1);
    }

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    match tokens.len() {
        1 => {
            match tokens[0] {
                "foo" | "bar" => {
                    eprintln!(
                        "Need to specify what action you expect. Available verbs: lorem, ipsum"
                    );
                }
                _ => {
                    eprintln!("Unrecognized vocative. Available vocatives: foo, bar");
                }
            }
            std::process::exit(1);
        }
        2 => match parse_sentence(&raw) {
            Ok((_, sentence)) => {
                if cli.verbose {
                    eprintln!("✔ Parsed sentence: {}", sentence);
                } else {
                    println!("{}", sentence);
                }
            }
            Err(_) => {
                eprintln!("Unrecognized verb. Available verbs: lorem, ipsum");
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("Too many words. Sentence must be exactly two words.");
            std::process::exit(1);
        }
    }
}

