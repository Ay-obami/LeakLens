use leaklens_wallet::storage::open_testnet_wallet;

#[test]
fn testnet_wallet_storage_is_persistent_and_reopenable() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("wallet.sqlite");
    assert!(!path.exists());

    let first = open_testnet_wallet(&path).expect("initialize testnet wallet DB");
    assert!(
        path.exists(),
        "SQLite backend should create a real database"
    );
    drop(first);

    let reopened = open_testnet_wallet(&path).expect("reopening must be idempotent");
    drop(reopened);
}

#[test]
fn opening_under_missing_parent_reports_error_without_creating_directories() {
    let temporary = tempfile::tempdir().unwrap();
    let nonexistent_parent = temporary.path().join("never-created");
    let path = nonexistent_parent.join("wallet.sqlite");

    let failure = open_testnet_wallet(&path)
        .err()
        .expect("missing parent should fail");
    assert!(failure.to_string().contains("opening testnet wallet DB"));
    assert!(!nonexistent_parent.exists());
}
