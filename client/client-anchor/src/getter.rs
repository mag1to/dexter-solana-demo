use solana_sdk::pubkey::Pubkey;

use anchor_lang::AccountDeserialize;

use dexter_client_api::base::getter::{
    GetAccount, GetMultipleAccounts, GetProgramAccounts, ProgramAccountsFilter,
};
use dexter_client_api::errors::{ClientError, ClientResult};
use dexter_client_api::Client;

use crate::account::AnchorAccount;

pub trait AnchorGetter: Client {
    fn get_anchor_account<T>(&self, pubkey: &Pubkey) -> ClientResult<Option<AnchorAccount<T>>>
    where
        Self: GetAccount,
        T: AccountDeserialize,
    {
        let Some(account) = self.get_account(pubkey)? else {
            return Ok(None);
        };

        let account = AnchorAccount::try_from_account(*pubkey, account)
            .map_err(|_| ClientError::AccountDidNotDeserialize(*pubkey))?;

        Ok(Some(account))
    }

    fn try_get_anchor_account<T>(&self, pubkey: &Pubkey) -> ClientResult<AnchorAccount<T>>
    where
        Self: GetAccount,
        T: AccountDeserialize,
    {
        match self.get_anchor_account(pubkey)? {
            Some(account) => Ok(account),
            None => Err(ClientError::AccountNotFound(*pubkey)),
        }
    }

    fn get_anchor_program_accounts<T>(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<AnchorAccount<T>>>
    where
        Self: GetProgramAccounts,
        T: AccountDeserialize,
    {
        self.get_program_accounts(program_id, filters)?
            .into_iter()
            .map(|(key, account)| {
                AnchorAccount::try_from_account(key, account)
                    .map_err(|_| ClientError::AccountDidNotDeserialize(key))
            })
            .collect()
    }

    fn get_anchor_multiple_accounts<T>(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<Option<AnchorAccount<T>>>>
    where
        Self: GetMultipleAccounts,
        T: AccountDeserialize,
    {
        let accounts = self.get_multiple_accounts(pubkeys)?;
        pubkeys
            .iter()
            .copied()
            .zip(accounts)
            .map(|(key, account_opt)| {
                let Some(account) = account_opt else {
                    return Ok(None);
                };

                let account = AnchorAccount::try_from_account(key, account)
                    .map_err(|_| ClientError::AccountDidNotDeserialize(key))?;

                Ok(Some(account))
            })
            .collect()
    }

    fn try_get_anchor_multiple_accounts<T>(
        &self,
        pubkeys: &[Pubkey],
    ) -> ClientResult<Vec<AnchorAccount<T>>>
    where
        Self: GetMultipleAccounts,
        T: AccountDeserialize,
    {
        let anchor_accounts = self.get_anchor_multiple_accounts(pubkeys)?;
        pubkeys
            .iter()
            .copied()
            .zip(anchor_accounts)
            .map(|(key, account)| match account {
                Some(account) => Ok(account),
                None => Err(ClientError::AccountNotFound(key)),
            })
            .collect()
    }
}

impl<C: ?Sized + Client> AnchorGetter for C {}
