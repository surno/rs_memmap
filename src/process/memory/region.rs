use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryParseError {
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
    type Err = MemoryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            return Err(MemoryParseError::InvalidPermissions(s.to_string()));
        }

        Ok(Permissions{
            read: bytes[0] == b'r',
            write: bytes[1] == b'w',
            execute: bytes[2] == b'x',
            shared: bytes[3] == b's',
        })
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.read { 'r' } else { '-' },
            if self.write { 'w' } else { '-' },
            if self.execute { 'x' } else { '-' },
            if self.shared { 's' } else { 'p' },
        )
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
    type Err = MemoryParseError;

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

impl fmt::Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathType::Anonymous => write!(f, "[anonymous]"),
            PathType::Heap => write!(f, "[heap]"),
            PathType::Stack => write!(f, "[stack]"),
            PathType::Vdso => write!(f, "[vdso]"),
            PathType::Vvar => write!(f, "[vvar]"),
            PathType::Vsyscall => write!(f, "[vsyscall]"),
            PathType::File(path) => write!(f, "{}", path.display()),
            PathType::Deleted(path) => write!(f, "{} (deleted)", path.display()),
        }
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

impl MemoryRegion {
    fn size(&self) -> u64 {
        return self.end - self.start;
    }
}

impl FromStr for MemoryRegion {
    type Err = MemoryParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        // Format: "start-end perms offset dev inode pathname"
        // Example: "7f2a1b3c4000-7f2a1b5c4000 r-xp 00001000 08:01 1234567 /usr/lib/libc.so.6"
        
        let mut parts = line.splitn(6, char::is_whitespace)
            .filter(|s| !s.is_empty());

        // Parse address range
        let addr_range = parts.next()
            .ok_or(MemoryParseError::MissingField("address range"))?;
        let (start, end) = parse_address_range(addr_range)?;

        // Parse permissions
        let perms_str = parts.next()
            .ok_or(MemoryParseError::MissingField("permissions"))?;
        let permissions = perms_str.parse()?;

        // Parse offset
        let offset_str = parts.next()
            .ok_or(MemoryParseError::MissingField("offset"))?;
        let offset = u64::from_str_radix(offset_str, 16)?;

        // Parse device major:minor
        let dev_str = parts.next()
            .ok_or(MemoryParseError::MissingField("device"))?;
        let device = parse_device(dev_str)?;

        // Parse inode
        let inode_str = parts.next()
            .ok_or(MemoryParseError::MissingField("inode"))?;
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

fn parse_address_range(s: &str) -> Result<(u64, u64), MemoryParseError> {
    let (start_str, end_str) = s
        .split_once('-')
        .ok_or_else(|| MemoryParseError::InvalidAddress(s.to_string()))?;

    let start = u64::from_str_radix(start_str, 16)?;
    let end = u64::from_str_radix(end_str, 16)?;

    Ok((start, end))
}

fn parse_device(s: &str) -> Result<(u8, u8), MemoryParseError> {
    let (major_str, minor_str) = s
        .split_once(':')
        .ok_or_else(|| MemoryParseError::InvalidDevice(s.to_string()))?;

    let major = u8::from_str_radix(major_str, 16)?;
    let minor = u8::from_str_radix(minor_str, 16)?;

    Ok((major, minor))
}

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;

    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}

impl fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:012x}-{:012x} {} {:8x} {:>10} {:02x}:{:02x} {:8} {}",
            self.start,
            self.end,
            self.permissions,
            self.offset,
            format_size(self.size()),
            self.device.0,
            self.device.1,
            self.inode,
            self.path_name.as_ref().map_or(String::new(), |p| p.to_string()),
        )
    }
}