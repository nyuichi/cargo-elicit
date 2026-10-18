use anyhow::{Context, Result, bail};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Store {
    pub root: PathBuf,
}
pub struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}
impl Store {
    pub fn discover(from: &Path) -> Result<Self> {
        for p in from.ancestors() {
            if p.join("elicit.toml").is_file() {
                return Ok(Self {
                    root: p.to_path_buf(),
                });
            }
        }
        bail!("elicit.toml not found; run cargo elicit init")
    }
    pub fn lock(&self, name: &str) -> Result<Lock> {
        self.lock_for(name, std::time::Duration::ZERO)
    }
    fn lock_for(&self, name: &str, wait: std::time::Duration) -> Result<Lock> {
        fs::create_dir_all(self.root.join(".elicit/runtime"))?;
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(format!(".elicit/runtime/{name}.lock")))?;
        let started = std::time::Instant::now();
        loop {
            match f.try_lock_exclusive() {
                Ok(()) => return Ok(Lock(f)),
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && started.elapsed() < wait =>
                {
                    if crate::process::interrupted() {
                        bail!("interrupted while waiting for {name} lock");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("{name} is locked by another cargo-elicit process")
                    });
                }
            }
        }
    }
    pub fn event(&self, event: &str, data: serde_json::Value) -> Result<()> {
        let path = self.root.join(".elicit/events.jsonl");
        fs::create_dir_all(path.parent().unwrap())?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();
        let mut line = serde_json::to_vec(
            &serde_json::json!({"timestamp_ms":timestamp,"event":event,"data":data}),
        )?;
        line.push(b'\n');
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?
            .write_all(&line)?;
        Ok(())
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path.parent().context("missing parent")?;
    fs::create_dir_all(dir)?;
    let mut temp = tempfile::NamedTempFile::new_in(dir)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    File::open(dir)?.sync_all()?;
    Ok(())
}
