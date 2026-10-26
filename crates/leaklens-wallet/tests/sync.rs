use leaklens_wallet::sync::SyncConfig;

#[test]
fn sync_rejects_zero_scan_budget_before_network_request() {
    let error = SyncConfig::new("https://testnet.zec.rocks:443", 0)
        .expect_err("zero block budget must be refused");
    assert!(error.to_string().contains("nonzero"));
}

#[test]
fn sync_refuses_plaintext_lightwalletd_endpoint() {
    let error = SyncConfig::new("http://testnet.zec.rocks:9067", 10)
        .expect_err("unencrypted remote lightwalletd is unsafe");
    assert!(error.to_string().contains("https"));
}

#[test]
fn sync_accepts_bounded_tls_testnet_configuration() {
    let config =
        SyncConfig::new("https://testnet.zec.rocks:443", 100).expect("valid bounded config");
    assert_eq!(config.max_blocks(), 100);
}
