mod cli;

fn main() -> Result<(), anyhow::Error> {
    cli::Cli::run()
}
