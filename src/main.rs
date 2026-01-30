use std::fs;

use clap::Parser;

mod memory;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pid: usize,
}

fn main() {
    let args = Args::parse();
    let maps_path = format!("/proc/{}/maps", args.pid);

    println!("Attempting to open memmap of process: {} - maps path: {}, contents:", args.pid, maps_path);

    let contents = fs::read_to_string(maps_path).unwrap_or_else(|err| 
        panic!("Unable to read file: {}", err)
    );

    println!("{}", contents);

}
