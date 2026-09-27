use anyhow::Result;
use clap::Parser;
use jev::{cli, output, provider};

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    let res = provider::of(cli.provider).evaluate(&cli.request()?)?;
    print!("{}", output::render(cli.output, cli.provider, &res));
    Ok(())
}
