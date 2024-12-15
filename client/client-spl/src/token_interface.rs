use solana_sdk::account::ReadableAccount;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;

use anchor_lang::Key;
use anchor_spl::token::spl_token;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use dexter_client_anchor::{AnchorAccount, AnchorGetter};
use dexter_client_api::base::executor::ProcessTransaction;
use dexter_client_api::base::getter::{
    GetAccount, GetLatestBlockhash, GetMinimumBalanceForRentExemption, GetMultipleAccounts,
};
use dexter_client_api::base::setter::{HasRent, SetAccount};
use dexter_client_api::errors::ClientResult;
use dexter_client_api::execution::ExecutionOutput;
use dexter_client_api::exts::executor::CompilingProcessTransaction;
use dexter_client_api::Client;
use dexter_client_sys::pack::PackingSetter;

const MINT_LEN: usize = spl_token_2022::state::Mint::LEN;
const TOKEN_ACCOUNT_LEN: usize = spl_token_2022::state::Account::LEN;

pub trait TokenInterfaceGetter: Client {
    fn get_token_program_id(&self, mint: &Pubkey) -> ClientResult<Option<Pubkey>>
    where
        Self: GetAccount,
    {
        self.get_mint(mint)
            .map(|mint| mint.map(|mint| *ReadableAccount::owner(&mint)))
    }

    fn try_get_token_program_id(&self, mint: &Pubkey) -> ClientResult<Pubkey>
    where
        Self: GetAccount,
    {
        self.try_get_mint(mint)
            .map(|mint| *ReadableAccount::owner(&mint))
    }

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

    fn get_mints(&self, mints: &[Pubkey]) -> ClientResult<Vec<Option<AnchorAccount<Mint>>>>
    where
        Self: GetMultipleAccounts,
    {
        self.get_anchor_multiple_accounts(mints)
    }

