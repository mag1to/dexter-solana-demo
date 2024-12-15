use once_cell::sync::Lazy;
use std::sync::Arc;

use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

use solana_banks_client::BanksClient;

use crate::base::executor::{ProcessTransaction, SimulateTransaction};
use crate::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetMultipleAccounts,
};
use crate::client::Client;
use crate::errors::ClientResult;
use crate::execution::ExecutionOutput;

static RUNTIME: Lazy<Arc<tokio::runtime::Runtime>> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("dexter-bank-clients")
        .enable_all()
        .build()
        .map(Arc::new)
        .unwrap()
});

trait BanksClientExt {
    fn blocking(&self) -> dexter_solana_banks_client_blocking::BanksClient;
}

impl BanksClientExt for BanksClient {
    fn blocking(&self) -> dexter_solana_banks_client_blocking::BanksClient {
        dexter_solana_banks_client_blocking::BanksClient::with_runtime(self.clone(), RUNTIME.clone())
    }
}

impl Client for BanksClient {}

impl GetAccount for BanksClient {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        self.blocking().get_account(pubkey)
    }
}

impl GetMultipleAccounts for BanksClient {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        self.blocking().get_multiple_accounts(pubkeys)
    }
}

impl GetMinimumBalanceForRentExemption for BanksClient {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        self.blocking()
            .get_minimum_balance_for_rent_exemption(data_len)
    }
}

impl GetLatestBlockhash for BanksClient {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        self.blocking().get_latest_blockhash()
    }
}

impl ProcessTransaction<Signature> for BanksClient {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<Signature> {
        self.blocking().process_transaction(transaction)
    }
}

impl ProcessTransaction<ExecutionOutput> for BanksClient {
    fn process_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        self.blocking().process_transaction(transaction)
    }
}

impl SimulateTransaction<ExecutionOutput> for BanksClient {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        self.blocking().simulate_transaction(transaction)
    }
}
