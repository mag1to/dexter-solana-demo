use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;

use base64::prelude::{Engine, BASE64_STANDARD};

use solana_account_decoder::UiAccountEncoding;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::client_error::{
    Error as RpcClientError, ErrorKind as RpcClientErrorKind, Result as RpcClientResult,
};
use solana_rpc_client_api::config::{
    RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig,
    RpcSimulateTransactionAccountsConfig, RpcSimulateTransactionConfig, RpcTransactionConfig,
};
use solana_rpc_client_api::request::{RpcError, RpcRequest, RpcResponseErrorData};
use solana_rpc_client_api::response::{Response as RpcResponse, RpcSimulateTransactionResult};
use solana_sdk::account::Account;
use solana_sdk::bs58;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::hash::Hash;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::system_program;
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};
use solana_sdk::transaction_context::TransactionReturnData;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransactionWithStatusMeta,
    UiTransactionEncoding, UiTransactionReturnData, UiTransactionStatusMeta,
};

use crate::base::executor::{ProcessTransaction, SimulateTransaction};
use crate::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetMultipleAccounts,
    GetProgramAccounts, ProgramAccountsFilter,
};
use crate::client::Client;
use crate::errors::ClientResult;
use crate::execution::{ExecutionEffect, ExecutionOutput};
use crate::exts::getter::GetMultipleAccountsExt;
use crate::internals::sanitize::SanitizeTransaction;

impl Client for RpcClient {}

impl GetAccount for RpcClient {
    fn get_account(&self, pubkey: &Pubkey) -> ClientResult<Option<Account>> {
        let response = self.get_account_with_commitment(pubkey, self.commitment())?;
        Ok(response.value)
    }
}

impl GetProgramAccounts for RpcClient {
    fn get_program_accounts(
        &self,
        program_id: &Pubkey,
        filters: Option<Vec<ProgramAccountsFilter>>,
    ) -> ClientResult<Vec<(Pubkey, Account)>> {
        let mut program_accounts = self.get_program_accounts_with_config(
            program_id,
            RpcProgramAccountsConfig {
                filters,
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(self.commitment()),
                    min_context_slot: None,
                },
                with_context: None,
            },
        )?;

        // returned accounts are not sorted if the underlying bank enables indexing
        program_accounts.sort_by_key(|(key, _)| *key);

        Ok(program_accounts)
    }
}

impl GetMultipleAccounts for RpcClient {
    fn get_multiple_accounts(&self, pubkeys: &[Pubkey]) -> ClientResult<Vec<Option<Account>>> {
        let accounts = self
            .get_multiple_accounts_with_commitment(pubkeys, self.commitment())?
            .value;
        assert_eq!(accounts.len(), pubkeys.len());
        Ok(accounts)
    }
}

impl GetMinimumBalanceForRentExemption for RpcClient {
    fn get_minimum_balance_for_rent_exemption(&self, data_len: usize) -> ClientResult<u64> {
        Ok(RpcClient::get_minimum_balance_for_rent_exemption(
            self, data_len,
        )?)
    }
}

impl GetLatestBlockhash for RpcClient {
    fn get_latest_blockhash(&self) -> ClientResult<Hash> {
        let (blockhash, _) = self.get_latest_blockhash_with_commitment(self.commitment())?;
        Ok(blockhash)
    }
}

impl ProcessTransaction<Signature> for RpcClient {
    fn process_transaction(&self, transaction: VersionedTransaction) -> ClientResult<Signature> {
        let signature = transaction.signatures[0];

        let result = self.send_transaction_with_config(
            &transaction,
            RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Processed),
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        );

        if let Err(RpcClientError {
            kind:
                RpcClientErrorKind::RpcError(RpcError::RpcResponseError {
                    data: RpcResponseErrorData::SendTransactionPreflightFailure(tx_result),
                    ..
                }),
            ..
        }) = result
        {
            if tx_result.units_consumed.unwrap() == 0 {
                return Err(tx_result.err.unwrap().into());
            }
        }

