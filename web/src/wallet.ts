import { getNoirWallet, type ZcashConnectResult, type ZcashBalanceResult, type TransactionHistoryEntry, type SendTransactionParams } from '@noir-wallet/sdk';
export type WalletData={connection:ZcashConnectResult;balance:ZcashBalanceResult;history:TransactionHistoryEntry[]};
export function isTestnetAddress(address:string):boolean{
 return /^(utest1|ztestsapling|tm|t2)/.test(address);
}
export function formatBalance(balance:string|undefined):string{
 if(balance===undefined||balance===null||balance==='')return '—';
 return balance;
}
function client(){
 const instance=getNoirWallet();
 if(!instance) throw new Error('Noir Wallet is not installed in this browser. Install the testnet extension to continue.');
 return instance.zcash;
}
function requireTestnet(connection:ZcashConnectResult){
 const addresses=[connection.shielded,connection.transparent].filter(Boolean);
 if(!addresses.length||addresses.some(a=>!isTestnetAddress(a)))throw new Error('This account is not Zcash testnet. Install the separate [Testnet] Noir Wallet extension; mainnet wallets are blocked.');
}
export const noirWallet={
 installed:()=>!!getNoirWallet(),
 async existing():Promise<ZcashConnectResult|null>{ const connection=await client().getAccounts();if(connection)requireTestnet(connection);return connection;},
 async connect():Promise<ZcashConnectResult>{const connection=await client().connect();requireTestnet(connection);return connection;},
 async refresh():Promise<WalletData>{
   const connection=await client().getAccounts();if(!connection)throw new Error('Wallet disconnected. Connect again.');
   requireTestnet(connection);
   const [balance,history]=await Promise.all([client().getBalance(),client().getTransactionHistory()]);
   return {connection,balance,history};
 },
 async send(params:SendTransactionParams):Promise<string>{
   const connection=await client().getAccounts();if(!connection)throw new Error('Wallet disconnected.');
   requireTestnet(connection);
   if(!isTestnetAddress(params.to))throw new Error('Only testnet recipients are supported.');
   return client().sendTransaction(params);
 },
 async disconnect(){await client().disconnect();}
};
