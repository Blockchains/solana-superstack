//! Solana Universal Trading
//! by Sergio Flores from SAFT.Industries (sergio@saft.industries)
//!
//! A tool to secure sales of physical items in long distance operations.
//!
//!
mod instruction;
mod scatype;
mod account;
mod operation;
mod dispute;

use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg, program_error::ProgramError,
	declare_id,
};

use operation::{
	initialize_operation,
	register_buyer, register_arbiter,
	participant_approves_arbiters, buyer_deposit,
	buyer_release, seller_refund,
};

use dispute::{
	start_dispute,
	seller_add_info, buyer_add_info,
	arbiter_vote, participant_claim,
};

use instruction::OperationInstruction;

declare_id!("7f3bKvFg9WrUr3RGig5gGj8GnEFYMML86ffgxaH19ft1");  // Localhost

entrypoint!(fn_main);

pub fn fn_main(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {

	// Check that this program is loaded to the publickey that is supposed to be loaded in.
	if id() != *program_id {
		return Err(ProgramError::IncorrectProgramId);
	}

	let instruction = OperationInstruction::unpack_instruction_data(instruction_data)?;
	return process_instruction(instruction, program_id, accounts);
}

/// Executes the appropriate instruction, already deserialized.
#[inline(never)]
fn process_instruction(instruction: OperationInstruction, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

	match instruction {
		OperationInstruction::InitializeOperation(operation_data) => {
			return initialize_operation(program_id, accounts, operation_data);
		},
		OperationInstruction::RegisterBuyer => {
			return register_buyer(program_id, accounts);
		},
		OperationInstruction::RegisterArbiter => {
			return register_arbiter(program_id, accounts);
		},
		OperationInstruction::ParticipantApprovesArbiters(is_seller) => {
			return participant_approves_arbiters(program_id, accounts, is_seller);
		},
		OperationInstruction::BuyerDeposit => {
			return buyer_deposit(program_id, accounts);
		},
		OperationInstruction::BuyerRelease => {
			return buyer_release(program_id, accounts);
		},
		OperationInstruction::SellerRefund => {
			return seller_refund(program_id, accounts);
		},
		OperationInstruction::StartDispute => {
			return start_dispute(program_id, accounts);
		},
		OperationInstruction::SellerAddInfo(ipfs_hash_bytes) => {
			return seller_add_info(program_id, accounts, ipfs_hash_bytes);
		},
		OperationInstruction::BuyerAddInfo(ipfs_hash_bytes) => {
			return buyer_add_info(program_id, accounts, ipfs_hash_bytes);
		},
		OperationInstruction::ArbiterVote(vote) => {
			return arbiter_vote(program_id, accounts, vote);
		},
		OperationInstruction::ParticipantClaim => {
			return participant_claim(program_id, accounts);
		},
	}
}