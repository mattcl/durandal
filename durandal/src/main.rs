use cli::Cli;

mod cli;

fn main() -> Result<(), anyhow::Error> {
    Cli::run()
}
