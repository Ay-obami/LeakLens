use leaklens_zcash::{encode_canonical_standard_proposal, fixture::sample_shielded_proposal};
use prost::Message;
use zcash_client_backend::proto::proposal::Proposal as ProtoProposal;

#[test]
fn real_sdk_proposal_has_deterministic_complete_protobuf_encoding() {
    let sdk_proposal = sample_shielded_proposal();
    let first = encode_canonical_standard_proposal(&sdk_proposal);
    let second = encode_canonical_standard_proposal(&sdk_proposal);
    assert!(!first.is_empty());
    assert_eq!(first, second);
    let parsed = ProtoProposal::decode(first.as_slice()).expect("valid SDK proposal protobuf");
    assert_eq!(parsed.steps.len(), 1);
    assert_eq!(parsed.steps[0].payment_output_pools.len(), 1);
    assert!(parsed.steps[0].transaction_request.contains("zcash:"));
}
