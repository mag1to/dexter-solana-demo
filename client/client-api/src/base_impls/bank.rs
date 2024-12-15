use solana_accounts_db::accounts_index::{AccountIndex, IndexKey};
use solana_accounts_db::transaction_results::{
    DurableNonceFee, TransactionExecutionDetails, TransactionExecutionResult,
};
use solana_runtime::bank::{Bank, TransactionSimulationResult};
use solana_sdk::account::{Account, AccountSharedData, ReadableAccount};
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::system_program;
use solana_sdk::transaction::{
    SanitizedTransaction, TransactionVerificationMode, VersionedTransaction,
};

use crate::base::executor::{ProcessTransaction, SimulateTransaction};
use crate::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetMultipleAccounts,
    GetProgramAccounts, ProgramAccountsFilter,
};
use crate::base::setter::{HasRent, SetAccount};
use crate::client::Client;
use crate::errors::{ClientError, ClientResult};
use crate::execution::{ExecutionEffect, ExecutionOutput};

impl Client for Bank {}

impl GetAccount for Bank {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        Ok(Bank::get_account(self, pubkey).map(Into::into))
    }
}

impl GetProgramAccounts for Bank {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        let filters = filters.unwrap_or_default();
        let filter_closure = |account: &AccountSharedData| {
            filters
                .iter()
                .all(|filter_type| filter_type.allows(account))
        };

        let indexed = self
            .rc
            .accounts
            .accounts_db
            .account_indexes
            .contains(&AccountIndex::ProgramId);

        let scan_result = if indexed {
            self.get_filtered_indexed_accounts(
                &IndexKey::ProgramId(*program_id),
                |account| account.owner() == program_id && filter_closure(account),
                &Default::default(),
                self.byte_limit_for_scans(),
            )
            .map(|mut accounts| {
                accounts.sort_by_key(|(key, _)| *key);
                accounts
            })
        } else {
            self.get_filtered_program_accounts(program_id, filter_closure, &Default::default())
        };

        let program_accounts = scan_result
            .map_err(|e| ClientError::DomainSpecific(e.into()))?
            .into_iter()
            .map(|(key, data)| (key, data.into()))
            .collect();

        Ok(program_accounts)
    }
}

impl GetMultipleAccounts for Bank {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        let mut accounts = Vec::with_capacity(pubkeys.len());
        for pubkey in pubkeys {
            accounts.push(GetAccount::get_account(self, pubkey)?);
        }
        Ok(accounts)
    }
}

impl GetMinimumBalanceForRentExemption for Bank {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        Ok(Bank::get_minimum_balance_for_rent_exemption(self, data_len))
    }
}

impl GetLatestBlockhash for Bank {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        Ok(self.last_blockhash())
    }
}

impl SetAccount for Bank {
    fn set_account(&mut self, pubkey: Pubkey, account: Account) {
        self.store_account(&pubkey, &account);
    }
}

impl HasRent for Bank {
    fn rent(&self) -> Rent {
        self.rent_collector().rent
    }

    fn minimum_balance_for_rent_exemption(&self, data_len: usize) -> u64 {
        Bank::get_minimum_balance_for_rent_exemption(self, data_len)
    }
}

impl ProcessTransaction<ExecutionOutput> for Bank {
    fn process_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        let result = self.process_transaction_with_metadata(transaction.clone());

        let details = match result {
            TransactionExecutionResult::Executed { details, .. } => details,
            TransactionExecutionResult::NotExecuted(tx_error) => {
                return Err(tx_error.into());
            }
        };

        let TransactionExecutionDetails {
            status,
            log_messages,
            inner_instructions: _,
            durable_nonce_fee,
            return_data,
            executed_units,
            accounts_data_len_delta: _,
        } = details;

        let sanitized_transaction = self
            .verify_transaction(transaction.clone(), TransactionVerificationMode::HashOnly)
            .expect("must be verified after execution");

        let lamports_per_signature = match durable_nonce_fee {
            Some(DurableNonceFee::Valid(lamports_per_signature)) => Some(lamports_per_signature),
            Some(DurableNonceFee::Invalid) => None,
            None => self.get_lamports_per_signature_for_blockhash(
                sanitized_transaction.message().recent_blockhash(),
            ),
        }
        .expect("must be available");

        let fee = self.get_fee_for_message_with_lamports_per_signature(
            sanitized_transaction.message(),
            lamports_per_signature,
        );

        Ok(ExecutionOutput {
            transaction,
            result: status,
            logs: log_messages.unwrap_or_default(),
            compute_units_consumed: executed_units,
            return_data,
            fee,
        })
    }
}

impl SimulateTransaction<ExecutionOutput> for Bank {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        SimulateTransaction::<ExecutionEffect>::simulate_transaction(self, transaction)
            .map(Into::into)
    }
}

impl SimulateTransaction<ExecutionEffect> for Bank {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionEffect> {
        let sanitized_transaction = self.fully_verify_transaction(transaction.clone())?;
        let result = self.simulate_transaction_unchecked(&sanitized_transaction, false);

        if result.units_consumed == 0 {
            return Err(result.result.unwrap_err().into());
        }

        let lamports_per_signature = self
            .get_lamports_per_signature_for_blockhash(
                sanitized_transaction.message().recent_blockhash(),
            )
            .unwrap();

        let fee = self.get_fee_for_message_with_lamports_per_signature(
            sanitized_transaction.message(),
            lamports_per_signature,
        );

        Ok(convert_simulation_result(
            self,
            transaction,
            sanitized_transaction,
            result,
            fee,
        ))
    }
}

fn convert_simulation_result(
    bank: &Bank,
    transaction: VersionedTransaction,
    sanitized_transaction: SanitizedTransaction,
    result: TransactionSimulationResult,
    fee: u64,
) -> ExecutionEffect {
    let TransactionSimulationResult {
        result,
        logs,
        post_simulation_accounts,
        units_consumed,
        return_data,
        inner_instructions: _,
    } = result;

    // TODO: missing post accounts if the tx is not executed (e.g. blockhash not found)
    let account_keys = sanitized_transaction.message().account_keys();
    assert_eq!(post_simulation_accounts.len(), account_keys.len());

    let post_accounts = account_keys
        .iter()
        .map(|account_key| {
            let account: Account = post_simulation_accounts
                .iter()
                .find_map(|(key, account)| key.eq(account_key).then_some(account.clone().into()))
                .unwrap();

            if account.owner == system_program::id()
                && account.data.is_empty()
                && account.lamports == 0
            {
                (*account_key, None)
            } else if account.executable {
                let program_account = bank.get_account(account_key).unwrap();
                (*account_key, Some(program_account.into()))
            } else {
                (*account_key, Some(account))
            }
        })
        .collect();

    ExecutionEffect {
        transaction,
        result,
        logs,
        compute_units_consumed: units_consumed,
        return_data,
        fee,
        post_accounts,
    }
}
