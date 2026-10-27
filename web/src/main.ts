import '@fontsource/dm-sans/400.css';
import '@fontsource/dm-sans/500.css';
import '@fontsource/dm-sans/600.css';
import '@fontsource/dm-sans/700.css';
import '@fontsource/dm-sans/800.css';
import '@fontsource/manrope/500.css';
import '@fontsource/manrope/700.css';
import '@fontsource/manrope/800.css';
import './style.css';
import { noirWallet, formatBalance, type WalletData } from './wallet.ts';
import { analyzePreflight, parsePaymentDraft, samePayment, type Payment, type Preflight } from './preflight.ts';

const github='https://github.com/Ay-obami/LeakLens';
const noirReleases='https://github.com/NoirWallet/noir-wallet/releases';
const root=document.querySelector<HTMLDivElement>('#app');
if(!root)throw new Error('Missing #app');
type Page='overview'|'send'|'receive'|'activity';
let page:Page='overview';
let wallet:WalletData|null=null;
let busy=false;
let error='';
let notice='';
let sendStage:'edit'|'review'|'pending'|'sent'='edit';
let draft:Payment={to:'',amount:'',memo:'',fundingSource:'shielded'};
let reviewed:Payment|null=null;
let analysis:Preflight|null=null;
let lastTxid='';
const esc=(v:unknown)=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
const short=(value:string,n=12)=>value.length>n*2?value.slice(0,n)+'…'+value.slice(-n):value;
const icon=(name:string,size=20)=>{const paths:Record<string,string>={
 eye:'<circle cx="12" cy="12" r="3"/><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"/>',
 arrow:'<path d="M4 12h16m-7-7 7 7-7 7"/>',
 shield:'<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10Z"/><path d="m9 12 2 2 4-4"/>',
 lock:'<rect x="5" y="10" width="14" height="11" rx="2"/><path d="M8 10V7a4 4 0 0 1 8 0v3"/>',
 check:'<path d="m5 12 4 4L19 6"/>',
 wallet:'<rect x="3" y="5" width="18" height="15" rx="3"/><path d="M16 12h5M3 9h18"/>',
 copy:'<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
 bolt:'<path d="m13 2-9 11h7l-1 9 10-12h-7l0-8Z"/>',
 chevron:'<path d="m9 18 6-6-6-6"/>',
 activity:'<path d="M3 12h4l3-7 4 14 3-7h4"/>',
 info:'<circle cx="12" cy="12" r="10"/><path d="M12 11v6m0-10h.01"/>',
 home:'<rect x="3" y="10" width="18" height="11" rx="2"/><path d="m2 11 10-9 10 9"/>',
 inbox:'<rect x="3" y="5" width="18" height="15" rx="2"/><path d="M3 14h5l2 3h4l2-3h5"/>',
 send:'<path d="m22 2-7 20-4-9-9-4Z"/><path d="M22 2 11 13"/>',
 menu:'<path d="M4 7h16M4 12h16M4 17h16"/>',
 };return '<svg aria-hidden="true" width="'+size+'" height="'+size+'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">'+(paths[name]||paths.shield)+'</svg>'};
