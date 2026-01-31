use std::{fs, io};

use thiserror::Error;

use crate::process::memory::MemoryRegion;


#[derive(Debug, Error)]
pub enum ProcessParseError {
    #[error("invalid integer: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
    #[error("IO error: {0}")]
    Io(#[from] io::Error)
}

pub struct Process {
    pub pid: u32,
    pub cmd_line: String,
    pub memory_regions: Vec<MemoryRegion>,
}

impl TryFrom<u32> for Process {
    type Error = ProcessParseError;

    fn try_from(pid: u32) -> Result<Self, Self::Error> {
        // get the string from cmdline
        let cmd_line = fs::read_to_string(format!("/proc/{}/cmdline", pid))?
            .replace('\0', " ")
            .trim()
            .to_string();

        let maps_content = fs::read_to_string(format!("/proc/{}/maps", pid))?;
        let memory_regions = maps_content.lines().filter_map(|line| line.parse().ok())
        .collect();

        Ok(Process { pid, cmd_line, memory_regions })
    }
    
}


