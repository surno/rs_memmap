use std::fs;

use clap::Parser;

use crate::memory::MemoryRegion;

mod memory;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    pid: u32,
}

fn main() {
    let args = Args::parse();
    let regions = read_maps(args.pid).unwrap_or_else(|err| 
        panic!("Unable to read file: {}", err)
    );

    for region in regions {
        println!("{}",region);
    }
}


fn read_maps(pid: u32) -> Result<Vec<MemoryRegion>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(format!("/proc/{pid}/maps"))?;
    
    content
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
