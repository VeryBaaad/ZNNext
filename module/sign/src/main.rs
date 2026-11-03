use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

use machikado_rs::{FileEntry, sign_file_entries};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 4 {
        return Err(invalid("usage: znn-sign <root> <private-key> <jobs>").into());
    }

    let root = Path::new(&arguments[1]);
    let key = fs::read(&arguments[2])?;
    if key.len() != 64 {
        return Err(invalid(&format!(
            "private key must be 64 bytes, found {}",
            key.len()
        ))
        .into());
    }
    let key: [u8; 64] = key.try_into().map_err(|_| invalid("invalid private key"))?;

    let jobs = fs::read_to_string(&arguments[3])?;
    let mut groups: BTreeMap<String, Vec<FileEntry>> = BTreeMap::new();
    for (index, line) in jobs.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let output = fields
            .next()
            .ok_or_else(|| invalid(&format!("line {}: missing output", index + 1)))?;
        let target = fields
            .next()
            .ok_or_else(|| invalid(&format!("line {}: missing target", index + 1)))?;
        let source = fields.next().unwrap_or("");
        let source = if source.is_empty() { target } else { source };
        if fields.next().is_some() {
            return Err(invalid(&format!("line {}: too many fields", index + 1)).into());
        }
        let content = fs::read(root.join(source))?;
        groups
            .entry(output.to_string())
            .or_default()
            .push(FileEntry {
                relative_path: target.to_string(),
                content,
            });
    }

    for (output, mut entries) in groups {
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        let blob = sign_file_entries(&entries, &key)?;
        fs::write(root.join(output), blob.as_bytes())?;
    }

    Ok(())
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
