#![allow(clippy::result_large_err)]

use borsh::BorshDeserialize;
use std::sync::Arc;
use tarpc::context::Context;

use solana_banks_client::BanksClient as AsyncBanksClient;
pub use solana_banks_client::BanksClientError;
use solana_banks_interface::TransactionStatus;
use solana_banks_interface::{
    BanksTransactionResultWithMetadata, BanksTransactionResultWithSimulation,
};
use solana_sdk::account::Account;
use solana_sdk::clock::Slot;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::hash::Hash;
use solana_sdk::message::Message;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::Signature;
use solana_sdk::sysvar::Sysvar;
use solana_sdk::transaction::{self, VersionedTransaction};

#[derive(Clone)]
pub struct BanksClient {
    client: AsyncBanksClient,
    rt: Arc<tokio::runtime::Runtime>,
}

impl From<AsyncBanksClient> for BanksClient {
    fn from(client: AsyncBanksClient) -> Self {
        Self::new(client)
    }
}

impl From<BanksClient> for AsyncBanksClient {
    fn from(client: BanksClient) -> Self {
        client.client
    }
}

impl BanksClient {
    pub fn new(client: AsyncBanksClient) -> Self {
        Self::with_runtime(client, Arc::new(tokio::runtime::Runtime::new().unwrap()))
    }

    pub fn with_runtime(client: AsyncBanksClient, rt: Arc<tokio::runtime::Runtime>) -> Self {
        Self { client, rt }
    }

