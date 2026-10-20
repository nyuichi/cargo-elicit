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
    pub fn prepare_gitignore(&self) -> Result<()> {
        let path = self.root.join(".elicit/.gitignore");
        if !path.exists() {
            atomic_write(
                &path,
                b"/*\n!/.gitignore\n!/evidence/\n/evidence/*\n!/evidence/*.json\n",
            )?;
        }
        Ok(())
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
    /// Compact milestones, separate from high-volume native pipeline telemetry.
    pub fn history(&self, event: &str, data: serde_json::Value) -> Result<()> {
        let path = self.root.join(".elicit/history.jsonl");
        fs::create_dir_all(path.parent().unwrap())?;
        let mut line = serde_json::to_vec(
            &serde_json::json!({"schema":1,"recorded_at":crate::project::now(),"event":event,"data":data}),
        )?;
        line.push(b'\n');
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?
            .write_all(&line)?;
        Ok(())
    }
    pub fn recent_history(&self) -> Result<Vec<serde_json::Value>> {
        use std::io::{Read, Seek, SeekFrom};
        let path = self.root.join(".elicit/history.jsonl");
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let len = file.metadata()?.len();
        let start = len.saturating_sub(65536);
        file.seek(SeekFrom::Start(start))?;
        let mut bytes = Vec::new();
        file.take(65536).read_to_end(&mut bytes)?;
        let text = String::from_utf8_lossy(&bytes);
        let mut lines = text.lines();
        if start > 0 {
            lines.next();
        }
        let mut records: Vec<_> = lines.filter_map(|l| serde_json::from_str(l).ok()).collect();
        records.drain(..records.len().saturating_sub(8));
        Ok(records)
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
