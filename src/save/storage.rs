use super::snapshot::Snapshot;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Remembers exactly what was read to avoid overwriting another running game's save.
#[derive(Debug)]
pub(super) struct Store {
    pub path: PathBuf,
    expected: Option<Vec<u8>>,
    readable: bool,
    valid: bool,
}
impl Store {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            expected: None,
            readable: false,
            valid: false,
        }
    }
    fn bytes(&self) -> Result<Option<Vec<u8>>, String> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("Cannot read save: {error}")),
        };
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Save exceeds the 16 MiB limit. Move it aside before retrying.".into());
        }
        Ok(Some(bytes))
    }
    pub fn read(&mut self) -> Result<Option<Snapshot>, String> {
        self.readable = false;
        self.valid = false;
        self.expected = self.bytes()?;
        self.readable = true;
        let snapshot = self.expected.as_deref().map(Snapshot::decode).transpose()?;
        self.valid = true;
        Ok(snapshot)
    }
    pub fn write(&mut self, snapshot: &Snapshot, replace: bool) -> Result<(), String> {
        if !self.readable {
            return Err("Save could not be read. Resolve the file error and retry first.".into());
        }
        if !self.valid && !replace {
            return Err("Existing save is invalid; explicit replacement is required.".into());
        }
        let bytes = serde_json::to_vec_pretty(snapshot).map_err(|e| e.to_string())?;
        Snapshot::decode(&bytes)?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err("Campaign exceeds the 16 MiB save limit.".into());
        }
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(std::path::Path::new("."));
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create save directory: {e}"))?;
        // The stable lock file is intentionally retained; the OS releases its lock on exit/crash.
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.with_extension("lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock().map_err(|e| format!("Save is busy: {e}"))?;
        if self.bytes()? != self.expected {
            return Err("Save changed in another app. Quit and reopen to load it; this campaign has not overwritten it.".into());
        }
        if let Some(old) = &self.expected {
            if replace {
                let mut archive = tempfile::Builder::new()
                    .prefix("campaign-replaced-")
                    .suffix(".json")
                    .tempfile_in(parent)
                    .map_err(|e| e.to_string())?;
                archive
                    .write_all(old)
                    .and_then(|()| archive.as_file().sync_all())
                    .map_err(|e| e.to_string())?;
                archive.keep().map_err(|e| e.to_string())?;
            } else {
                let mut backup =
                    tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
                backup
                    .write_all(old)
                    .and_then(|()| backup.as_file().sync_all())
                    .map_err(|e| e.to_string())?;
                backup
                    .persist(self.path.with_extension("previous.json"))
                    .map_err(|e| format!("Cannot preserve previous save: {e}"))?;
            }
        }
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temp.write_all(&bytes)
            .and_then(|()| temp.as_file().sync_all())
            .map_err(|e| format!("Cannot write save: {e}"))?;
        temp.persist(&self.path)
            .map_err(|e| format!("Cannot replace save: {e}"))?;
        // Track the committed bytes even if directory sync fails; retry is then safe.
        self.expected = Some(bytes);
        self.valid = true;
        #[cfg(unix)]
        File::open(parent)
            .and_then(|file| file.sync_all())
            .map_err(|e| format!("Cannot sync save directory: {e}"))?;
        Ok(())
    }
}

pub(super) fn default_path() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("DRONE_SAVE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or("HOME is unavailable. Set DRONE_SAVE_PATH to choose a save file.")?;
    #[cfg(target_os = "macos")]
    let folder = PathBuf::from(home).join("Library/Application Support/Drone Survivors");
    #[cfg(not(target_os = "macos"))]
    let folder = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".local/share"))
        .join("drone-survivors");
    Ok(folder.join("campaign.json"))
}
