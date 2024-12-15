use solana_banks_interface::{
    BanksTransactionResultWithMetadata, BanksTransactionResultWithSimulation, TransactionMetadata,
    TransactionSimulationDetails,
};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;

use dexter_solana_banks_client_blocking::BanksClient;

use crate::base::executor::{ProcessTransaction, SimulateTransaction};
use crate::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetMultipleAccounts,
};
use crate::client::Client;
use crate::errors::ClientResult;
use crate::execution::ExecutionOutput;
use crate::internals::sanitize::SanitizeTransaction;

impl Client for BanksClient {}

impl GetAccount for BanksClient {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        Ok(self
            .clone()
            .get_account_with_commitment(*pubkey, CommitmentLevel::Processed)?)
    }
}

impl GetMultipleAccounts for BanksClient {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        let mut client = self.clone();
        let mut accounts = Vec::new();
        for pubkey in pubkeys {
            accounts.push(client.get_account_with_commitment(*pubkey, CommitmentLevel::Processed)?);
        }
        Ok(accounts)
    }
}

impl GetMinimumBalanceForRentExemption for BanksClient {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        Ok(self.clone().get_rent()?.minimum_balance(data_len))
    }
}

impl GetLatestBlockhash for BanksClient {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        let (blockhash, _) = self
            .clone()
            .get_latest_blockhash_with_commitment(CommitmentLevel::Processed)?
            .expect("missing blockhash");
        Ok(blockhash)
    }
}

impl ProcessTransaction<Signature> for BanksClient {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<Signature> {
        let signature = transaction.signatures[0];
        self.clone()
            .process_transaction_with_metadata(transaction)?;
        Ok(signature)
    }
}

impl ProcessTransaction<ExecutionOutput> for BanksClient {
    fn process_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        let BanksTransactionResultWithMetadata { result, metadata } = self
            .clone()
            .process_transaction_with_metadata(transaction.clone())?;

        let Some(metadata) = metadata else {
            return Err(result.unwrap_err().into());
        };

        let fee = self
            .get_fee_for_versioned_transaction(transaction.clone())?
            .unwrap();

        let TransactionMetadata {
            log_messages,
            compute_units_consumed,
            return_data,
        } = metadata;

        Ok(ExecutionOutput {
            transaction,
            result,
            logs: log_messages,
            compute_units_consumed,
            return_data,
            fee,
        })
    }
}

impl SimulateTransaction<ExecutionOutput> for BanksClient {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        let result = self
            .clone()
            .simulate_transaction_with_commitment(transaction.clone(), CommitmentLevel::Processed);

        match result {
            Ok(BanksTransactionResultWithSimulation {
                result,
                simulation_details,
            }) => {
                let result = result.expect("missing transaction result");

                let TransactionSimulationDetails {
                    logs,
                    units_consumed,
                    return_data,
                    inner_instructions: _,
                } = simulation_details.expect("missing transaction simulation details");

                if units_consumed == 0 {
                    return Err(result.unwrap_err().into());
                }

                let fee = self
                    .get_fee_for_versioned_transaction(transaction.clone())?
                    .unwrap();

                Ok(ExecutionOutput {
                    transaction,
                    result,
                    logs,
                    compute_units_consumed: units_consumed,
                    return_data,
                    fee,
                })
            }
            Err(err) => Err(err.into()),
        }
    }
}

trait BanksClientExt {
    fn get_fee_for_versioned_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<Option<u64>>;
}

impl BanksClientExt for BanksClient {
    fn get_fee_for_versioned_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<Option<u64>> {
        let sanitized_transaction = self.sanitize_transaction(transaction.clone())?;

        let account_keys: Vec<_> = sanitized_transaction
            .message()
            .account_keys()
            .iter()
            .copied()
            .collect();

        let legacy_message = solana_sdk::message::legacy::Message {
            header: *transaction.message.header(),
            account_keys,
            recent_blockhash: *transaction.message.recent_blockhash(),
            instructions: transaction.message.instructions().to_vec(),
        };

        Ok(self.clone().get_fee_for_message(legacy_message)?)
    }
}
