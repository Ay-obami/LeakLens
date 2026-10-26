# LeakLens Privacy Model and Limitations

## What the firewall can observe

The policy evaluates facts made available by a wallet's proposed transaction: selected recipient value pool, transparent inputs, whether shielded inputs are present, change-output pools, and proven provenance between proposal steps when the adapter supplies it.

These observations support **two distinct classes of statement**:

1. **Public disclosure:** For example, an intended transparent output is visible on-chain, as are transparent input histories. These are direct properties of the transaction representation.
2. **Possible correlation:** An observer may compare amounts or timing across shielded-pool boundaries. Such a pattern is not proof that particular shielded transactions share an owner, sender, or recipient.

An incomplete/unsupported proposal is a third class, **unknown**, and must not be silently scored safe.

## Threats covered by the current rule set

- A user or app selecting a transparent receiver while expecting shielded delivery.
- Transparent source/input disclosure and transparent change.
- Potential linkability patterns associated with shielded-to-transparent payment paths.
- Missed exposures hidden in later proposal steps.
- Incomplete policy evaluation and stale proposal approval.

## Explicit non-guarantees

- LeakLens does **not** break Zcash shielded encryption, reveal hidden sender/receiver/amount, or deanonymize users.
- A clean report is not an anonymity guarantee, not a probability score, and not a substitute for a wallet security audit.
- It does not eliminate network/IP metadata exposure, exchange KYC, merchant records, address sharing, browser telemetry, memo disclosure outside the inspected path, or external device compromise.
- It cannot force other wallets to honor the gate. Protection requires integrating the gate into the actual authorization path.
- It does not replace Zcash consensus checks, transaction balancing, signature verification, proof construction, or chain confirmation.
- A test against a structured SDK proposal is not evidence of a funded wallet, completed signing, successful network broadcast, or production-readiness.

## Handling sensitive data

The policy engine only needs normalized transaction facts; do not pass seed phrases, spending keys, viewing keys, unredacted wallet backups, or external identity profiles. Keep fixture data synthetic/non-sensitive. Avoid putting payment data in analytics, third-party logs, URLs, or screenshots that are published without review.

## Failure behavior

Unsupported pool kinds, contradictory recipient/pool combinations, or unknown multi-step provenance must produce a detectable incomplete result; within the controlled gate, incomplete assessment denies authorization. Reassess if any proposal-relevant contents or policy version change. Known warnings may require explicit acknowledgment, but incomplete analysis is not an ordinary warning that can be clicked through.
