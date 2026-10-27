import test from 'node:test';
import assert from 'node:assert/strict';
import { chromium } from 'playwright-core';

const base=process.env.BASE_URL||'http://127.0.0.1:5176';
async function browserPage(run){
 const browser=await chromium.launch({headless:true,executablePath:'/usr/bin/google-chrome',args:['--no-sandbox','--disable-dev-shm-usage']});
 try{const page=await browser.newPage({viewport:{width:1365,height:900}});await run(page);}
 finally{await browser.close();}
}
test('landing has real launch action leading to wallet workspace',async()=>{
 await browserPage(async page=>{
  await page.goto(base);
  assert.match(await page.locator('h1').first().innerText(),/Your payment/);
  await page.getByRole('link',{name:/Open LeakLens/i}).click();
  assert.equal(new URL(page.url()).pathname,'/app');
  assert.match(await page.locator('main').innerText(),/WALLET BALANCE|Wallet balance/i);
  assert.equal(await page.getByText('Connect wallet',{exact:true}).count(),1);
 });
});
test('guest can review a real planned address without any wallet and no fake send',async()=>{
 await browserPage(async page=>{
  await page.goto(base+'/app');
  await page.getByRole('button',{name:/Send payment/}).first().click();
  await page.locator('#recipient').fill('tm'+'x'.repeat(33));
  await page.locator('#amount').fill('0.01');
  await page.getByRole('button',{name:/Review privacy/}).click();
  assert.match(await page.locator('main').innerText(),/Public exposure detected/);
  assert.match(await page.locator('main').innerText(),/preflight estimate/i);
  assert.equal(await page.getByRole('button',{name:/Connect wallet to send/}).count(),1);
  await page.getByRole('button',{name:/Connect wallet to send/}).click();
  assert.match(await page.locator('[role=alert]').innerText(),/Noir Wallet is not installed/);
  assert.equal(await page.getByText('Transaction submitted').count(),0);
 });
});
test('mobile navigation exposes receive and activity screens',async()=>{
 await browserPage(async page=>{
  await page.setViewportSize({width:390,height:844});
  await page.goto(base+'/app');
  await page.locator('.bottom-nav [data-page=receive]').click();
  assert.match(await page.locator('main').innerText(),/Receive on your terms/);
  await page.locator('.bottom-nav [data-page=activity]').click();
  assert.match(await page.locator('main').innerText(),/Every move/);
  assert.equal(await page.locator('.sidebar').isVisible(),false);
 });
});
