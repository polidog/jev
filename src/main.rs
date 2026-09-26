mod cli;
mod http;
mod model;
mod output;
mod provider;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let res = provider::of(cli.provider).evaluate(&cli.request()?)?;
    print!("{}", output::render(cli.output, cli.provider, &res));
    Ok(())
}
