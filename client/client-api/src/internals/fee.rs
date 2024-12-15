use solana_program_runtime::compute_budget_processor::process_compute_budget_instructions;
use solana_program_runtime::prioritization_fee::{PrioritizationFeeDetails, PrioritizationFeeType};
use solana_sdk::transaction::SanitizedTransaction;

use crate::client::Client;
use crate::errors::ClientResult;

pub trait CalculatePrioritizationFee: Client {
    fn calculate_prioritization_fee(
        &self,
        sanitized_transaction: &SanitizedTransaction,
    ) -> ClientResult<PrioritizationFeeDetails> {
        let compute_budget_limits = process_compute_budget_instructions(
            sanitized_transaction.message().program_instructions_iter(),
        )?;
        let prioritization_fee_details = PrioritizationFeeDetails::new(
            PrioritizationFeeType::ComputeUnitPrice(compute_budget_limits.compute_unit_price),
            u64::from(compute_budget_limits.compute_unit_limit),
        );
        Ok(prioritization_fee_details)
    }
}

impl<C: ?Sized + Client> CalculatePrioritizationFee for C {}
