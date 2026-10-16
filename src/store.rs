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
        bail!("elicit.toml not found; run cargo elicit init --tool creusot (or kani)")
    }
    pub fn lock(&self, name: &str) -> Result<Lock> {
        fs::create_dir_all(self.root.join(".elicit/runtime"))?;
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(format!(".elicit/runtime/{name}.lock")))?;
        f.try_lock_exclusive()
            .with_context(|| format!("{name} is locked by another cargo-elicit process"))?;
        Ok(Lock(f))
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
        let _lock = self.lock("state")?;
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
