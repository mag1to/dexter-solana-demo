use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;

use anchor_lang::AccountDeserialize;
use anchor_spl::token::{spl_token, Mint, TokenAccount};

use dexter_client_anchor::{AnchorAccount, AnchorGetter};
use dexter_client_api::base::executor::ProcessTransaction;
use dexter_client_api::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetProgramAccounts, Memcmp,
    ProgramAccountsFilter,
};
use dexter_client_api::base::setter::{HasRent, SetAccount};
use dexter_client_api::errors::ClientResult;
use dexter_client_api::execution::ExecutionOutput;
use dexter_client_api::exts::executor::CompilingProcessTransaction;
use dexter_client_api::Client;
use dexter_client_sys::pack::PackingSetter;

pub trait TokenGetter: Client {
    fn get_mint(&self, mint: &Pubkey) -> ClientResult<Option<AnchorAccount<Mint>>>
    where
        Self: GetAccount,
    {
        self.get_anchor_account(mint)
    }

    fn try_get_mint(&self, mint: &Pubkey) -> ClientResult<AnchorAccount<Mint>>
    where
        Self: GetAccount,
    {
        self.try_get_anchor_account(mint)
    }

    fn get_token_account(
        &self,
        token_account: &Pubkey,
    ) -> ClientResult<Option<AnchorAccount<TokenAccount>>>
    where
        Self: GetAccount,
    {
        self.get_anchor_account(token_account)
    }

    fn try_get_token_account(
        &self,
        token_account: &Pubkey,
    ) -> ClientResult<AnchorAccount<TokenAccount>>
    where
        Self: GetAccount,
    {
        self.try_get_anchor_account(token_account)
    }

    fn get_token_account_balance(&self, token_account: &Pubkey) -> ClientResult<Option<u64>>
    where
        Self: GetAccount,
    {
        self.get_token_account(token_account)
            .map(|ta| ta.map(|ta| ta.amount))
    }

    fn try_get_token_account_balance(&self, token_account: &Pubkey) -> ClientResult<u64>
    where
        Self: GetAccount,
    {
        self.try_get_token_account(token_account)
            .map(|ta| ta.amount)
    }

    fn get_token_accounts_for_owner(
        &self,
        owner: &Pubkey,
    ) -> ClientResult<Vec<(Pubkey, TokenAccount)>>
    where
        Self: GetProgramAccounts,
    {
        let filters = vec![
            ProgramAccountsFilter::DataSize(TokenAccount::LEN as u64),
            ProgramAccountsFilter::Memcmp(Memcmp::new_base58_encoded(32, owner.as_ref())),
        ];

        let accounts = self.get_program_accounts(&spl_token::id(), Some(filters))?;
        let token_accounts = accounts
            .into_iter()
            .map(|(pubkey, account)| {
                (
                    pubkey,
                    TokenAccount::try_deserialize(&mut account.data.as_slice()).unwrap(),
                )
            })
            .collect();

        Ok(token_accounts)
    }

    fn get_mint_supply(&self, mint: &Pubkey) -> ClientResult<Option<u64>>
    where
        Self: GetAccount,
    {
        self.get_mint(mint).map(|tm| tm.map(|tm| tm.supply))
    }

    fn try_get_mint_supply(&self, mint: &Pubkey) -> ClientResult<u64>
    where
        Self: GetAccount,
    {
        self.try_get_mint(mint).map(|tm| tm.supply)
    }
}

impl<C: ?Sized + Client> TokenGetter for C {}

