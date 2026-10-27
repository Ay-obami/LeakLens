export type PoolKind = 'transparent' | 'sapling' | 'orchard' | 'ironwood' | 'unknown';
export type RecipientKind = 'transparent' | 'shielded' | 'unified' | 'unknown';
export type Scenario = 'shielded' | 'transparent' | 'unified' | 'deshield';
export type Proposal = { steps: Array<{
  payments: Array<{ recipient_kind: RecipientKind; selected_pool: PoolKind; amount_zat: number }>;
  has_transparent_inputs: boolean;
  has_shielded_inputs: boolean;
  has_prior_shielded_funding: boolean;
  change_pools: PoolKind[];
}> };
export type Finding = {
  code: string; step_index: number; payment_index: number | null;
  severity: 'warning' | 'caution' | 'incomplete';
  certainty: 'public_disclosure' | 'possible_correlation' | 'unknown';
  message: string;
};
export type Assessment = { findings: Finding[]; complete: boolean; policy_version: number };

/** Public, synthetic, non-spendable scenarios. Not SDK-validated transaction proposals. */
export function makeProposal(scenario: Scenario): Proposal {
  return { steps: [{
    payments: [{
      recipient_kind: scenario === 'unified' ? 'unified' : scenario === 'shielded' ? 'shielded' : 'transparent',
      selected_pool: scenario === 'shielded' ? 'orchard' : 'transparent',
      amount_zat: 1_000_000,
    }],
    has_transparent_inputs: scenario === 'transparent',
    has_shielded_inputs: scenario === 'shielded' || scenario === 'deshield',
    has_prior_shielded_funding: false,
    change_pools: [],
  }] };
}

/** Browser-only educational mirror of the Rust v1 policy. No authorization decisions are made here. */
export function assessLocally(proposal: Proposal): Assessment {
  const findings: Finding[] = [];
  let complete = proposal.steps.length > 0;
  const add = (code: string, step_index: number, payment_index: number | null, severity: Finding['severity'], certainty: Finding['certainty'], message: string) => {
    findings.push({ code, step_index, payment_index, severity, certainty, message });
    if (severity === 'incomplete') complete = false;
  };
  if (!proposal.steps.length) add('LL-005', 0, null, 'incomplete', 'unknown', 'Transaction proposal contains no steps');
  proposal.steps.forEach((step, stepIndex) => {
    if (!step.payments.length && !step.has_transparent_inputs && !step.has_shielded_inputs && !step.has_prior_shielded_funding && !step.change_pools.length) {
      add('LL-005', stepIndex, null, 'incomplete', 'unknown', 'Transaction step has no payment, funding input, or change to assess');
    }
    if (step.has_transparent_inputs) {
      add('LL-002', stepIndex, null, 'warning', 'public_disclosure', 'Transparent funding inputs and their public history are observable');
    }
    step.payments.forEach((payment, paymentIndex) => {
      if (payment.selected_pool === 'unknown' || payment.recipient_kind === 'unknown') {
        add('LL-005', stepIndex, paymentIndex, 'incomplete', 'unknown', 'Recipient or selected payment pool cannot be assessed');
        return;
      }
      if ((payment.recipient_kind === 'transparent' && payment.selected_pool !== 'transparent') ||
          (payment.recipient_kind === 'shielded' && payment.selected_pool === 'transparent')) {
        add('LL-005', stepIndex, paymentIndex, 'incomplete', 'unknown', 'Recipient kind conflicts with the selected output pool');
        return;
      }
      if (payment.selected_pool === 'transparent') {
        add('LL-001', stepIndex, paymentIndex, 'warning', 'public_disclosure', 'The transparent recipient and payment amount will be publicly visible');
        if (step.has_shielded_inputs || step.has_prior_shielded_funding) {
          add('LL-003', stepIndex, paymentIndex, 'caution', 'possible_correlation', 'Deshielding may expose amount or timing patterns; it does not prove an identity link');
        }
        if (payment.recipient_kind === 'unified') {
          add('LL-004', stepIndex, paymentIndex, 'warning', 'public_disclosure', 'The Unified Address is being paid through its transparent receiver');
        }
      }
    });
    if (step.change_pools.includes('transparent')) {
      add('LL-007', stepIndex, null, 'warning', 'public_disclosure', 'Transparent change output and its value will be publicly visible');
      if (step.has_shielded_inputs || step.has_prior_shielded_funding) {
        add('LL-003', stepIndex, null, 'caution', 'possible_correlation', 'Shielded funding alongside transparent change may create public correlation signals');
      }
    }
    if (step.change_pools.includes('unknown')) {
      add('LL-005', stepIndex, null, 'incomplete', 'unknown', 'A change output pool cannot be assessed');
    }
  });
  return { findings, complete, policy_version: 1 };
}
