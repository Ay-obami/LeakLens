import test from 'node:test';
import assert from 'node:assert/strict';
import { reviewWithApi } from './api.ts';
import { makeProposal } from './policy.ts';

test('configured API outage produces explicitly tagged backend-unavailable simulation', async () => {
  const review = await reviewWithApi(makeProposal('transparent'), 'https://api.example', async () => { throw new Error('offline'); });
  assert.equal(review.source, 'backend_unavailable');
  assert.ok(review.assessment.findings.some(f => f.code === 'LL-002'));
});

test('unconfigured API is labeled local simulation', async () => {
  const review = await reviewWithApi(makeProposal('shielded'), '', async () => { throw new Error('should not call'); });
  assert.equal(review.source, 'local_simulation');
});

test('successful API response is labeled Rust API', async () => {
  const result = { complete: true, policy_version: 1, findings: [] };
  const review = await reviewWithApi(makeProposal('shielded'), 'https://api.example', async () => new Response(JSON.stringify(result), { status: 200 }));
  assert.equal(review.source, 'rust_api');
});
