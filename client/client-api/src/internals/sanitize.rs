use solana_sdk::address_lookup_table;
use solana_sdk::address_lookup_table::state::AddressLookupTable;
use solana_sdk::clock::Slot;
use solana_sdk::message::v0::LoadedAddresses;
use solana_sdk::message::SimpleAddressLoader;
use solana_sdk::sysvar::slot_hashes::{self, SlotHashes};
use solana_sdk::transaction::{MessageHash, SanitizedTransaction, VersionedTransaction};

use crate::base::getter::GetMultipleAccounts;
use crate::client::Client;
use crate::errors::{AddressLookupError, ClientError, ClientResult};

pub trait SanitizeTransaction: Client + GetMultipleAccounts {
    fn sanitize_transaction(
        &self,
        transaction: VersionedTransaction,
    ) -> ClientResult<SanitizedTransaction> {
        let loaded_addresses =
            if let Some(address_table_lookups) = transaction.message.address_table_lookups() {
                let table_account_keys: Vec<_> = address_table_lookups
                    .iter()
                    .map(|lookup| lookup.account_key)
                    .collect();

                let (slot_hashes, table_accounts) = {
                    let account_keys: Vec<_> = table_account_keys
                        .iter()
                        .copied()
                        .chain(std::iter::once(slot_hashes::id()))
                        .collect();
                    let mut accounts = self.get_multiple_accounts(&account_keys)?;

                    let slot_hashes: SlotHashes = {
                        let account = accounts
                            .pop()
                            .unwrap()
                            .ok_or(ClientError::AccountNotFound(slot_hashes::id()))?;
                        bincode::deserialize(&account.data)
                            .map_err(|_| ClientError::AccountDidNotDeserialize(slot_hashes::id()))?
                    };

                    (slot_hashes, accounts)
                };

                let current_slot = Slot::MAX;

                let mut loaded = Vec::with_capacity(address_table_lookups.len());
                for (address_table_lookup, table_account_opt) in
                    address_table_lookups.iter().zip(table_accounts)
                {
                    let table_account =
                        table_account_opt.ok_or(AddressLookupError::LookupTableAccountNotFound)?;

                    if table_account.owner != address_lookup_table::program::id() {
                        return Err(AddressLookupError::InvalidAccountOwner.into());
                    }

                    let lookup_table = AddressLookupTable::deserialize(&table_account.data)
                        .map_err(|_| AddressLookupError::InvalidAccountData)?;

                    loaded.push(LoadedAddresses {
                        writable: lookup_table.lookup(
                            current_slot,
                            &address_table_lookup.writable_indexes,
                            &slot_hashes,
                        )?,
                        readonly: lookup_table.lookup(
                            current_slot,
                            &address_table_lookup.readonly_indexes,
                            &slot_hashes,
                        )?,
                    });
                }

                loaded.into_iter().collect()
            } else {
                LoadedAddresses::default()
            };

        let sanitized_transaction = SanitizedTransaction::try_create(
            transaction,
            MessageHash::Compute,
            Some(false),
            SimpleAddressLoader::Enabled(loaded_addresses),
        )?;

        Ok(sanitized_transaction)
    }
}

impl<C: ?Sized + Client + GetMultipleAccounts> SanitizeTransaction for C {}
