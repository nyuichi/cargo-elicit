#[cfg(unix)]
#[test]
fn timeout_stops_descendants() {
    let start = std::time::Instant::now();
    let output = cargo_elicit::process::run(
        std::process::Command::new("sh").args([
            "-c",
            if cfg!(target_os = "linux") {
                "setsid sh -c 'sleep 60' & wait"
            } else {
                "sleep 60 & wait"
            },
        ]),
        std::time::Duration::from_millis(150),
    )
    .unwrap();
    assert!(output.timeout);
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
}
