export type FundingSource = 'shielded' | 'transparent';
export type Payment = { to:string; amount:string; memo:string; fundingSource:FundingSource };
export type Preflight = {risk:'warning'|'caution'|'unknown'; title:string; items:string[]; exactProposalVerified:false; destinationType:'transparent'|'unified'|'shielded'|'unknown'};
const money=/^(?:0|[1-9]\d*)(?:\.\d{1,8})?$/;
const prefix=(a:string):Preflight['destinationType'] => {
 if (/^(tm|t2)[A-Za-z0-9]{30,40}$/.test(a)) return 'transparent';
 if (/^utest1[a-z0-9]{60,}$/.test(a)) return 'unified';
 if (/^ztestsapling[a-z0-9]{60,}$/.test(a)) return 'shielded';
 return 'unknown';
};
export function parsePaymentDraft(value:Payment):{ok:true;value:Payment}|{ok:false;message:string}{
 const to=value.to.trim();
 if(prefix(to)==='unknown') return {ok:false,message:'Enter a supported Zcash testnet recipient address.'};
 if(!money.test(value.amount)||Number(value.amount)<=0||!Number.isFinite(Number(value.amount))) return {ok:false,message:'Enter a positive amount with up to eight decimal places.'};
 if(Number(value.amount)>21_000_000) return {ok:false,message:'Amount exceeds the Zcash supply.'};
 if(value.memo && prefix(to)==='transparent') return {ok:false,message:'Memos require a shielded recipient.'};
 if(new TextEncoder().encode(value.memo).length>512) return {ok:false,message:'Memo must be 512 UTF-8 bytes or less.'};
 return {ok:true,value:{...value,to}};
}
export function samePayment(a:Payment,b:Payment):boolean{
 return a.to===b.to && a.amount===b.amount && a.memo===b.memo && a.fundingSource===b.fundingSource;
}
export function analyzePreflight(value:Payment):Preflight{
 const dest=prefix(value.to.trim()); const items:string[]=[];
 if(dest==='transparent') items.push('The recipient address and payment amount will be visible on the public chain.');
 if(dest==='unified') items.push('A Unified Address may contain multiple receiver types; Noir chooses the final route. LeakLens cannot verify that selection before wallet approval.');
 if(dest==='shielded') items.push('This destination appears shielded. Final input, change and output routes remain unverified by LeakLens.');
 if(dest==='unknown') items.push('The destination type cannot be established.');
 if(value.fundingSource==='transparent') items.push('Transparent funding can reveal input history and publicly link funds.');
 if(value.fundingSource==='shielded' && dest==='transparent') items.push('Moving from shielded funding to a public output can create correlation clues; this is not proof of identity.');
 items.push('Preflight estimate only: Noir does not expose the exact unsigned transaction proposal to this app.');
 const risk=dest==='transparent'||value.fundingSource==='transparent'?'warning':dest==='unknown'?'unknown':'caution';
 return {risk,title:risk==='warning'?'Public exposure detected':risk==='unknown'?'Unable to assess the route':'Review the final wallet details',items,exactProposalVerified:false,destinationType:dest};
}
