//! Runs the explicitly non-signing SDK fixture demonstration.

use std::io;

fn main() -> io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let fixture = match (arguments.next().as_deref(), arguments.next()) {
        (None, None) | (Some("--shielded"), None) => leaklens_demo::DemoFixture::Shielded,
        (Some("--transparent"), None) => leaklens_demo::DemoFixture::Transparent,
        _ => {
            eprintln!("Usage: leaklens-demo [--shielded|--transparent]");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unknown demo option",
            ));
        }
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let result = leaklens_demo::run_sdk_fixture_demo_mode(stdin.lock(), &mut out, fixture)?;
    if result == leaklens_demo::DemoStatus::Cancelled {
        use std::io::Write;
        writeln!(out, "Review cancelled. No wallet action was performed.")?;
    }
    Ok(())
}
