use crate::model::{Question, Request};
use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use std::{fs, io::Read, path::PathBuf};

/// Unofficial CLI for TypeSafe Jev. State is read from stdin.
#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[arg(short, long, global = true, env = "JEV_PROVIDER", value_enum, default_value_t = ProviderKind::Typesafe)]
    pub provider: ProviderKind,

    #[arg(short, long, global = true, value_enum, default_value_t = Output::Json)]
    pub output: Output,

    /// Full TypeSafe request JSON ({"state", "questions"}); stdin when omitted or "-"
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ProviderKind {
    /// TYPESAFE_API_KEY
    Typesafe,
    /// CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_API_TOKEN
    Cloudflare,
    /// AI_GATEWAY_API_KEY
    Vercel,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Output {
    Json,
    Text,
}

#[derive(Subcommand)]
enum Command {
    /// Probability that the answer is yes
    Noul { question: String },
    /// Pick one of the options
    Choice {
        #[arg(long = "option", required = true, num_args = 1)]
        options: Vec<String>,
        question: String,
    },
    /// Rate along ordered levels, low to high
    Score {
        #[arg(long = "level", required = true, num_args = 1)]
        levels: Vec<String>,
        question: String,
    },
}

impl Cli {
    pub fn request(&self) -> Result<Request> {
        let Some(command) = &self.command else {
            let text = match self.file.as_deref() {
                Some(f) if f.as_os_str() != "-" => {
                    fs::read_to_string(f).with_context(|| format!("reading {}", f.display()))?
                }
                _ => read_stdin()?,
            };
            return serde_json::from_str(&text).context("invalid request");
        };
        ensure!(self.file.is_none(), "FILE cannot be combined with a subcommand");
        let question = command.question()?;
        Ok(Request { state: json!(read_stdin()?), questions: [("result".to_string(), question)].into() })
    }
}

impl Command {
    fn question(&self) -> Result<Question> {
        Ok(match self {
            Command::Noul { question } => Question::Noul { instructions: json!(question), criteria: None },
            Command::Choice { options, question } => {
                ensure!(options.len() >= 2, "choice needs at least two --option");
                // options are bare labels, no descriptions
                let criteria = options.iter().map(|o| (o.clone(), None)).collect();
                Question::Choice { instructions: json!(question), criteria }
            }
            Command::Score { levels, question } => {
                ensure!(levels.len() >= 2, "score needs at least two --level");
                let criteria = levels.iter().map(|l| json!(l)).collect();
                Question::Score { instructions: json!(question), criteria }
            }
        })
    }
}

fn read_stdin() -> Result<String> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).context("reading stdin")?;
    Ok(s)
}
