use std::io::Cursor;

use leaklens_demo::{run_sdk_fixture_demo, DemoStatus};

#[test]
fn real_sdk_fixture_passes_through_adapter_review_and_noop_gate() {
    let mut output = Vec::new();
    let result = run_sdk_fixture_demo(Cursor::new(b"CONTINUE\n"), &mut output).unwrap();
    let display = String::from_utf8(output).unwrap();
    assert_eq!(result, DemoStatus::NoopReached);
    assert!(display.contains("SDK-validated proposal"));
    assert!(display.contains("NOT SIGNING"));
    assert!(display.contains("full SDK protobuf fingerprint"));
}

#[test]
fn aborted_sdk_fixture_never_reaches_noop_callback() {
    let mut output = Vec::new();
    let result = run_sdk_fixture_demo(Cursor::new(Vec::<u8>::new()), &mut output).unwrap();
    assert_eq!(result, DemoStatus::Cancelled);
    assert!(!String::from_utf8(output).unwrap().contains("NO-OP CALLBACK REACHED"));
}

#[test]
fn no_fixture_signing_or_broadcast_claim_appears() {
    let mut output = Vec::new();
    run_sdk_fixture_demo(Cursor::new(b"CONTINUE\n"), &mut output).unwrap();
    let text = String::from_utf8(output).unwrap();
    assert!(!text.contains("Transaction signed successfully"));
    assert!(!text.contains("Broadcast successful"));
}
