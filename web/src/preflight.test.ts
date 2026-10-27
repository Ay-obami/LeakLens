import test from 'node:test';
import assert from 'node:assert/strict';
import { analyzePreflight, parsePaymentDraft, samePayment } from './preflight.ts';

test('transparent testnet recipient yields public disclosure warning', () => {
  const report = analyzePreflight({to:'tm'+'x'.repeat(33),amount:'0.01',fundingSource:'shielded',memo:''});
  assert.equal(report.risk,'warning');
  assert.match(report.items.join(' '),/public/i);
});
test('unified testnet address cannot claim zero risk', () => {
  const report = analyzePreflight({to:'utest1'+'q'.repeat(100),amount:'0.01',fundingSource:'shielded',memo:''});
  assert.notEqual(report.risk,'clear');
  assert.equal(report.exactProposalVerified,false);
});
test('mainnet addresses and malformed amounts are invalid', () => {
  assert.equal(parsePaymentDraft({to:'u1abc',amount:'0.01',fundingSource:'shielded',memo:''}).ok,false);
  assert.equal(parsePaymentDraft({to:'tm'+'x'.repeat(33),amount:'-1',fundingSource:'shielded',memo:''}).ok,false);
  assert.equal(parsePaymentDraft({to:'tm'+'x'.repeat(33),amount:'0.123456789',fundingSource:'shielded',memo:''}).ok,false);
});
test('changing payment after review invalidates consent', () => {
  const a={to:'tm'+'x'.repeat(33),amount:'0.01',fundingSource:'shielded' as const,memo:''};
  assert.equal(samePayment(a,{...a,amount:'0.02'}),false);
  assert.equal(samePayment(a,{...a}),true);
});
