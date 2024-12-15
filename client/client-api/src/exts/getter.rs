use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use crate::base::getter::{GetAccount, GetMultipleAccounts};
use crate::errors::{ClientError, ClientResult};

pub trait GetAccountExt: GetAccount {
    fn try_get_account(&self, pubkey: &Pubkey) -> ClientResult<Account> {
        match self.get_account(pubkey)? {
            Some(account) => Ok(account),
            None => Err(ClientError::AccountNotFound(*pubkey)),
        }
    }
}

impl<C: ?Sized + GetAccount> GetAccountExt for C {}

pub trait GetMultipleAccountsExt: GetMultipleAccounts {
    fn try_get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Account>> {
        self.get_multiple_accounts(pubkeys).and_then(|accounts| {
            pubkeys
                .iter()
                .copied()
                .zip(accounts)
                .map(|(key, acc_opt)| acc_opt.ok_or(ClientError::AccountNotFound(key)))
                .collect()
        })
    }

    fn get_multiple_accounts_lossy(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        self.get_multiple_accounts(pubkeys).map(|accounts| {
            pubkeys
                .iter()
                .copied()
                .zip(accounts)
                .filter_map(|(key, acc_opt)| acc_opt.map(|acc| (key, acc)))
                .collect()
        })
    }

    fn get_multiple_accounts_array<const N: usize>(
        &self,
        pubkeys: &[Pubkey; N],
    ) -> ClientResult<[Option<Account>; N]> {
        let accounts = self.get_multiple_accounts(pubkeys)?;
        assert_eq!(accounts.len(), N);
        Ok(accounts.try_into().unwrap())
    }

    fn try_get_multiple_accounts_array<const N: usize>(
        &self,
        pubkeys: &[Pubkey; N],
    ) -> ClientResult<[Account; N]> {
        let accounts = self.try_get_multiple_accounts(pubkeys)?;
        assert_eq!(accounts.len(), N);
        Ok(accounts.try_into().unwrap())
    }
}

impl<C: ?Sized + GetMultipleAccounts> GetMultipleAccountsExt for C {}