pub trait TokenInstruction: Client {
    fn build_initialize_mint(
        &self,
        mint: &Pubkey,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
    ) -> Instruction {
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint,
            mint_authority,
            freeze_authority,
            decimals,
        )
        .unwrap()
    }

    fn build_initialize_account(
        &self,
        token_account: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Instruction {
        spl_token::instruction::initialize_account(&spl_token::id(), token_account, mint, owner)
            .unwrap()
    }

    fn build_mint_to(
        &self,
        token_account: &Pubkey,
        mint: &Pubkey,
        mint_authority: &Pubkey,
        amount: u64,
    ) -> Instruction {
        spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            token_account,
            mint_authority,
            &[],
            amount,
        )
        .unwrap()
    }

    fn build_transfer(
        &self,
        source: &Pubkey,
        destination: &Pubkey,
        source_authority: &Pubkey,
        amount: u64,
    ) -> Instruction {
        spl_token::instruction::transfer(
            &spl_token::id(),
            source,
            destination,
            source_authority,
            &[],
            amount,
        )
        .unwrap()
    }

    fn build_create_and_initialize_mint(
        &self,
        payer: &Pubkey,
        mint: &Pubkey,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
    ) -> ClientResult<[Instruction; 2]>
    where
        Self: GetMinimumBalanceForRentExemption,
    {
        let instructions = [
            system_instruction::create_account(
                payer,
                mint,
                self.get_minimum_balance_for_rent_exemption(Mint::LEN)?,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            self.build_initialize_mint(mint, mint_authority, freeze_authority, decimals),
        ];

        Ok(instructions)
    }

    fn build_create_and_initialize_token_account(
        &self,
        payer: &Pubkey,
        token_account: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> ClientResult<[Instruction; 2]>
    where
        Self: GetMinimumBalanceForRentExemption,
    {
        let instructions = [
            system_instruction::create_account(
                payer,
                token_account,
                self.get_minimum_balance_for_rent_exemption(TokenAccount::LEN)?,
                TokenAccount::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                token_account,
                mint,
                owner,
            )
            .unwrap(),
        ];

        Ok(instructions)
    }
}

impl<C: ?Sized + Client> TokenInstruction for C {}

pub trait TokenProcessor: Client {
    fn process_create_and_initialize_mint(
        &self,
        payer: &impl Signer,
        mint: &impl Signer,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetMinimumBalanceForRentExemption
            + GetLatestBlockhash
            + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = self.build_create_and_initialize_mint(
            &payer.pubkey(),
            &mint.pubkey(),
            mint_authority,
            freeze_authority,
            decimals,
        )?;
        let signers: Vec<&dyn Signer> = vec![payer, mint];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_create_and_initialize_token_account(
        &self,
        payer: &impl Signer,
        token_account: &impl Signer,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetMinimumBalanceForRentExemption
            + GetLatestBlockhash
            + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = self.build_create_and_initialize_token_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            mint,
            owner,
        )?;
        let signers: Vec<&dyn Signer> = vec![payer, token_account];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_mint_to(
        &self,
        payer: &impl Signer,
        token_account: &Pubkey,
        mint: &Pubkey,
        mint_authority: &impl Signer,
        amount: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instructions =
            [self.build_mint_to(token_account, mint, &mint_authority.pubkey(), amount)];
        let signers: Vec<&dyn Signer> = vec![payer, mint_authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_transfer_token(
        &self,
        payer: &impl Signer,
        source: &Pubkey,
        destination: &Pubkey,
        source_authority: &impl Signer,
        amount: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instructions =
            [self.build_transfer(source, destination, &source_authority.pubkey(), amount)];
        let signers: Vec<&dyn Signer> = vec![payer, source_authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_wrap_native(
        &self,
        payer: &impl Signer,
        source: &impl Signer,
        destination: &Pubkey,
        lamports: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = [
            system_instruction::transfer(&source.pubkey(), destination, lamports),
            spl_token::instruction::sync_native(&spl_token::id(), destination).unwrap(),
        ];
        let signers: Vec<&dyn Signer> = vec![payer, source];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }
}

impl<C: ?Sized + Client> TokenProcessor for C {}

pub trait TokenSetter: Client {
    fn set_mint(
        &mut self,
        mint_pk: Pubkey,
        mint_authority: Option<Pubkey>,
        supply: u64,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
    ) -> AnchorAccount<Mint>
    where
        Self: SetAccount + HasRent,
    {
        let mint = spl_token::state::Mint {
            mint_authority: mint_authority.into(),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: freeze_authority.into(),
        };

        let account = self.packing_set_account(
            mint_pk,
            self.minimum_balance_for_rent_exemption(Mint::LEN),
            spl_token::id(),
            &mint,
        );

        AnchorAccount::try_from_account(mint_pk, account).unwrap()
    }

    fn set_token_account(
        &mut self,
        token_account_pk: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> AnchorAccount<TokenAccount>
    where
        Self: SetAccount + HasRent,
    {
        let rent_exempt = self.minimum_balance_for_rent_exemption(TokenAccount::LEN);

        let (lamports, is_native) = if spl_token::native_mint::check_id(&mint) {
            (rent_exempt + amount, Some(rent_exempt))
        } else {
            (rent_exempt, None)
        };

        let token_account = spl_token::state::Account {
            mint,
            owner,
            amount,
            delegate: None.into(),
            state: spl_token::state::AccountState::Initialized,
            is_native: is_native.into(),
            delegated_amount: 0,
            close_authority: None.into(),
        };

        let account =
            self.packing_set_account(token_account_pk, lamports, spl_token::id(), &token_account);

        AnchorAccount::try_from_account(token_account_pk, account).unwrap()
    }
}

impl<C: ?Sized + Client> TokenSetter for C {}
