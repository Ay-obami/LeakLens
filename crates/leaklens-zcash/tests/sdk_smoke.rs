use leaklens_zcash::fixture::sample_shielded_proposal;

#[test]
fn sdk_generates_real_shielded_proposal() {
    let proposal = sample_shielded_proposal();
    assert_eq!(proposal.steps().len(), 1);
    assert_eq!(proposal.steps().first().payment_pools().len(), 1);
}
