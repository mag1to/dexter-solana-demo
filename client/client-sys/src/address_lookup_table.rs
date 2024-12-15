use std::borrow::Cow;

use solana_sdk::address_lookup_table;
use solana_sdk::address_lookup_table::instruction::{
    close_lookup_table, create_lookup_table, deactivate_lookup_table, extend_lookup_table,
};
use solana_sdk::address_lookup_table::state::AddressLookupTable;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use dexter_client_api::base::executor::ProcessTransaction;
use dexter_client_api::base::getter::{
    GetAccount, GetLatestBlockhash, GetProgramAccounts, Memcmp, ProgramAccountsFilter,
};
use dexter_client_api::errors::{ClientError, ClientResult};
use dexter_client_api::execution::ExecutionOutput;
use dexter_client_api::exts::executor::CompilingProcessTransaction;
use dexter_client_api::Client;

use crate::sysvar::SysvarGetter;

const LOOKUP_TABLE_META_AUTHORITY_OFFSET: usize = 22;
const RECENT_SLOT_INDEX: usize = 1;

const COMPUTE_BUDGET_UNITS: u32 = 2_000;
const COMPUTE_BUDGET_PRICE: u64 = 1_000_000;

fn with_compute_budget(instruction: Instruction) -> [Instruction; 3] {
    [
        ComputeBudgetInstruction::set_compute_unit_limit(COMPUTE_BUDGET_UNITS),
        ComputeBudgetInstruction::set_compute_unit_price(COMPUTE_BUDGET_PRICE),
        instruction,
    ]
}

pub trait AddressLookupTableGetter: Client {
    fn get_address_lookup_table(&self, pubkey: &Pubkey) -> ClientResult<Option<AddressLookupTable>>
    where
        Self: GetAccount,
    {
        let Some(account) = self.get_account(pubkey)? else {
            return Ok(None);
        };

        let lookup_table = AddressLookupTable::deserialize(&account.data)
            .map_err(|_| ClientError::AccountDidNotDeserialize(*pubkey))?;

        Ok(Some(convert_to_owned(lookup_table)))
    }

    fn get_address_lookup_tables_for_authority(
        &self,
        authority: &Pubkey,
    ) -> ClientResult<Vec<(Pubkey, AddressLookupTable)>>
    where
        Self: GetProgramAccounts,
    {
        let filters = vec![ProgramAccountsFilter::Memcmp(Memcmp::new_base58_encoded(
            LOOKUP_TABLE_META_AUTHORITY_OFFSET,
            authority.as_ref(),
        ))];

        let accounts =
            self.get_program_accounts(&address_lookup_table::program::id(), Some(filters))?;

        let lookup_tables = accounts
            .into_iter()
            .map(|(pubkey, account)| {
                let lookup_table = AddressLookupTable::deserialize(&account.data).unwrap();
                (pubkey, convert_to_owned(lookup_table))
            })
            .collect();

        Ok(lookup_tables)
    }
}

impl<C: ?Sized + Client> AddressLookupTableGetter for C {}

pub trait AddressLookupTableProcessor: Client {
    fn process_create_lookup_table(
        &self,
        payer: &impl Signer,
        authority: Pubkey,
    ) -> ClientResult<Pubkey>
    where
        Self: GetAccount + GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let (recent_slot, _) = self.try_get_sysvar_slothashes()?.slot_hashes()[RECENT_SLOT_INDEX];

        let (instruction, lookup_table_address) =
            create_lookup_table(authority, payer.pubkey(), recent_slot);
        let instructions = with_compute_budget(instruction);

        let signers = vec![payer];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])?;

        Ok(lookup_table_address)
    }

    fn process_extend_lookup_table(
        &self,
        payer: &impl Signer,
        authority: &impl Signer,
        lookup_table_address: Pubkey,
        new_addresses: Vec<Pubkey>,
    ) -> ClientResult<()>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instruction = extend_lookup_table(
            lookup_table_address,
            authority.pubkey(),
            Some(payer.pubkey()),
            new_addresses,
        );
        let instructions = with_compute_budget(instruction);

        let signers: Vec<&dyn Signer> = if payer.pubkey() == authority.pubkey() {
            vec![payer]
        } else {
            vec![payer, authority]
        };
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])?;

        Ok(())
    }

    fn process_deactivate_lookup_table(
        &self,
        payer: &impl Signer,
        authority: &impl Signer,
        lookup_table_address: Pubkey,
    ) -> ClientResult<()>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instruction = deactivate_lookup_table(lookup_table_address, authority.pubkey());
        let instructions = with_compute_budget(instruction);

        let signers: Vec<&dyn Signer> = vec![payer, authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])?;

        Ok(())
    }

    fn process_close_lookup_table(
        &self,
        payer: &impl Signer,
        authority: &impl Signer,
        lookup_table_address: Pubkey,
        recipient_address: Pubkey,
    ) -> ClientResult<()>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instruction =
            close_lookup_table(lookup_table_address, authority.pubkey(), recipient_address);
        let instructions = with_compute_budget(instruction);

        let signers: Vec<&dyn Signer> = vec![payer, authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])?;

        Ok(())
    }
}

impl<C: ?Sized + Client> AddressLookupTableProcessor for C {}

fn convert_to_owned(lookup_table: AddressLookupTable<'_>) -> AddressLookupTable<'static> {
    let AddressLookupTable { meta, addresses } = lookup_table;
    AddressLookupTable {
        meta,
        addresses: Cow::Owned(addresses.into_owned()),
    }
}
