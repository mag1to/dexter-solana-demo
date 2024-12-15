use std::collections::BTreeSet;

use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::signer::SignerError;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::VersionedTransaction;

use crate::base::executor::{ProcessTransaction, SimulateTransaction};
use crate::base::getter::GetLatestBlockhash;
use crate::client::Client;
use crate::errors::ClientResult;

pub trait CompileTransaction: Client + GetLatestBlockhash {
    fn compile_transaction<S>(
        &self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &S,
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> ClientResult<VersionedTransaction>
    where
        S: Signers + ?Sized,
    {
        let recent_blockhash = self.get_latest_blockhash()?;

        let message = Message::try_compile(
            payer,
            instructions,
            address_lookup_table_accounts,
            recent_blockhash,
        )?;

        let signers = PrimeSigners::new(signers)?;

        let transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), &signers)?;

        Ok(transaction)
    }
}

impl<C: ?Sized + Client + GetLatestBlockhash> CompileTransaction for C {}

pub trait CompilingProcessTransaction<T>:
    Client + GetLatestBlockhash + ProcessTransaction<T>
{
    fn compiling_process_transaction<S>(
        &self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &S,
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> ClientResult<T>
    where
        S: Signers + ?Sized,
    {
        let transaction =
            self.compile_transaction(instructions, payer, signers, address_lookup_table_accounts)?;
        self.process_transaction(transaction)
    }
}

impl<T, C: ?Sized + Client + GetLatestBlockhash + ProcessTransaction<T>>
    CompilingProcessTransaction<T> for C
{
}

pub trait CompilingSimulateTransaction<T>:
    Client + GetLatestBlockhash + SimulateTransaction<T>
{
    fn compiling_simulate_transaction<S>(
        &self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &S,
        address_lookup_table_accounts: &[AddressLookupTableAccount],
    ) -> ClientResult<T>
    where
        S: Signers + ?Sized,
    {
        let transaction =
            self.compile_transaction(instructions, payer, signers, address_lookup_table_accounts)?;
        self.simulate_transaction(transaction)
    }
}

impl<T, C: ?Sized + Client + GetLatestBlockhash + SimulateTransaction<T>>
    CompilingSimulateTransaction<T> for C
{
}

struct PrimeSigners<'a, S: Signers + ?Sized> {
    signers: &'a S,
    indexes: Vec<usize>,
}

impl<'a, S: Signers + ?Sized> PrimeSigners<'a, S> {
    fn new(signers: &'a S) -> Result<Self, SignerError> {
        let signer_keys = signers.try_pubkeys()?;

        let mut seen = BTreeSet::new();
        let mut indexes = Vec::new();
        for (i, key) in signer_keys.into_iter().enumerate() {
            if !seen.insert(key) {
                continue;
            }
            indexes.push(i);
        }

        Ok(Self { signers, indexes })
    }
}

impl<'a, S: Signers + ?Sized> Signers for PrimeSigners<'a, S> {
    fn pubkeys(&self) -> Vec<Pubkey> {
        let pubkeys = self.signers.pubkeys();
        self.indexes.iter().map(|&i| pubkeys[i]).collect()
    }

    fn try_pubkeys(&self) -> Result<Vec<Pubkey>, SignerError> {
        let pubkeys = self.signers.try_pubkeys()?;
        Ok(self.indexes.iter().map(|&i| pubkeys[i]).collect())
    }

    fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
        let signatures = self.signers.sign_message(message);
        self.indexes.iter().map(|&i| signatures[i]).collect()
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Vec<Signature>, SignerError> {
        let signatures = self.signers.try_sign_message(message)?;
        Ok(self.indexes.iter().map(|&i| signatures[i]).collect())
    }

    fn is_interactive(&self) -> bool {
        self.signers.is_interactive()
    }
}