    fn try_get_mints(&self, mints: &[Pubkey]) -> ClientResult<Vec<AnchorAccount<Mint>>>
    where
        Self: GetMultipleAccounts,
    {
        self.try_get_anchor_multiple_accounts(mints)
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

impl<C: ?Sized + Client> TokenInterfaceGetter for C {}

pub trait TokenInterfaceInstruction: Client {
    fn build_initialize_mint(
        &self,
        token_program_id: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
        decimals: u8,
    ) -> Instruction {
        spl_token_2022::instruction::initialize_mint(
            &token_program_id,
            &mint,
            &mint_authority,
            freeze_authority.as_ref(),
            decimals,
        )
        .unwrap()
    }

    fn build_initialize_account(
        &self,
        token_program_id: Pubkey,
        token_account: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
    ) -> Instruction {
        spl_token_2022::instruction::initialize_account(
            &token_program_id,
            &token_account,
            &mint,
            &owner,
        )
        .unwrap()
    }

    fn build_mint_to(
        &self,
        token_program_id: Pubkey,
        token_account: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        amount: u64,
    ) -> Instruction {
        spl_token_2022::instruction::mint_to(
            &token_program_id,
            &mint,
            &token_account,
            &mint_authority,
            &[],
            amount,
        )
        .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn build_transfer_checked(
        &self,
        token_program_id: Pubkey,
        source: Pubkey,
        mint: Pubkey,
        destination: Pubkey,
        authority: Pubkey,
        signers: &[Pubkey],
        amount: u64,
        decimals: u8,
    ) -> Instruction {
        spl_token_2022::instruction::transfer_checked(
            &token_program_id,
            &source,
            &mint,
            &destination,
            &authority,
            &signers.iter().collect::<Vec<_>>(),
            amount,
            decimals,
        )
        .unwrap()
    }

    fn build_create_and_initialize_mint(
        &self,
        payer: Pubkey,
        token_program_id: Pubkey,
        mint: Pubkey,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
        decimals: u8,
    ) -> ClientResult<[Instruction; 2]>
    where
        Self: GetMinimumBalanceForRentExemption,
    {
        let instructions = [
            system_instruction::create_account(
                &payer,
                &mint,
                self.get_minimum_balance_for_rent_exemption(MINT_LEN)?,
                MINT_LEN as u64,
                &token_program_id,
            ),
            self.build_initialize_mint(
                token_program_id,
                mint,
                mint_authority,
                freeze_authority,
                decimals,
            ),
        ];

        Ok(instructions)
    }

    fn build_create_and_initialize_token_account(
        &self,
        payer: Pubkey,
        token_program_id: Pubkey,
        token_account: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
    ) -> ClientResult<[Instruction; 2]>
    where
        Self: GetMinimumBalanceForRentExemption,
    {
        let instructions = [
            system_instruction::create_account(
                &payer,
                &token_account,
                self.get_minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_LEN)?,
                TOKEN_ACCOUNT_LEN as u64,
                &token_program_id,
            ),
            spl_token_2022::instruction::initialize_account(
                &token_program_id,
                &token_account,
                &mint,
                &owner,
            )
            .unwrap(),
        ];

        Ok(instructions)
    }
}

impl<C: ?Sized + Client> TokenInterfaceInstruction for C {}

pub trait TokenInterfaceProcessor: Client {
    fn process_create_and_initialize_mint(
        &self,
        payer: &impl Signer,
        token_program_id: Pubkey,
        mint: &impl Signer,
        mint_authority: Pubkey,
        freeze_authority: Option<Pubkey>,
        decimals: u8,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetMinimumBalanceForRentExemption
            + GetLatestBlockhash
            + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = self.build_create_and_initialize_mint(
            payer.pubkey(),
            token_program_id,
            mint.pubkey(),
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
        token_program_id: Pubkey,
        token_account: &impl Signer,
        mint: Pubkey,
        owner: Pubkey,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetMinimumBalanceForRentExemption
            + GetLatestBlockhash
            + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = self.build_create_and_initialize_token_account(
            payer.pubkey(),
            token_program_id,
            token_account.pubkey(),
            mint,
            owner,
        )?;
        let signers: Vec<&dyn Signer> = vec![payer, token_account];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_mint_to(
        &self,
        payer: &impl Signer,
        token_account: Pubkey,
        mint_authority: &impl Signer,
        amount: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetAccount + GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let token_account_account = self.try_get_token_account(&token_account)?;
        let token_program_id = *ReadableAccount::owner(&token_account_account);
        let mint = token_account_account.mint;

        let instructions = [self.build_mint_to(
            token_program_id,
            token_account,
            mint,
            mint_authority.pubkey(),
            amount,
        )];
        let signers: Vec<&dyn Signer> = vec![payer, mint_authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_transfer_checked(
        &self,
        payer: &impl Signer,
        source: Pubkey,
        destination: Pubkey,
        authority: &impl Signer,
        signers: &[Pubkey],
        amount: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetAccount + GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let source_account = self.try_get_token_account(&source)?;
        let mint_account = self.try_get_mint(&source_account.mint)?;
        let token_program_id = *ReadableAccount::owner(&mint_account);
        let decimals = mint_account.decimals;

        let instructions = [self.build_transfer_checked(
            token_program_id,
            source,
            mint_account.key(),
            destination,
            authority.pubkey(),
            signers,
            amount,
            decimals,
        )];
        let signers: Vec<&dyn Signer> = vec![payer, authority];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }

    fn process_wrap_native(
        &self,
        payer: &impl Signer,
        source: &impl Signer,
        destination: Pubkey,
        lamports: u64,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetAccount + GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let destination_account = self.try_get_token_account(&destination)?;
        let token_program_id = self.try_get_token_program_id(&destination_account.mint)?;

        let instructions = [
            system_instruction::transfer(&source.pubkey(), &destination, lamports),
            spl_token_2022::instruction::sync_native(&token_program_id, &destination).unwrap(),
        ];
        let signers: Vec<&dyn Signer> = vec![payer, source];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }
}

impl<C: ?Sized + Client> TokenInterfaceProcessor for C {}

pub trait TokenInterfaceSetter: Client {
    fn set_mint(
        &mut self,
        token_program_id: Pubkey,
        mint_pk: Pubkey,
        mint_authority: Option<Pubkey>,
        supply: u64,
        decimals: u8,
        freeze_authority: Option<Pubkey>,
    ) -> AnchorAccount<Mint>
    where
        Self: SetAccount + HasRent,
    {
        let mint = spl_token_2022::state::Mint {
            mint_authority: mint_authority.into(),
            supply,
            decimals,
            is_initialized: true,
            freeze_authority: freeze_authority.into(),
        };

        let account = self.packing_set_account(
            mint_pk,
            self.minimum_balance_for_rent_exemption(MINT_LEN),
            token_program_id,
            &mint,
        );

        AnchorAccount::try_from_account(mint_pk, account).unwrap()
    }

    fn set_token_account(
        &mut self,
        token_program_id: Pubkey,
        token_account_pk: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> AnchorAccount<TokenAccount>
    where
        Self: SetAccount + HasRent,
    {
        let rent_exempt = self.minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_LEN);

        let (lamports, is_native) = if spl_token::native_mint::check_id(&mint) {
            (rent_exempt + amount, Some(rent_exempt))
        } else {
            (rent_exempt, None)
        };

        let token_account = spl_token_2022::state::Account {
            mint,
            owner,
            amount,
            delegate: None.into(),
            state: spl_token_2022::state::AccountState::Initialized,
            is_native: is_native.into(),
            delegated_amount: 0,
            close_authority: None.into(),
        };

        let account =
            self.packing_set_account(token_account_pk, lamports, token_program_id, &token_account);

        AnchorAccount::try_from_account(token_account_pk, account).unwrap()
    }
}

impl<C: ?Sized + Client> TokenInterfaceSetter for C {}