        let result = self.send_and_confirm_transaction_with_spinner_and_config(
            &transaction,
            CommitmentConfig::confirmed(),
            RpcSendTransactionConfig {
                skip_preflight: true,
                preflight_commitment: None,
                encoding: None,
                max_retries: None,
                min_context_slot: None,
            },
        );

        let error = match result {
            // confirmed successfully
            Ok(confirmed_signature) => {
                assert_eq!(confirmed_signature, signature);
                return Ok(signature);
            }
            Err(error) => error,
        };

        match &error.kind {
            // confirmed but failed
            RpcClientErrorKind::TransactionError(_) => Ok(signature),
            _ => Err(error.into()),
        }
    }
}

impl ProcessTransaction<ExecutionOutput> for RpcClient {
    fn process_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        const MAX_RETRIES: usize = 10;
        const RETRY_INTERVAL: Duration = Duration::from_secs(1);

        let signature =
            ProcessTransaction::<Signature>::process_transaction(self, transaction.clone())?;

        let confirmed = {
            let mut num_retries = 0;

            loop {
                let result = self.get_transaction_with_config(
                    &signature,
                    RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: Some(0),
                    },
                );

                match result {
                    Ok(confirmed) => break confirmed,
                    Err(err) if num_retries >= MAX_RETRIES => return Err(err.into()),
                    Err(_) => {
                        num_retries += 1;
                        thread::sleep(RETRY_INTERVAL);
                    }
                }
            }
        };

        Ok(convert_processed(transaction, confirmed))
    }
}

impl SimulateTransaction<ExecutionOutput> for RpcClient {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionOutput> {
        SimulateTransaction::<ExecutionEffect>::simulate_transaction(self, transaction)
            .map(Into::into)
    }
}

impl SimulateTransaction<ExecutionEffect> for RpcClient {
    fn simulate_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<ExecutionEffect> {
        let sanitized_transaction = self.sanitize_transaction(transaction.clone())?;

        let addresses = sanitized_transaction
            .message()
            .account_keys()
            .iter()
            .map(ToString::to_string)
            .collect();

        let result = self
            .simulate_transaction_with_config(
                &transaction,
                RpcSimulateTransactionConfig {
                    sig_verify: true,
                    replace_recent_blockhash: false,
                    commitment: Some(CommitmentConfig::processed()),
                    encoding: Some(UiTransactionEncoding::Base64),
                    accounts: Some(RpcSimulateTransactionAccountsConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        addresses,
                    }),
                    min_context_slot: None,
                    inner_instructions: false,
                },
            )?
            .value;

        if result.units_consumed.unwrap() == 0 {
            return Err(result.err.unwrap().into());
        }

        let fee = self.get_fee_for_versioned_message(&transaction.message)?;

        convert_simulated(self, transaction, sanitized_transaction, result, fee)
    }
}

fn convert_processed(
    transaction: VersionedTransaction,
    confirmed: EncodedConfirmedTransactionWithStatusMeta,
) -> ExecutionOutput {
    let EncodedConfirmedTransactionWithStatusMeta {
        transaction:
            EncodedTransactionWithStatusMeta {
                meta:
                    Some(UiTransactionStatusMeta {
                        err,
                        fee,
                        log_messages: OptionSerializer::Some(logs),
                        return_data: ui_return_data_opt,
                        compute_units_consumed: OptionSerializer::Some(compute_units_consumed),
                        ..
                    }),
                ..
            },
        ..
    } = confirmed
    else {
        panic!("unexpected transaction format: {:?}", confirmed);
    };

    let return_data = if let OptionSerializer::Some(ui_return_data) = ui_return_data_opt {
        let UiTransactionReturnData {
            program_id,
            data: (ui_data, _),
        } = ui_return_data;

        let program_id = program_id.parse().expect("return data program id");
        let data = BASE64_STANDARD.decode(ui_data).expect("return data data");

        Some(TransactionReturnData { program_id, data })
    } else {
        None
    };

    let result = match err {
        None => Ok(()),
        Some(err) => Err(err),
    };

    ExecutionOutput {
        transaction,
        result,
        logs,
        compute_units_consumed,
        return_data,
        fee,
    }
}