    pub fn send_transaction_with_context(
        &mut self,
        ctx: Context,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<(), BanksClientError> {
        self.rt
            .block_on(self.client.send_transaction_with_context(ctx, transaction))
    }

    pub fn get_transaction_status_with_context(
        &mut self,
        ctx: Context,
        signature: Signature,
    ) -> Result<Option<TransactionStatus>, BanksClientError> {
        self.rt.block_on(
            self.client
                .get_transaction_status_with_context(ctx, signature),
        )
    }

    pub fn get_slot_with_context(
        &mut self,
        ctx: Context,
        commitment: CommitmentLevel,
    ) -> Result<Slot, BanksClientError> {
        self.rt
            .block_on(self.client.get_slot_with_context(ctx, commitment))
    }

    pub fn get_block_height_with_context(
        &mut self,
        ctx: Context,
        commitment: CommitmentLevel,
    ) -> Result<Slot, BanksClientError> {
        self.rt
            .block_on(self.client.get_block_height_with_context(ctx, commitment))
    }

    pub fn process_transaction_with_commitment_and_context(
        &mut self,
        ctx: Context,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<Option<transaction::Result<()>>, BanksClientError> {
        self.rt
            .block_on(self.client.process_transaction_with_commitment_and_context(
                ctx,
                transaction,
                commitment,
            ))
    }

    pub fn process_transaction_with_preflight_and_commitment_and_context(
        &mut self,
        ctx: Context,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        self.rt.block_on(
            self.client
                .process_transaction_with_preflight_and_commitment_and_context(
                    ctx,
                    transaction,
                    commitment,
                ),
        )
    }

    pub fn process_transaction_with_metadata_and_context(
        &mut self,
        ctx: Context,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        self.rt.block_on(
            self.client
                .process_transaction_with_metadata_and_context(ctx, transaction),
        )
    }

    pub fn simulate_transaction_with_commitment_and_context(
        &mut self,
        ctx: Context,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        self.rt.block_on(
            self.client
                .simulate_transaction_with_commitment_and_context(ctx, transaction, commitment),
        )
    }

    pub fn get_account_with_commitment_and_context(
        &mut self,
        ctx: Context,
        address: Pubkey,
        commitment: CommitmentLevel,
    ) -> Result<Option<Account>, BanksClientError> {
        self.rt.block_on(
            self.client
                .get_account_with_commitment_and_context(ctx, address, commitment),
        )
    }

    pub fn send_transaction(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<(), BanksClientError> {
        self.rt.block_on(self.client.send_transaction(transaction))
    }

    pub fn get_sysvar<T: Sysvar>(&mut self) -> Result<T, BanksClientError> {
        self.rt.block_on(self.client.get_sysvar())
    }

    pub fn get_rent(&mut self) -> Result<Rent, BanksClientError> {
        self.rt.block_on(self.client.get_rent())
    }

    pub fn process_transaction_with_commitment(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<(), BanksClientError> {
        self.rt.block_on(
            self.client
                .process_transaction_with_commitment(transaction, commitment),
        )
    }

    pub fn process_transaction_with_metadata(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<BanksTransactionResultWithMetadata, BanksClientError> {
        self.rt
            .block_on(self.client.process_transaction_with_metadata(transaction))
    }

    pub fn process_transaction_with_preflight_and_commitment(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<(), BanksClientError> {
        self.rt.block_on(
            self.client
                .process_transaction_with_preflight_and_commitment(transaction, commitment),
        )
    }

    pub fn process_transaction_with_preflight(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<(), BanksClientError> {
        self.rt
            .block_on(self.client.process_transaction_with_preflight(transaction))
    }

    pub fn process_transaction(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<(), BanksClientError> {
        self.rt
            .block_on(self.client.process_transaction(transaction))
    }

    pub fn process_transactions_with_commitment<T: Into<VersionedTransaction>>(
        &mut self,
        transactions: Vec<T>,
        commitment: CommitmentLevel,
    ) -> Result<(), BanksClientError> {
        self.rt.block_on(
            self.client
                .process_transactions_with_commitment(transactions, commitment),
        )
    }

    pub fn process_transactions<'a, T: Into<VersionedTransaction> + 'a>(
        &'a mut self,
        transactions: Vec<T>,
    ) -> Result<(), BanksClientError> {
        self.rt
            .block_on(self.client.process_transactions(transactions))
    }

    pub fn simulate_transaction_with_commitment(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
        commitment: CommitmentLevel,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        self.rt.block_on(
            self.client
                .simulate_transaction_with_commitment(transaction, commitment),
        )
    }

    pub fn simulate_transaction(
        &mut self,
        transaction: impl Into<VersionedTransaction>,
    ) -> Result<BanksTransactionResultWithSimulation, BanksClientError> {
        self.rt
            .block_on(self.client.simulate_transaction(transaction))
    }

    pub fn get_root_slot(&mut self) -> Result<Slot, BanksClientError> {
        self.rt.block_on(self.client.get_root_slot())
    }

    pub fn get_root_block_height(&mut self) -> Result<Slot, BanksClientError> {
        self.rt.block_on(self.client.get_root_block_height())
    }

    pub fn get_account_with_commitment(
        &mut self,
        address: Pubkey,
        commitment: CommitmentLevel,
    ) -> Result<Option<Account>, BanksClientError> {
        self.rt
            .block_on(self.client.get_account_with_commitment(address, commitment))
    }

    pub fn get_account(&mut self, address: Pubkey) -> Result<Option<Account>, BanksClientError> {
        self.rt.block_on(self.client.get_account(address))
    }

    pub fn get_packed_account_data<T: Pack>(
        &mut self,
        address: Pubkey,
    ) -> Result<T, BanksClientError> {
        self.rt
            .block_on(self.client.get_packed_account_data(address))
    }

    pub fn get_account_data_with_borsh<T: BorshDeserialize>(
        &mut self,
        address: Pubkey,
    ) -> Result<T, BanksClientError> {
        self.rt
            .block_on(self.client.get_account_data_with_borsh(address))
    }

    pub fn get_balance_with_commitment(
        &mut self,
        address: Pubkey,
        commitment: CommitmentLevel,
    ) -> Result<u64, BanksClientError> {
        self.rt
            .block_on(self.client.get_balance_with_commitment(address, commitment))
    }

    pub fn get_balance(&mut self, address: Pubkey) -> Result<u64, BanksClientError> {
        self.rt.block_on(self.client.get_balance(address))
    }

    pub fn get_transaction_status(
        &mut self,
        signature: Signature,
    ) -> Result<Option<TransactionStatus>, BanksClientError> {
        self.rt
            .block_on(self.client.get_transaction_status(signature))
    }

    pub fn get_transaction_statuses(
        &mut self,
        signatures: Vec<Signature>,
    ) -> Result<Vec<Option<TransactionStatus>>, BanksClientError> {
        self.rt
            .block_on(self.client.get_transaction_statuses(signatures))
    }

    pub fn get_latest_blockhash(&mut self) -> Result<Hash, BanksClientError> {
        self.rt.block_on(self.client.get_latest_blockhash())
    }

    pub fn get_latest_blockhash_with_commitment(
        &mut self,
        commitment: CommitmentLevel,
    ) -> Result<Option<(Hash, u64)>, BanksClientError> {
        self.rt
            .block_on(self.client.get_latest_blockhash_with_commitment(commitment))
    }

    pub fn get_latest_blockhash_with_commitment_and_context(
        &mut self,
        ctx: Context,
        commitment: CommitmentLevel,
    ) -> Result<Option<(Hash, u64)>, BanksClientError> {
        self.rt.block_on(
            self.client
                .get_latest_blockhash_with_commitment_and_context(ctx, commitment),
        )
    }

    pub fn get_fee_for_message(
        &mut self,
        message: Message,
    ) -> Result<Option<u64>, BanksClientError> {
        self.rt.block_on(self.client.get_fee_for_message(message))
    }

    pub fn get_fee_for_message_with_commitment(
        &mut self,
        message: Message,
        commitment: CommitmentLevel,
    ) -> Result<Option<u64>, BanksClientError> {
        self.rt.block_on(
            self.client
                .get_fee_for_message_with_commitment(message, commitment),
        )
    }

    pub fn get_fee_for_message_with_commitment_and_context(
        &mut self,
        ctx: Context,
        message: Message,
        commitment: CommitmentLevel,
    ) -> Result<Option<u64>, BanksClientError> {
        self.rt.block_on(
            self.client
                .get_fee_for_message_with_commitment_and_context(ctx, message, commitment),
        )
    }
}
