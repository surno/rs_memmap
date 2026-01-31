use std::{fs, io, str::FromStr};

use thiserror::Error;

use crate::process::memory::{MemoryRegion, region::{MemoryParseError, DetailedMemoryRegion, parse_detail_into_region}};


#[derive(Debug, Error)]
pub enum ProcessParseError {
    #[error("invalid integer: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
    #[error("Memory Parsing error: {0}")]
    MemoryParseError(#[from] MemoryParseError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error)
}

pub struct Process {
    pub pid: u32,
    pub cmd_line: String,
    pub memory_regions: Vec<DetailedMemoryRegion>,
}

impl TryFrom<u32> for Process {
    type Error = ProcessParseError;

    fn try_from(pid: u32) -> Result<Self, Self::Error> {
        // get the string from cmdline
        let cmd_line = fs::read_to_string(format!("/proc/{}/cmdline", pid))?
            .replace('\0', " ")
            .trim()
            .to_string();

        let smaps_content = fs::read_to_string(format!("/proc/{}/smaps", pid))?;
        let mut lines = smaps_content.lines().peekable();
        let mut memory_regions  = Vec::new();

        while let Some(line) = lines.next() {
            if is_address_line(line) {
                let base_region = MemoryRegion::from_str(line)?;
                let mut region = DetailedMemoryRegion::from_region(base_region);

                while let Some(next) = lines.peek() {
                    if is_address_line(next) {
                        break;
                    }
                    let detail = lines.next().unwrap();
                    parse_detail_into_region(&mut region, detail);
                }
                memory_regions.push(region);
            }
        }

        Ok(Process { pid, cmd_line, memory_regions})
    }
    
}

fn is_address_line(line: &str) -> bool {
    line.chars().next().map_or(false, |c| c.is_ascii_hexdigit())
}


