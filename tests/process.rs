#[cfg(unix)]
#[test]
fn timeout_stops_descendants_in_separate_process_groups() {
    let start = std::time::Instant::now();
    let output = cargo_elicit::process::run(
        std::process::Command::new("sh").args(["-c", "setsid sh -c 'sleep 60' & wait"]),
        std::time::Duration::from_millis(150),
    )
    .unwrap();
    assert!(output.timeout);
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
}