fn convert_simulated<C: GetMultipleAccounts>(
    client: &C,
    transaction: VersionedTransaction,
    sanitized_transaction: SanitizedTransaction,
    result: RpcSimulateTransactionResult,
    fee: u64,
) -> ClientResult<ExecutionEffect> {
    let RpcSimulateTransactionResult {
        err,
        logs,
        accounts: ui_accounts_opt,
        units_consumed,
        return_data: ui_return_data_opt,
        inner_instructions: _,
    } = result;

    let ui_accounts = ui_accounts_opt.unwrap();

    let account_keys: Vec<_> = sanitized_transaction
        .message()
        .account_keys()
        .iter()
        .copied()
        .collect();
    assert_eq!(ui_accounts.len(), account_keys.len());

    let post_accounts: Vec<(Pubkey, Option<Account>)> = account_keys
        .into_iter()
        .zip(ui_accounts)
        .map(|(key, ui_acc_opt)| {
            let acc_opt = ui_acc_opt.map(|ui_acc| ui_acc.decode::<Account>().unwrap());
            (key, acc_opt)
        })
        .map(|(key, acc_opt)| {
            let acc_opt = acc_opt.and_then(|acc| {
                if acc.owner == system_program::id() && acc.data.is_empty() && acc.lamports == 0 {
                    None
                } else {
                    Some(acc)
                }
            });
            (key, acc_opt)
        })
        .collect();

    let program_ids: Vec<_> = post_accounts
        .iter()
        .filter_map(|(key, acc_opt)| {
            acc_opt
                .as_ref()
                .and_then(|acc| acc.executable.then_some(*key))
        })
        .collect();

    let mut programs: BTreeMap<_, _> = program_ids
        .iter()
        .copied()
        .zip(client.try_get_multiple_accounts(&program_ids)?)
        .collect();

    let post_accounts = post_accounts
        .into_iter()
        .map(|(key, acc_opt)| {
            let acc_opt = acc_opt.map(|acc| programs.remove(&key).unwrap_or(acc));
            (key, acc_opt)
        })
        .collect();

    let return_data = ui_return_data_opt.map(|ui_return_data| {
        let UiTransactionReturnData {
            program_id,
            data: (ui_data, _),
        } = ui_return_data;

        let program_id = program_id.parse().unwrap();
        let data = BASE64_STANDARD.decode(ui_data).unwrap();

        TransactionReturnData { program_id, data }
    });

    let result = if let Some(err) = err {
        Err(err)
    } else {
        Ok(())
    };

    Ok(ExecutionEffect {
        transaction,
        result,
        logs: logs.unwrap(),
        compute_units_consumed: units_consumed.unwrap(),
        return_data,
        fee,
        post_accounts,
    })
}

trait RpcClientExt {
    fn get_fee_for_versioned_message(&self, message: &VersionedMessage) -> RpcClientResult<u64>;
}

impl RpcClientExt for RpcClient {
    fn get_fee_for_versioned_message(&self, message: &VersionedMessage) -> RpcClientResult<u64> {
        let serialized_encoded = serialize_and_encode(message, UiTransactionEncoding::Base64)?;
        let result = self.send::<RpcResponse<Option<u64>>>(
            RpcRequest::GetFeeForMessage,
            serde_json::json!([serialized_encoded, self.commitment()]),
        )?;
        result
            .value
            .ok_or_else(|| RpcClientErrorKind::Custom("Invalid blockhash".to_string()).into())
    }
}

fn serialize_and_encode<T>(input: &T, encoding: UiTransactionEncoding) -> RpcClientResult<String>
where
    T: serde::ser::Serialize,
{
    let serialized = bincode::serialize(input)
        .map_err(|e| RpcClientErrorKind::Custom(format!("Serialization failed: {e}")))?;
    let encoded = match encoding {
        UiTransactionEncoding::Base58 => bs58::encode(serialized).into_string(),
        UiTransactionEncoding::Base64 => BASE64_STANDARD.encode(serialized),
        _ => {
            return Err(RpcClientErrorKind::Custom(format!(
                "unsupported encoding: {encoding}. Supported encodings: base58, base64"
            ))
            .into())
        }
    };
    Ok(encoded)
}
