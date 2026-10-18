use anyhow::{Context, Result};
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);
pub fn install_interrupt() -> Result<()> {
    ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst))?;
    Ok(())
}
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}
#[derive(Debug)]
pub struct Output {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timeout: bool,
    pub interrupted: bool,
    pub elapsed_ms: u64,
}
#[cfg(unix)]
pub fn group(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0);
}
#[cfg(not(unix))]
pub fn group(_: &mut Command) {}
#[cfg(unix)]
pub fn kill_group(id: u32) {
    // Compiler subprocesses may create their own process groups.
    // Collect descendants while the parent is still alive, then stop all of them.
    #[cfg(target_os = "linux")]
    let descendants = {
        let mut pairs = vec![];
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
                    continue;
                };
                if let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) {
                    if let Some((_, rest)) = stat.rsplit_once(") ") {
                        if let Some(parent) = rest
                            .split_whitespace()
                            .nth(1)
                            .and_then(|v| v.parse::<u32>().ok())
                        {
                            pairs.push((pid, parent));
                        }
                    }
                }
            }
        }
        let mut found = vec![id];
        let mut i = 0;
        while i < found.len() {
            let parent = found[i];
            for (pid, ppid) in &pairs {
                if *ppid == parent && !found.contains(pid) {
                    found.push(*pid);
                }
            }
            i += 1;
        }
        found
    };
    unsafe {
        libc::kill(-(id as i32), libc::SIGKILL);
    }
    #[cfg(target_os = "linux")]
    for pid in descendants.into_iter().rev() {
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }
}
#[cfg(not(unix))]
pub fn kill_group(_: u32) {}
pub fn run(cmd: &mut Command, timeout: Duration) -> Result<Output> {
    group(cmd);
    let start = Instant::now();
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("cannot execute {:?}", cmd.get_program()))?;
    let id = child.id();
    let done = Arc::new(AtomicBool::new(false));
    fn drain(
        mut r: impl Read + Send + 'static,
        done: Arc<AtomicBool>,
    ) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut out = vec![];
            let mut b = [0; 8192];
            loop {
                match r.read(&mut b) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if out.len() < 8 * 1024 * 1024 {
                            out.extend_from_slice(&b[..n]);
                        }
                    }
                }
            }
            done.store(true, Ordering::SeqCst);
            out
        })
    }
    let a = drain(child.stdout.take().unwrap(), done.clone());
    let errdone = Arc::new(AtomicBool::new(false));
    let b = drain(child.stderr.take().unwrap(), errdone.clone());
    let mut timed = false;
    let mut stop = false;
    let code = loop {
        if interrupted() || start.elapsed() >= timeout {
            timed = !interrupted();
            stop = interrupted();
            kill_group(id);
            let _ = child.kill();
            break child.wait()?.code();
        }
        if let Some(s) = child.try_wait()? {
            break s.code();
        }
        thread::sleep(Duration::from_millis(20));
    };
    // Also terminate descendants holding pipe handles after the direct child exits.
    kill_group(id);
    let stdout = String::from_utf8_lossy(&a.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&b.join().unwrap_or_default()).into_owned();
    Ok(Output {
        code,
        stdout,
        stderr,
        timeout: timed,
        interrupted: stop,
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}
