use std::collections::{btree_map, BTreeMap};
use thiserror::Error;

use solana_sdk::account::Account;
use solana_sdk::instruction::InstructionError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::{TransactionError, VersionedTransaction};
use solana_sdk::transaction_context::TransactionReturnData;

use anchor_lang::AccountDeserialize;

use crate::errors::{ClientError, ClientResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutput {
    pub transaction: VersionedTransaction,
    pub result: Result<(), TransactionError>,
    pub logs: Vec<String>,
    pub compute_units_consumed: u64,
    pub return_data: Option<TransactionReturnData>,
    pub fee: u64,
}

impl ExecutionOutput {
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    pub fn try_success(self) -> Result<Self, TransactionError> {
        self.result.clone()?;
        Ok(self)
    }

    pub fn signature(&self) -> Signature {
        self.transaction.signatures[0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEffect {
    pub transaction: VersionedTransaction,
    pub result: Result<(), TransactionError>,
    pub logs: Vec<String>,
    pub compute_units_consumed: u64,
    pub return_data: Option<TransactionReturnData>,
    pub fee: u64,
    pub post_accounts: PostAccounts,
}

impl ExecutionEffect {
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    pub fn get_post_account(&self, pubkey: &Pubkey) -> Option<Option<&Account>> {
        self.post_accounts
            .iter()
            .find(|(account_pubkey, _)| *account_pubkey == pubkey)
            .map(|(_, account_opt)| account_opt.as_ref())
    }

    pub fn try_deserialize_post_account<T: AccountDeserialize>(
        &self,
        pubkey: &Pubkey,
    ) -> ClientResult<T> {
        Ok(self.post_accounts.deserialize_account(pubkey)?)
    }

    pub fn custom_error_code(&self) -> Option<u32> {
        if let Err(TransactionError::InstructionError(_, InstructionError::Custom(error_code))) =
            &self.result
        {
            Some(*error_code)
        } else {
            None
        }
    }
}

impl From<ExecutionEffect> for ExecutionOutput {
    fn from(execution: ExecutionEffect) -> Self {
        let ExecutionEffect {
            transaction,
            result,
            logs,
            compute_units_consumed,
            return_data,
            fee,
            ..
        } = execution;

        Self {
            transaction,
            result,
            logs,
            compute_units_consumed,
            return_data,
            fee,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PostAccountsError {
    #[error("an account `{0}` was not found")]
    AccountNotFound(Pubkey),
    #[error("an account `{0}` was closed")]
    AccountClosed(Pubkey),
    #[error("failed to deserialize the account")]
    AccountDidNotDeserialize(Pubkey),
}

impl From<PostAccountsError> for ClientError {
    fn from(error: PostAccountsError) -> Self {
        match error {
            PostAccountsError::AccountNotFound(pubkey) => Self::AccountNotFound(pubkey),
            PostAccountsError::AccountClosed(pubkey) => Self::AccountNotFound(pubkey),
            PostAccountsError::AccountDidNotDeserialize(pubkey) => {
                Self::AccountDidNotDeserialize(pubkey)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostAccounts(BTreeMap<Pubkey, Option<Account>>);

impl PostAccounts {
    pub fn new(accounts: BTreeMap<Pubkey, Option<Account>>) -> Self {
        Self(accounts)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<&Account> {
        self.0.get(pubkey).and_then(Option::as_ref)
    }

    pub fn try_get_account(&self, pubkey: &Pubkey) -> Result<&Account, PostAccountsError> {
        match self.0.get(pubkey) {
            Some(Some(account)) => Ok(account),
            Some(None) => Err(PostAccountsError::AccountClosed(*pubkey)),
            None => Err(PostAccountsError::AccountNotFound(*pubkey)),
        }
    }

    pub fn deserialize_account<T: AccountDeserialize>(
        &self,
        pubkey: &Pubkey,
    ) -> Result<T, PostAccountsError> {
        let account = self.try_get_account(pubkey)?;
        T::try_deserialize(&mut account.data.as_ref())
            .map_err(|_| PostAccountsError::AccountDidNotDeserialize(*pubkey))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Pubkey, &Option<Account>)> {
        self.0.iter()
    }
}

impl From<BTreeMap<Pubkey, Option<Account>>> for PostAccounts {
    fn from(accounts: BTreeMap<Pubkey, Option<Account>>) -> Self {
        Self(accounts)
    }
}

impl FromIterator<(Pubkey, Option<Account>)> for PostAccounts {
    fn from_iter<T: IntoIterator<Item = (Pubkey, Option<Account>)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for PostAccounts {
    type Item = (Pubkey, Option<Account>);
    type IntoIter = btree_map::IntoIter<Pubkey, Option<Account>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a PostAccounts {
    type Item = (&'a Pubkey, &'a Option<Account>);
    type IntoIter = btree_map::Iter<'a, Pubkey, Option<Account>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
