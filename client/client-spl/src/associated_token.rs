use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;

use anchor_spl::associated_token::{get_associated_token_address, spl_associated_token_account};
use anchor_spl::token::{spl_token, TokenAccount};

use dexter_client_anchor::AnchorAccount;
use dexter_client_api::base::executor::ProcessTransaction;
use dexter_client_api::base::getter::{GetAccount, GetLatestBlockhash};
use dexter_client_api::base::setter::{HasRent, SetAccount};
use dexter_client_api::errors::ClientResult;
use dexter_client_api::execution::ExecutionOutput;
use dexter_client_api::exts::executor::CompilingProcessTransaction;
use dexter_client_api::Client;

use crate::token::{TokenGetter, TokenSetter};

const COMPUTE_BUDGET_UNITS: u32 = 50_000;
const COMPUTE_BUDGET_PRICE: u64 = 1_000_000;

fn with_compute_budget(instruction: Instruction) -> [Instruction; 3] {
    [
        ComputeBudgetInstruction::set_compute_unit_limit(COMPUTE_BUDGET_UNITS),
        ComputeBudgetInstruction::set_compute_unit_price(COMPUTE_BUDGET_PRICE),
        instruction,
    ]
}

pub trait AssociatedTokenGetter: Client {
    fn get_associated_token_address(&self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        get_associated_token_address(owner, mint)
    }

    fn get_associated_token_account(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> ClientResult<Option<AnchorAccount<TokenAccount>>>
    where
        Self: GetAccount,
    {
        self.get_token_account(&self.get_associated_token_address(owner, mint))
    }

    fn try_get_associated_token_account(
        &self,
        owner: &Pubkey,
        mint: &Pubkey,
    ) -> ClientResult<AnchorAccount<TokenAccount>>
    where
        Self: GetAccount,
    {
        self.try_get_token_account(&self.get_associated_token_address(owner, mint))
    }
}

impl<C: ?Sized + Client> AssociatedTokenGetter for C {}

pub trait AssociatedTokenProcessor: Client {
    fn process_create_associated_token_account(
        &self,
        payer: &impl Signer,
        owner: Pubkey,
        mint: Pubkey,
    ) -> ClientResult<ExecutionOutput>
    where
        Self: GetLatestBlockhash + ProcessTransaction<ExecutionOutput>,
    {
        let instructions = with_compute_budget(
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer.pubkey(),
                &owner,
                &mint,
                &spl_token::id(),
            ),
        );
        let signers: Vec<&dyn Signer> = vec![payer];
        self.compiling_process_transaction(&instructions, &payer.pubkey(), &signers, &[])
    }
}

impl<C: ?Sized + Client> AssociatedTokenProcessor for C {}

pub trait AssociatedTokenSetter: Client {
    fn set_associated_token_account(
        &mut self,
        mint: Pubkey,
        owner: Pubkey,
        amount: u64,
    ) -> AnchorAccount<TokenAccount>
    where
        Self: SetAccount + HasRent,
    {
        self.set_token_account(
            self.get_associated_token_address(&owner, &mint),
            mint,
            owner,
            amount,
        )
    }
}

impl<C: ?Sized + Client> AssociatedTokenSetter for C {}
