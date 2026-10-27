import test from 'node:test';
import assert from 'node:assert/strict';
import { isTestnetAddress, formatBalance } from './wallet.ts';

test('connection rejects mainnet addresses',()=>{
 assert.equal(isTestnetAddress('u1'+'q'.repeat(60)),false);
 assert.equal(isTestnetAddress('utest1'+'q'.repeat(60)),true);
});
test('unknown balances never displayed as zero',()=>{
 assert.equal(formatBalance(undefined),'—');
 assert.equal(formatBalance('0'),'0');
});
