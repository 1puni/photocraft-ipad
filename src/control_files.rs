//! Bounded, page-lifetime file capabilities for the browser control transport.
//! These names never become host filesystem paths.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::HashMap;

const MAX_FILE: usize = 64 << 20;
const MAX_TOTAL: usize = 128 << 20;
const MAX_CHUNK: usize = 512 << 10;

struct File {
    expected: usize,
    bytes: Vec<u8>,
}

#[derive(Default)]
pub struct Files {
    entries: HashMap<String, File>,
}

fn name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 255
        || value.contains(['/', '\\', '\0'])
        || matches!(value, "." | "..")
    {
        return Err("use a plain file name, not a filesystem path".into());
    }
    Ok(())
}

impl Files {
    fn reserve(&self, key: &str, size: usize) -> Result<(), String> {
        name(key)?;
        if size > MAX_FILE {
            return Err("file exceeds 64 MiB".into());
        }
        let used: usize = self
            .entries
            .iter()
            .filter(|(k, _)| *k != key)
            .map(|(_, f)| f.expected)
            .sum();
        if used.saturating_add(size) > MAX_TOTAL
            || (self.entries.len() >= 16 && !self.entries.contains_key(key))
        {
            return Err("browser file store is full; remove staged files".into());
        }
        Ok(())
    }

    pub fn read_file(&self, key: &str) -> Result<(String, Vec<u8>), String> {
        name(key)?;
        let file = self
            .entries
            .get(key)
            .ok_or("file has not been transferred to this browser")?;
        if file.bytes.len() != file.expected {
            return Err("file transfer is incomplete".into());
        }
        Ok((key.to_owned(), file.bytes.clone()))
    }

    pub fn write_file(&mut self, key: &str, bytes: &[u8]) -> Result<(), String> {
        self.reserve(key, bytes.len())?;
        self.entries.insert(
            key.into(),
            File {
                expected: bytes.len(),
                bytes: bytes.to_vec(),
            },
        );
        Ok(())
    }

    pub fn call(&mut self, method: &str, params: &Value) -> Result<Value, String> {
        let key = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or("missing file name")?;
        name(key)?;
        match method {
            "transfer.begin" => {
                let size = number(params, "size")?;
                self.reserve(key, size)?;
                self.entries.insert(
                    key.into(),
                    File {
                        expected: size,
                        bytes: Vec::new(),
                    },
                );
                Ok(json!({"name": key, "size": size, "chunkBytes": MAX_CHUNK}))
            }
            "transfer.write" => {
                let offset = number(params, "offset")?;
                let encoded = params
                    .get("base64")
                    .and_then(Value::as_str)
                    .ok_or("missing base64")?;
                if encoded.len() > MAX_CHUNK.div_ceil(3) * 4 {
                    return Err("chunk exceeds 512 KiB".into());
                }
                let bytes = STANDARD.decode(encoded).map_err(|e| e.to_string())?;
                if bytes.len() > MAX_CHUNK {
                    return Err("chunk exceeds 512 KiB".into());
                }
                let file = self
                    .entries
                    .get_mut(key)
                    .ok_or("begin the transfer first")?;
                let end = offset.checked_add(bytes.len()).ok_or("offset overflow")?;
                if end > file.expected {
                    return Err("chunk exceeds declared file size".into());
                }
                if offset == file.bytes.len() {
                    file.bytes.extend_from_slice(&bytes);
                } else if file.bytes.get(offset..end) != Some(bytes.as_slice()) {
                    return Err("unexpected offset or conflicting repeated chunk".into());
                }
                Ok(
                    json!({"received": file.bytes.len(), "complete": file.bytes.len() == file.expected}),
                )
            }
            "transfer.info" => {
                let file = self.entries.get(key).ok_or("unknown staged file")?;
                Ok(
                    json!({"name": key, "size": file.expected, "received": file.bytes.len(), "complete": file.bytes.len() == file.expected}),
                )
            }
            "transfer.read" => {
                let offset = number(params, "offset")?;
                let count = number(params, "length")?;
                if count > MAX_CHUNK {
                    return Err("chunk exceeds 512 KiB".into());
                }
                let file = self.entries.get(key).ok_or("unknown staged file")?;
                if file.bytes.len() != file.expected {
                    return Err("file transfer is incomplete".into());
                }
                let end = offset
                    .checked_add(count)
                    .ok_or("offset overflow")?
                    .min(file.bytes.len());
                let bytes = file.bytes.get(offset..end).ok_or("offset beyond file")?;
                Ok(
                    json!({"offset": offset, "base64": STANDARD.encode(bytes), "size": file.bytes.len()}),
                )
            }
            "transfer.remove" => {
                self.entries.remove(key);
                Ok(json!({"removed": key}))
            }
            _ => Err(format!("unknown transfer method {method}")),
        }
    }
}

fn number(params: &Value, key: &str) -> Result<usize, String> {
    params
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("missing or invalid {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transfer_retries_are_idempotent_and_incomplete_files_cannot_open() {
        let mut files = Files::default();
        files
            .call("transfer.begin", &json!({"name":"drawing.psd","size":4}))
            .unwrap();
        let first = json!({"name":"drawing.psd","offset":0,"base64":STANDARD.encode([1,2])});
        files.call("transfer.write", &first).unwrap();
        files.call("transfer.write", &first).unwrap();
        assert!(files.read_file("drawing.psd").is_err());
        assert!(
            files
                .call(
                    "transfer.write",
                    &json!({"name":"drawing.psd","offset":0,"base64":STANDARD.encode([9,9])})
                )
                .is_err()
        );
        files
            .call(
                "transfer.write",
                &json!({"name":"drawing.psd","offset":2,"base64":STANDARD.encode([3,4])}),
            )
            .unwrap();
        assert_eq!(files.read_file("drawing.psd").unwrap().1, [1, 2, 3, 4]);
    }
    #[test]
    fn encoded_export_reads_exact_bytes_and_rejects_paths_and_limits() {
        let mut files = Files::default();
        files.write_file("saved.psd", &[0, 128, 255]).unwrap();
        let value = files
            .call(
                "transfer.read",
                &json!({"name":"saved.psd","offset":1,"length":8}),
            )
            .unwrap();
        assert_eq!(
            STANDARD.decode(value["base64"].as_str().unwrap()).unwrap(),
            [128, 255]
        );
        assert!(files.write_file("../secret", &[1]).is_err());
        assert!(files.read_file("/etc/passwd").is_err());
        assert!(
            files
                .call(
                    "transfer.begin",
                    &json!({"name":"large.psd","size":MAX_FILE+1})
                )
                .is_err()
        );
        assert!(
            files
                .call(
                    "transfer.read",
                    &json!({"name":"saved.psd","offset":9,"length":1})
                )
                .is_err()
        );
    }
}
