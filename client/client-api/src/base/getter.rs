use std::sync::Arc;

use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;

use crate::client::Client;
use crate::errors::ClientResult;

pub use solana_rpc_client_api::filter::{Memcmp, RpcFilterType as ProgramAccountsFilter};

pub trait GetAccount: Client {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>>;
}

impl<C: ?Sized + GetAccount> GetAccount for &C {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        (**self).get_account(pubkey)
    }
}

impl<C: ?Sized + GetAccount> GetAccount for &mut C {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        (**self).get_account(pubkey)
    }
}

impl<C: ?Sized + GetAccount> GetAccount for Box<C> {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        (**self).get_account(pubkey)
    }
}

impl<C: ?Sized + GetAccount> GetAccount for Arc<C> {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        (**self).get_account(pubkey)
    }
}

pub trait GetProgramAccounts: Client + GetAccount {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>>;
}

impl<C: ?Sized + GetProgramAccounts> GetProgramAccounts for &C {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        (**self).get_program_accounts(program_id, filters)
    }
}

impl<C: ?Sized + GetProgramAccounts> GetProgramAccounts for &mut C {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        (**self).get_program_accounts(program_id, filters)
    }
}

impl<C: ?Sized + GetProgramAccounts> GetProgramAccounts for Box<C> {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        (**self).get_program_accounts(program_id, filters)
    }
}

impl<C: ?Sized + GetProgramAccounts> GetProgramAccounts for Arc<C> {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        (**self).get_program_accounts(program_id, filters)
    }
}

pub trait GetMultipleAccounts: Client + GetAccount {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>>;
}

impl<C: ?Sized + GetMultipleAccounts> GetMultipleAccounts for &C {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        (**self).get_multiple_accounts(pubkeys)
    }
}

impl<C: ?Sized + GetMultipleAccounts> GetMultipleAccounts for &mut C {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        (**self).get_multiple_accounts(pubkeys)
    }
}

impl<C: ?Sized + GetMultipleAccounts> GetMultipleAccounts for Box<C> {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        (**self).get_multiple_accounts(pubkeys)
    }
}

impl<C: ?Sized + GetMultipleAccounts> GetMultipleAccounts for Arc<C> {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        (**self).get_multiple_accounts(pubkeys)
    }
}

pub trait GetMinimumBalanceForRentExemption: Client {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64>;
}

impl<C: ?Sized + GetMinimumBalanceForRentExemption> GetMinimumBalanceForRentExemption for &C {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        (**self).get_minimum_balance_for_rent_exemption(data_len)
    }
}

impl<C: ?Sized + GetMinimumBalanceForRentExemption> GetMinimumBalanceForRentExemption for &mut C {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        (**self).get_minimum_balance_for_rent_exemption(data_len)
    }
}

impl<C: ?Sized + GetMinimumBalanceForRentExemption> GetMinimumBalanceForRentExemption for Box<C> {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        (**self).get_minimum_balance_for_rent_exemption(data_len)
    }
}

impl<C: ?Sized + GetMinimumBalanceForRentExemption> GetMinimumBalanceForRentExemption for Arc<C> {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        (**self).get_minimum_balance_for_rent_exemption(data_len)
    }
}

pub trait GetLatestBlockhash: Client {
    fn get_latest_blockhash(&self) -> ClientResult<Hash>;
}

impl<C: ?Sized + GetLatestBlockhash> GetLatestBlockhash for &C {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        (**self).get_latest_blockhash()
    }
}

impl<C: ?Sized + GetLatestBlockhash> GetLatestBlockhash for &mut C {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        (**self).get_latest_blockhash()
    }
}

impl<C: ?Sized + GetLatestBlockhash> GetLatestBlockhash for Box<C> {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        (**self).get_latest_blockhash()
    }
}

impl<C: ?Sized + GetLatestBlockhash> GetLatestBlockhash for Arc<C> {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        (**self).get_latest_blockhash()
    }
}
