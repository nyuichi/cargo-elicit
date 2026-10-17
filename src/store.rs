use crate::model::{Config, State};
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
        bail!("elicit.toml not found; run cargo elicit init --tool creusot (or kani / aeneas)")
    }
    pub fn lock(&self, name: &str) -> Result<Lock> {
        self.lock_for(name, std::time::Duration::ZERO)
    }
    pub(crate) fn state_lock(&self) -> Result<Lock> {
        self.lock_for("state", std::time::Duration::from_secs(120))
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
    pub fn config(&self) -> Result<Config> {
        let c: Config = toml::from_str(&fs::read_to_string(self.root.join("elicit.toml"))?)?;
        c.validate()?;
        Ok(c)
    }
    pub fn load(&self) -> Result<State> {
        let s: State = toml::from_str(
            &fs::read_to_string(self.root.join(".elicit/state.toml"))
                .context("missing ledger; run cargo elicit init")?,
        )?;
        if s.schema_version != 1 {
            bail!(
                "unsupported ledger schema {}; state was preserved",
                s.schema_version
            );
        }
        if s.tool != self.config()?.verification.tool {
            bail!("changing verifier is unsupported; existing ledger was preserved");
        }
        Ok(s)
    }
    pub fn save(&self, state: &mut State) -> Result<()> {
        state.revision += 1;
        atomic_write(
            &self.root.join(".elicit/state.toml"),
            toml::to_string_pretty(state)?.as_bytes(),
        )
    }
    pub fn transaction<T>(
        &self,
        revision: Option<u64>,
        f: impl FnOnce(&mut State) -> Result<T>,
    ) -> Result<T> {
        let _lock = self.state_lock()?;
        let mut s = self.load()?;
        if revision.is_some_and(|r| r != s.revision) {
            bail!("revision conflict: current revision is {}", s.revision);
        }
        let out = f(&mut s)?;
        self.save(&mut s)?;
        Ok(out)
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
