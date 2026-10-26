//! Runs the explicitly non-signing SDK fixture demonstration.

use std::io;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let result = leaklens_demo::run_sdk_fixture_demo(stdin.lock(), &mut out)?;
    if result == leaklens_demo::DemoStatus::Cancelled {
        use std::io::Write;
        writeln!(out, "Review cancelled. No wallet action was performed.")?;
    }
    Ok(())
}
