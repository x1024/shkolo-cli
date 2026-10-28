//! Saving message attachments.
//!
//! Shkolo serves a message's attachments as one zip (`ShkoloClient::download_message_files`, the
//! same request as the app's "download all"), whose entries have transliterated names
//! ("Prilozenie__1A_...docx" for "Приложение_№_1А_...docx"). The thread's message lists the files
//! with their real names, in the same order, so each entry is saved under its real name.

use anyhow::{anyhow, Result};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};

/// One attachment of a message, as the thread lists it
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub id: i64,
    pub name: String,
    pub extension: Option<String>,
    pub mimetype: Option<String>,
}

/// A file written to disk
#[derive(Debug, Clone, Serialize)]
pub struct Saved {
    pub id: Option<i64>,
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub mimetype: Option<String>,
}

impl From<&crate::models::MessageFile> for Attachment {
    fn from(f: &crate::models::MessageFile) -> Self {
        Attachment { id: f.id, name: f.name.clone(), extension: f.extension.clone(), mimetype: f.mimetype.clone() }
    }
}

/// The attachments of a raw message (`files` in `/v1/messenger/threads/{id}/messages`)
pub fn attachments(message: &serde_json::Value) -> Vec<Attachment> {
    message.get("files")
        .and_then(|f| f.as_array())
        .map(|files| files.iter().filter_map(|f| {
            Some(Attachment {
                id: f.get("id")?.as_i64()?,
                name: f.get("original_name").and_then(|v| v.as_str()).unwrap_or("file").to_string(),
                extension: f.get("extension").and_then(|v| v.as_str()).map(str::to_string),
                mimetype: f.get("mimetype").and_then(|v| v.as_str()).map(str::to_string),
            })
        }).collect())
        .unwrap_or_default()
}

/// Where attachments go unless told otherwise: ~/Downloads/Shkolo
pub fn default_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|u| u.download_dir().map(Path::to_path_buf))
        .or_else(|| directories::BaseDirs::new().map(|b| b.home_dir().join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Shkolo")
}

/// A file name that stays inside the target folder
fn safe_name(name: &str) -> String {
    let cleaned: String = name.chars()
        .map(|c| if matches!(c, '/' | '\\' | ':' | '\0') || c.is_control() { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_start_matches('.').to_string();
    if trimmed.is_empty() { "file".to_string() } else { trimmed }
}

fn extension_of(name: &str) -> Option<String> {
    Path::new(name).extension().map(|e| e.to_string_lossy().to_lowercase())
}

/// Unpack a message's zip into `dir`, naming each entry after its attachment. With `only`, just that
/// attachment is written. Existing files of the same name are replaced (same message, same content).
pub fn save_zip(zip_bytes: &[u8], files: &[Attachment], only: Option<i64>, dir: &Path) -> Result<Vec<Saved>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .map_err(|e| anyhow!("Shkolo didn't send a zip: {}", e))?;
    let entries: Vec<usize> = (0..archive.len())
        .filter(|&i| archive.by_index(i).map(|e| e.is_file()).unwrap_or(false))
        .collect();

    // Entry -> attachment: by position when the counts agree, else the next unused one with the
    // same extension
    let mut used = vec![false; files.len()];
    let mut matched: Vec<Option<usize>> = Vec::with_capacity(entries.len());
    for (pos, &i) in entries.iter().enumerate() {
        let entry_ext = extension_of(archive.by_index(i)?.name());
        let pick = if entries.len() == files.len() {
            Some(pos)
        } else {
            (0..files.len()).find(|&j| {
                !used[j] && files[j].extension.as_deref().map(str::to_lowercase) == entry_ext
            })
        };
        if let Some(j) = pick {
            used[j] = true;
        }
        matched.push(pick);
    }

    std::fs::create_dir_all(dir)?;
    let mut saved = Vec::new();
    for (&i, pick) in entries.iter().zip(matched) {
        let file = pick.map(|j| &files[j]);
        if only.is_some() && file.map(|f| f.id) != only {
            continue;
        }
        let mut entry = archive.by_index(i)?;
        let zip_name = Path::new(entry.name()).file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        let name = safe_name(file.map(|f| f.name.as_str()).unwrap_or(&zip_name));
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let path = dir.join(&name);
        std::fs::write(&path, &data)?;
        saved.push(Saved {
            id: file.map(|f| f.id),
            name,
            path,
            size: data.len() as u64,
            mimetype: file.and_then(|f| f.mimetype.clone()),
        });
    }
    if let Some(id) = only {
        if saved.is_empty() {
            return Err(anyhow!("attachment {} isn't in the message's files", id));
        }
    }
    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            for (name, data) in entries {
                z.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
                z.write_all(data).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn att(id: i64, name: &str, ext: &str) -> Attachment {
        Attachment { id, name: name.into(), extension: Some(ext.into()), mimetype: None }
    }

    #[test]
    fn entries_get_their_real_names_in_order() {
        let dir = std::env::temp_dir().join(format!("shkolo-dl-{}", std::process::id()));
        let zip = zip_of(&[("Prilozenie_1.docx", b"one"), ("Snimka.jpg", b"two")]);
        let files = [att(1, "Приложение_№_1.docx", "docx"), att(2, "Снимка.jpg", "jpg")];
        let saved = save_zip(&zip, &files, None, &dir).unwrap();
        assert_eq!(saved.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Приложение_№_1.docx", "Снимка.jpg"]);
        assert_eq!(std::fs::read(dir.join("Снимка.jpg")).unwrap(), b"two");

        let only = save_zip(&zip, &files, Some(2), &dir).unwrap();
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].id, Some(2));
        assert!(save_zip(&zip, &files, Some(9), &dir).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn unsafe_names_stay_in_the_folder() {
        assert_eq!(safe_name("../../etc/passwd"), "_.._etc_passwd");
        assert_eq!(safe_name(""), "file");
    }
}
