use clap::Parser;

mod process;
mod app;

use crate::{app::App, process::Process};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pid: u32,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let process = Process::try_from(args.pid)?;

    color_eyre::install()?;
    ratatui::run(|t| App::new(process).run(t))?;
    Ok(())
}