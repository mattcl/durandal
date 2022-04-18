mod cli;
mod config;
mod iou_client;
mod parser;
mod task;
mod task_table;
mod workflow;

fn main() -> Result<(), anyhow::Error> {
    cli::Cli::run()
}
