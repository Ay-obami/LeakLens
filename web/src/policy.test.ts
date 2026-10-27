import test from 'node:test';
import assert from 'node:assert/strict';
import { assessLocally, makeProposal } from './policy.ts';

test('shielded fixture has no findings and is complete', () => {
  const result = assessLocally(makeProposal('shielded'));
  assert.equal(result.complete, true);
  assert.equal(result.findings.length, 0);
});

test('transparent fixture reports public input, public output', () => {
  const result = assessLocally(makeProposal('transparent'));
  assert.deepEqual(result.findings.map(x => x.code), ['LL-002', 'LL-001']);
});

test('unified recipient selecting transparent receiver is not assumed private', () => {
  const proposal = makeProposal('unified');
  const result = assessLocally(proposal);
  assert.deepEqual(result.findings.map(x => x.code), ['LL-001', 'LL-004']);
});

test('shielded funding to transparent recipient reports correlation, not proven linkage', () => {
  const result = assessLocally(makeProposal('deshield'));
  assert.deepEqual(result.findings.map(x => x.code), ['LL-001', 'LL-003']);
  assert.equal(result.findings[1].certainty, 'possible_correlation');
});

test('unknown pool fails closed', () => {
  const proposal = makeProposal('shielded');
  proposal.steps[0].payments[0].selected_pool = 'unknown';
  const result = assessLocally(proposal);
  assert.equal(result.complete, false);
  assert.equal(result.findings[0].code, 'LL-005');
});