const logo=()=>'<span class="logo-mark">'+icon('eye',19)+'</span><span class="logo-text">leak<span>lens</span><b>.</b></span>';
function navigate(next:Page){page=next;error='';notice='';sendStage='edit';reviewed=null;renderApp();document.querySelector('main')?.scrollIntoView({block:'start'});}
function home(){
 root!.innerHTML=`<div class="site">
 <header class="public-header"><div class="wrap header-inner"><a class="brand" href="/">${logo()}</a><nav class="public-nav"><a href="#why">The problem</a><a href="#how">How it works</a><a href="#security">Security</a><a href="${github}" target="_blank" rel="noopener">GitHub ↗</a></nav><a class="button button-dark small" href="/app">Launch app ${icon('arrow',16)}</a></div></header>
 <main>
 <section class="hero wrap"><div class="hero-copy"><div class="eyebrow"><span class="pulse"></span> A clearer way to send Zcash</div><h1>Your payment.<br><em>Your privacy.</em><br>Your call.</h1><p>Know what a payment might reveal <strong>before</strong> you approve it. LeakLens puts privacy context right where you make the decision.</p><div class="hero-buttons"><a class="button button-primary" href="/app">Open LeakLens ${icon('arrow')}</a><a class="button button-outline" href="#how">See how it works ${icon('chevron',16)}</a></div><div class="hero-trust">${icon('shield',16)} Testnet first. Non-custodial. Open source.</div></div>
 <div class="hero-product"><div class="preview-browser"><div class="preview-top"><div class="preview-dots"><i></i><i></i><i></i></div><span>LeakLens / Privacy review</span><span class="preview-live">● TESTNET</span></div><div class="preview-content"><div class="preview-label">TRANSACTION PREVIEW</div><div class="preview-heading">Make the informed move.</div><div class="preview-overview"><div><small>YOU'RE SENDING</small><strong>0.0500 <span>TAZ</span></strong></div><div class="preview-send-icon">${icon('arrow')}</div><div><small>TO A</small><strong>Public address</strong></div></div><div class="preview-alert"><div class="preview-alert-icon">!</div><div><strong>Some details will be public</strong><p>Your recipient address and payment amount may be visible to anyone on the blockchain.</p></div></div><div class="preview-bottom"><div>${icon('lock',16)} Check the route before approval</div><span>EDIT PAYMENT →</span></div></div></div><span class="preview-caption">THE MOMENT BEFORE YOU SEND. REIMAGINED.</span></div></section>
 <div class="ticker"><div class="wrap ticker-content"><span>PRIVATE BY INTENTION</span><i></i><span>OPEN-SOURCE POLICY ENGINE</span><i></i><span>BUILT FOR ZCASH</span><i></i><span>REAL TESTNET TRANSACTIONS</span></div></div>
 <section class="section problem" id="why"><div class="wrap two-col"><div><span class="kicker">01 / THE PROBLEM</span><h2>A private coin doesn't make every payment private.</h2></div><div class="problem-right"><p>Zcash supports both shielded and transparent transfers. The route your wallet uses can change what's visible to the world. Most people discover that distinction too late.</p><div class="insight-card"><span class="insight-badge">THE MISSING MOMENT</span><div>Not after the transaction.<br><strong>Before you hit send.</strong></div></div></div></div></section>
 <section class="section workflow" id="how"><div class="wrap"><div class="section-top"><div><span class="kicker">02 / HOW IT WORKS</span><h2>Clarity in three steps.</h2></div><p>No privacy jargon required. Understand the tradeoff and decide what to do next.</p></div><div class="step-grid"><article><span class="step-count">01</span><div class="step-symbol">${icon('wallet',28)}</div><h3>Connect your wallet</h3><p>Use the Noir Wallet testnet extension. Your keys and signing authority stay with your wallet.</p></article><article><span class="step-count">02</span><div class="step-symbol">${icon('eye',28)}</div><h3>See the exposure</h3><p>Enter a payment and get clear warnings about publicly visible data and uncertain routes.</p></article><article><span class="step-count">03</span><div class="step-symbol">${icon('shield',28)}</div><h3>Choose what happens next</h3><p>Change the payment or continue to an explicit approval inside your wallet.</p></article></div></div></section>
 <section class="section contrast" id="security"><div class="wrap contrast-grid"><div><span class="kicker light">03 / BUILT FOR HONEST PRIVACY</span><h2>We show what we know. <em>And what we don't.</em></h2><p>A browser preflight is an estimate, not proof of the final transaction route. Our separate Rust SDK integration can inspect the real unsigned proposal before local signing.</p><a class="button button-light" href="${github}/blob/main/docs/privacy-model.md" target="_blank" rel="noopener">Explore the privacy model ${icon('arrow',17)}</a></div><div class="security-panel"><div class="security-row">${icon('check',22)}<div><b>Non-custodial browser experience</b><span>Wallet owns keys and approves every payment.</span></div></div><div class="security-row">${icon('eye',22)}<div><b>Plain-language signals</b><span>Public facts are separated from possible correlations.</span></div></div><div class="security-row">${icon('lock',22)}<div><b>Exact-proposal Rust gate</b><span>An authoritative local integration—not a claim about another wallet.</span></div></div></div></div></section>
 <section class="final-cta wrap"><span class="kicker">YOUR NEXT PAYMENT, WITH CONTEXT</span><h2>See before you send.</h2><p>Explore the real testnet wallet experience. No real ZEC required.</p><a class="button button-primary" href="/app">Launch LeakLens ${icon('arrow')}</a></section>
 </main><footer class="footer wrap"><a href="/" class="brand">${logo()}</a><span>© 2026 LeakLens. Built for Zcash testnet.</span><a href="${github}" target="_blank" rel="noopener">View source ↗</a></footer></div>`;
}
function walletBadge(){return wallet?'<span class="network-pill"><i></i> Zcash Testnet</span>':'<span class="network-pill"><i></i> TESTNET ONLY</span>';}
function appShell(content:string){
 const menus:[Page,string,string][]=[['overview','Overview','home'],['send','Send payment','send'],['receive','Receive','inbox'],['activity','Activity','activity']];
 root!.innerHTML=`<div class="app-layout"><aside class="sidebar"><a class="brand" href="/">${logo()}</a><div class="sidebar-caption">YOUR WORKSPACE</div><nav class="side-nav">${menus.map(([key,label,glyph])=>`<button data-page="${key}" class="${page===key?'active':''}">${icon(glyph)}${label}</button>`).join('')}</nav><div class="side-foot"><div class="sidebar-security">${icon('shield')}<b>Review before you send</b><p>Your wallet signs. LeakLens helps you decide.</p></div><a href="${github}" target="_blank" rel="noopener">Open-source project ↗</a></div></aside><div class="app-main"><header class="app-header"><a class="mobile-logo brand" href="/">${logo()}</a><div class="app-breadcrumb">Workspace <span>/</span> ${page.charAt(0).toUpperCase()+page.slice(1)}</div><div class="header-actions">${walletBadge()}<button id="connect" class="button button-dark small">${icon('wallet',16)} ${wallet?'Wallet connected':'Connect wallet'}</button></div></header><main class="app-content"><div class="app-content-inner">${error?`<div class="toast error" role="alert">${icon('info',18)}${esc(error)}<button id="dismiss">×</button></div>`:''}${notice?`<div class="toast success" role="status">${icon('check',18)}${esc(notice)}</div>`:''}${content}</div></main><nav class="bottom-nav">${menus.map(([key,label,glyph])=>`<button data-page="${key}" class="${page===key?'active':''}">${icon(glyph,20)}<span>${label.split(' ')[0]}</span></button>`).join('')}</nav></div></div>`;
 document.querySelectorAll<HTMLButtonElement>('[data-page]').forEach(button=>button.addEventListener('click',()=>navigate(button.dataset.page as Page)));
 document.getElementById('connect')?.addEventListener('click',()=>void (wallet?disconnect():connect()));
 document.getElementById('dismiss')?.addEventListener('click',()=>{error='';renderApp()});
}
function greeting(){return '<span class="overline">WELCOME TO YOUR WORKSPACE</span><h1 class="app-title">Privacy starts <em>before</em> send.</h1><p class="app-intro">Your Zcash testnet dashboard. Everything you need to review a payment with confidence.</p>';}
function overview(){
 const balance=wallet?.balance;
 const synced=balance?.accounts?.length?balance.accounts.every(a=>a.synced):!!balance;
 const amount=!wallet?'—':synced?formatBalance(balance?.total):'Syncing…';
 return `${greeting()}<div class="stat-grid"><div class="balance-card"><div class="stat-heading">TOTAL WALLET BALANCE <span>${icon('wallet')}</span></div><div class="balance-value">${esc(amount)} <small>${wallet?'TAZ':''}</small></div><div class="balance-secondary">${wallet?'Available '+esc(formatBalance(balance?.available))+' TAZ':'Connect Noir Wallet to see real funds'}</div><div class="balance-actions"><button class="button button-white" data-jump="send">Send ${icon('arrow',16)}</button><button class="button button-translucent" data-jump="receive">Receive</button></div></div><div class="status-card"><div class="stat-heading">PRIVACY STATUS <span>${icon('shield')}</span></div><div class="status-big">${icon('eye',30)}<strong>Review first.</strong></div><p>LeakLens checks your payment choices before the wallet asks you to authorize a transaction.</p><span class="status-flag">PRE-SEND PROTECTION</span></div></div><div class="lower-grid"><section class="surface"><div class="surface-heading"><h2>Recent activity</h2><button data-jump="activity">View all ${icon('arrow',15)}</button></div>${historyPreview()}</section><section class="surface getting-started"><h2>${wallet?'Your connected wallet':'Get started in a minute'}</h2>${wallet?`<div class="address-box"><span>SHIELDED RECEIVER</span><code>${esc(short(wallet.connection.shielded,15))}</code></div><p>Keep this address private unless you want to share it for payment. Reveal and copy the full value in Receive.</p>`:`<p>Connect the separate Noir Wallet testnet extension to view real balances and prepare a testnet payment.</p><a class="text-link" href="${noirReleases}" target="_blank" rel="noopener">Get the official testnet extension ↗</a>`}</section></div>`;
}
function historyPreview(){if(!wallet)return '<div class="empty-state">'+icon('activity',28)+'<strong>Connect to see your activity</strong><p>Real transactions from your wallet will appear here.</p></div>';
 const items=wallet.history||[];if(!items.length)return '<div class="empty-state">'+icon('activity',28)+'<strong>No transactions yet</strong><p>Testnet activity appears after your wallet syncs.</p></div>';
 return '<div class="transaction-list">'+items.slice(0,4).map(tx=>'<div class="tx-row"><span class="tx-glyph">'+icon('arrow',17)+'</span><div><strong>'+esc(tx.type||'Transaction')+'</strong><small>'+esc(short(tx.txid,9))+'</small></div><div class="tx-end"><strong>'+esc(tx.amount)+' TAZ</strong><small>'+esc(tx.status)+'</small></div></div>').join('')+'</div>';
}
function connectPrompt(){return `<div class="connect-prompt"><div class="connect-illustration">${icon('wallet',38)}</div><h2>Your wallet. Your decision.</h2><p>Connect the <strong>testnet</strong> version of Noir Wallet to view your balance, receive address and activity. Your recovery phrase never enters LeakLens.</p><button class="button button-primary" id="connect-prompt" ${busy?'disabled':''}>${busy?'Waiting for wallet…':'Connect Noir Wallet'} ${icon('arrow',18)}</button><a href="${noirReleases}" target="_blank" rel="noopener">Don't have the testnet extension? Download it ↗</a><small>Install the official release with <code>-testnet.zip</code> and load it in Chrome extensions. The mainnet extension cannot be used here.</small></div>`;}
function sendForm(){return `<div class="form-stack"><form id="payment-form" novalidate><label for="recipient">Recipient testnet address</label><input id="recipient" name="recipient" autocomplete="off" placeholder="utest1… or tm…" value="${esc(draft.to)}" required /><div class="field-hint">Paste a Zcash testnet address. We never need your seed phrase.</div><label for="amount">Amount to send</label><div class="amount-wrap"><input id="amount" name="amount" inputmode="decimal" type="text" placeholder="0.00" value="${esc(draft.amount)}" required/><span>TAZ</span></div><label for="source">Pay from</label><select id="source" name="source"><option value="shielded" ${draft.fundingSource==='shielded'?'selected':''}>Shielded balance · preferred</option><option value="transparent" ${draft.fundingSource==='transparent'?'selected':''}>Transparent balance · public history</option></select><label for="memo">Memo <span>(optional)</span></label><textarea id="memo" name="memo" rows="3" placeholder="Only supported for shielded recipients">${esc(draft.memo)}</textarea><div class="form-footer"><span>${icon('lock',16)} Your wallet makes the final transaction</span><button class="button button-primary" type="submit">Review privacy ${icon('arrow',17)}</button></div></form></div>`;}
function reviewPanel(){
 const a=analysis!; const r=reviewed!;
 return `<div class="review-card"><div class="review-top">${icon(a.risk==='warning'?'info':'shield',30)}<span>PRE-SEND PRIVACY REVIEW</span></div><h2>${esc(a.title)}</h2><p class="review-lead">Here's what to consider before opening the wallet's final approval.</p><div class="review-detail"><div><span>AMOUNT</span><strong>${esc(r.amount)} TAZ</strong></div><div><span>RECIPIENT</span><code>${esc(short(r.to,16))}</code></div><div><span>FUNDING SOURCE</span><strong>${esc(r.fundingSource)}</strong></div></div><div class="finding-panel ${a.risk}"><strong>${a.risk==='warning'?'Privacy warning':'Important context'}</strong><ul>${a.items.map(i=>'<li>'+esc(i)+'</li>').join('')}</ul></div><div class="review-disclaimer">${icon('info',19)}<span>This is a <strong>preflight estimate</strong>, not a verified proposal. Noir Wallet can choose the final route. Inspect the wallet's approval carefully.</span></div><div class="review-actions"><button class="button button-outline" id="edit-payment">Edit payment</button><button class="button button-dark" id="confirm-send" ${busy?'disabled':''}>${busy?'Waiting for wallet…':'Continue to wallet approval'} ${icon('arrow',17)}</button></div></div>`;
}
function send(){
 return '<span class="overline">SEND WITH CLARITY</span><h1 class="app-title">Make a payment. <em>Know the tradeoff.</em></h1><p class="app-intro">Enter the details, check the privacy implications, then approve inside your wallet.</p>'+(!wallet?connectPrompt():sendStage==='sent'?'<div class="success-receipt">'+icon('check',44)+'<h2>Transaction submitted</h2><p>Noir Wallet returned a transaction ID. Check wallet activity for its status.</p><code>'+esc(lastTxid)+'</code><button class="button button-dark" id="new-payment">Make another payment</button></div>':sendStage==='review'||sendStage==='pending'?reviewPanel():sendForm());
}
function receive(){
 return '<span class="overline">YOUR RECEIVE ADDRESS</span><h1 class="app-title">Receive <em>on your terms.</em></h1><p class="app-intro">Share a shielded address when possible. Transparent transfers can be visible on the public chain.</p>'+(!wallet?connectPrompt():`<div class="receive-card"><div class="receive-type">${icon('shield',24)}<div><strong>Shielded receive address</strong><span>Preferred for privacy</span></div></div><div class="full-address" id="shielded-address">${esc(wallet.connection.shielded||'Not available from wallet')}</div><button class="button button-dark" id="copy-shielded">Copy shielded address ${icon('copy',17)}</button><div class="receive-separator">TRANSPARENT ADDRESS — PUBLIC DEPOSITS</div><div class="full-address">${esc(wallet.connection.transparent||'Not available from wallet')}</div><button class="button button-outline" id="copy-transparent">Copy transparent address ${icon('copy',17)}</button><div class="mini-notice">${icon('info',17)} Receiving on a transparent address publicly reveals that deposit. Your wallet may offer a shielding action afterward.</div></div>`);
}
function activity(){
 return '<span class="overline">TRANSACTION HISTORY</span><h1 class="app-title">Every move, <em>in view.</em></h1><p class="app-intro">Your wallet supplies these transactions and their current status.</p>'+(!wallet?connectPrompt():`<section class="surface activity-surface"><div class="surface-heading"><h2>Wallet transactions</h2><button id="refresh">Refresh ${icon('activity',15)}</button></div>${historyPreview()}</section>`);
}
function renderApp(){
 let content=page==='overview'?overview():page==='send'?send():page==='receive'?receive():activity();
 appShell(content);
 document.querySelectorAll<HTMLButtonElement>('[data-jump]').forEach(b=>b.addEventListener('click',()=>navigate(b.dataset.jump as Page)));
 document.getElementById('connect-prompt')?.addEventListener('click',()=>void connect());
 document.getElementById('refresh')?.addEventListener('click',()=>void refresh());
 document.getElementById('payment-form')?.addEventListener('submit',event=>{event.preventDefault();const parsed=parsePaymentDraft(readDraft());if(!parsed.ok){error=parsed.message;renderApp();return;}error='';reviewed={...parsed.value};analysis=analyzePreflight(parsed.value);sendStage='review';renderApp();});
 document.getElementById('edit-payment')?.addEventListener('click',()=>{sendStage='edit';reviewed=null;analysis=null;renderApp()});
 document.getElementById('confirm-send')?.addEventListener('click',()=>void authorize());
 document.getElementById('new-payment')?.addEventListener('click',()=>{sendStage='edit';draft={to:'',amount:'',memo:'',fundingSource:'shielded'};lastTxid='';renderApp()});
 document.getElementById('copy-shielded')?.addEventListener('click',()=>void copy(wallet?.connection.shielded||''));
 document.getElementById('copy-transparent')?.addEventListener('click',()=>void copy(wallet?.connection.transparent||''));
}
function readDraft():Payment{
 draft={to:(document.getElementById('recipient') as HTMLInputElement).value.trim(),amount:(document.getElementById('amount') as HTMLInputElement).value.trim(),memo:(document.getElementById('memo') as HTMLTextAreaElement).value,fundingSource:(document.getElementById('source') as HTMLSelectElement).value as Payment['fundingSource']};
 return draft;
}
async function connect(){if(busy)return;busy=true;error='';notice='';renderApp();try{await noirWallet.connect();await refresh(false);notice='Testnet wallet connected.';}catch(e){error=e instanceof Error?e.message:'Wallet connection failed.';}finally{busy=false;renderApp();}}
async function refresh(redraw=true){try{wallet=await noirWallet.refresh();if(redraw){error='';renderApp();}}catch(e){wallet=null;error=e instanceof Error?e.message:'Unable to load wallet state.';if(redraw)renderApp();}}
async function disconnect(){try{await noirWallet.disconnect();wallet=null;sendStage='edit';reviewed=null;notice='Wallet disconnected.';}catch(e){error=e instanceof Error?e.message:'Disconnect failed.';}renderApp();}
async function authorize(){
 if(busy||!wallet||!reviewed)return;
 if(!samePayment(draft,reviewed)){error='Payment changed after review. Review again.';sendStage='edit';renderApp();return;}
 const parsed=parsePaymentDraft(reviewed);if(!parsed.ok){error=parsed.message;sendStage='edit';renderApp();return;}
 busy=true;sendStage='pending';error='';renderApp();
 try{const txid=await noirWallet.send({to:reviewed.to,amount:reviewed.amount,fundingSource:reviewed.fundingSource,...(reviewed.memo?{memo:reviewed.memo}:{})});if(!txid)throw Error('Wallet did not return a transaction ID. Check the wallet activity before trying again.');lastTxid=txid;sendStage='sent';try{wallet=await noirWallet.refresh();}catch{notice='Transaction submitted. Wallet history may still be syncing.';}}
 catch(e){error=e instanceof Error?e.message:'Wallet did not authorize this payment.';sendStage='review';}
 finally{busy=false;renderApp();}
}
async function copy(value:string){if(!value){error='No address available yet.';renderApp();return;}try{await navigator.clipboard.writeText(value);notice='Address copied to clipboard.';error='';}catch{error='Copy failed. Select and copy the address manually.';}renderApp();}
if(location.pathname.startsWith('/app')){renderApp();if(noirWallet.installed())void (async()=>{try{const existing=await noirWallet.existing();if(existing)await refresh();}catch(e){error=e instanceof Error?e.message:'Unable to check connected wallet.';renderApp();}})();}else home();
