use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("missing field: {0}")]
    MissingField(&'static str),
    #[error("invalid address range: {0}")]
    InvalidAddress(String),
    #[error("invalid permissions: {0}")]
    InvalidPermissions(String),
    #[error("invalid device: {0}")]
    InvalidDevice(String),
    #[error("invalid integer: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
}

pub struct Permissions {
    read: bool,     // r or -
    write: bool,    // w or -
    execute: bool,  // x or -
    shared: bool,   // s (shared) or p (private/copy-on-write)
}

impl FromStr for Permissions {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(ParseError::InvalidPermissions(s.to_string()));
        }

        Ok(Permissions{
            read: bytes[0] == b'r',
            write: bytes[1] == b'w',
            execute: bytes[2] == b'x',
            shared: bytes[3] == b's',
        })
    }
}

pub enum PathType {
    // Actual file on disk
    File(PathBuf),
    
    // Special kernel-provided regions
    Stack,              // [stack]
    Heap,               // [heap]
    Vdso,               // [vdso] - virtual dynamic shared object
    Vvar,               // [vvar] - kernel variables
    Vsyscall,           // [vsyscall] - legacy syscall page
    
    // No path at all - truly anonymous
    Anonymous,
    
    // Deleted file (still mapped but unlinked)
    Deleted(PathBuf),
}

impl FromStr for PathType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            match s {
                "" => PathType::Anonymous,
                "[heap]" => PathType::Heap,
                "[stack]" => PathType::Stack,
                "[vdso]" => PathType::Vdso,
                "[vvar]" => PathType::Vvar,
                "[vsyscall]" => PathType::Vsyscall,
                path if path.ends_with(" (deleted)") => {
                    let actual_path = path.trim_end_matches(" (deleted)");
                    PathType::Deleted(PathBuf::from(actual_path))
                }
                path => PathType::File(PathBuf::from(path)),
            }
        )
    }
}


pub struct MemoryRegion {
    start: u64,
    end: u64,
    permissions: Permissions,
    offset: u64,
    device: (u8, u8),
    inode: u64,
    path_name: Option<PathType>
}

impl FromStr for MemoryRegion {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        // Format: "start-end perms offset dev inode pathname"
        // Example: "7f2a1b3c4000-7f2a1b5c4000 r-xp 00001000 08:01 1234567 /usr/lib/libc.so.6"
        
        let mut parts = line.splitn(6, char::is_whitespace)
            .filter(|s| !s.is_empty());

        // Parse address range
        let addr_range = parts.next()
            .ok_or(ParseError::MissingField("address range"))?;
        let (start, end) = parse_address_range(addr_range)?;

        // Parse permissions
        let perms_str = parts.next()
            .ok_or(ParseError::MissingField("permissions"))?;
        let permissions = perms_str.parse()?;

        // Parse offset
        let offset_str = parts.next()
            .ok_or(ParseError::MissingField("offset"))?;
        let offset = u64::from_str_radix(offset_str, 16)?;

        // Parse device major:minor
        let dev_str = parts.next()
            .ok_or(ParseError::MissingField("device"))?;
        let device = parse_device(dev_str)?;

        // Parse inode
        let inode_str = parts.next()
            .ok_or(ParseError::MissingField("inode"))?;
        let inode = inode_str.parse()?;

        // Pathname is optional and may contain spaces
        let path_name = parts.next()
            .map(|s| s.trim().parse())
            .transpose()?;

        Ok(MemoryRegion {
            start,
            end,
            permissions,
            offset,
            device,
            inode,
            path_name,
        })
    }
}

fn parse_address_range(s: &str) -> Result<(u64, u64), ParseError> {
    let (start_str, end_str) = s
        .split_once('-')
        .ok_or_else(|| ParseError::InvalidAddress(s.to_string()))?;

    let start = u64::from_str_radix(start_str, 16)?;
    let end = u64::from_str_radix(end_str, 16)?;

    Ok((start, end))
}

fn parse_device(s: &str) -> Result<(u8, u8), ParseError> {
    let (major_str, minor_str) = s
        .split_once(':')
        .ok_or_else(|| ParseError::InvalidDevice(s.to_string()))?;

    let major = u8::from_str_radix(major_str, 16)?;
    let minor = u8::from_str_radix(minor_str, 16)?;

    Ok((major, minor))
}