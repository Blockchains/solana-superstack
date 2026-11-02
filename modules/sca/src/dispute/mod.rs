//!	# operation
//!	Functionality related to Operations
//!
//! List of supported instructions
//!
//! 1. Initialize Operation -> [initialize_operation]
//!
//!

use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg, system_instruction::transfer,
    program_error::ProgramError,
    pubkey::Pubkey, system_program::check_id,
	clock::Clock, program::invoke,
	sysvar::{
		Sysvar,
		rent::Rent,
	}

};

use std::str;

use crate::scatype::{
	OperationAccount, OperationStatus, TokenVersion,
	ZERO_ACCOUNT, SCAError, VotingOptions,
};

use crate::account::{
	is_owned_and_writable,
	is_signed_by_writable_account,
};

/// Allows a Buyer/Seller to start a dispute on the operation
/// Note that this function expectes a BuyerDeposited [OperationAccount].
///
///	Accounts:
///	1. PARTICIPANT - Account of the item seller/buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn start_dispute(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get PARTICIPANT account
	let participant_account_info = next_account_info(accounts_iter)?;

	// Check PARTICIPANT account validity
	is_signed_by_writable_account(participant_account_info, "PARTICIPANT account is not a valid account.")?;
	
	// Get the OPERATIONACCOUNT account 
	let operation_account_info = next_account_info(accounts_iter)?;

	// Check OPERATIONACCOUNT account validity
	is_owned_and_writable(program_id, operation_account_info, "OPERATIONACCOUNT account is not a valid account.")?;


	// ================ Enforce configuration rules section


	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this DATA account in an incorrect state??? -> Reject
	if operation_account_data.status != OperationStatus::BuyerDeposited {
		msg!("Operation account not setup.");
		return Err(ProgramError::UninitializedAccount)
	}


	// ======================= Enforce data validity using accounts data section

	if *participant_account_info.key != operation_account_data.seller &&
     *participant_account_info.key != operation_account_data.buyer {
        msg!("Invalid Buyer/Seller account.");
        return Err(ProgramError::InvalidAccountData)
	}

	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::InDispute;

	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Participant opened dispute.");

	Ok(())
}

/// Allow Seller to save additional info. Note that this function expectes a InDispute [OperationAccount].
///
/// operation_data: A tuple conformant to [OperationInstruction::SellerAddInfo]
///
///	Accounts:
///	1. SELLER - Account of the item seller, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Initialized here, reused elsewhere. Comformant to [OperationAccount]
#[inline(never)]
pub fn seller_add_info(program_id: &Pubkey, accounts: &[AccountInfo], 
	ipfs_hash_bytes: [u8;46]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get SELLER account
	let seller_account_info = next_account_info(accounts_iter)?;

	// Check SELLER account validity
	is_signed_by_writable_account(seller_account_info, "SELLER account is not a valid account.")?;
	
	// Get the OPERATIONACCOUNT account 
	let operation_account_info = next_account_info(accounts_iter)?;

	// Check OPERATIONACCOUNT account validity
	is_owned_and_writable(program_id, operation_account_info, "OPERATIONACCOUNT account is not a valid account.")?;


	// ================ Enforce configuration rules section


	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this a previously activated DATA account??? -> Reject
	if operation_account_data.status != OperationStatus::InDispute {
		msg!("Operation account incorrect state.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ======================= Enforce data validity using accounts data section

	// Bytes to string, must be valid utf-8 (base58btc ascii in practice)
	let ipfs_hash_str = match str::from_utf8(&ipfs_hash_bytes) {
		Ok(v) => v,
		Err(_e) => return Err(ProgramError::InvalidInstructionData),
	};

	if !ipfs_hash_str.starts_with("Qm") {
		msg!("Invalid IPFS hash.");
		return Err(ProgramError::InvalidArgument);
	}

	// ========================= Change state section

	// Set externally
	operation_account_data.seller_ipfs_ext = ipfs_hash_bytes;

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Seller added extra info.");

	Ok(())
}

/// Allow Buyer to save additional info. Note that this function expectes a InDispute [OperationAccount].
///
/// operation_data: A tuple conformant to [OperationInstruction::BuyerAddInfo]
///
///	Accounts:
///	1. BUYER - Account of the item buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Initialized here, reused elsewhere. Comformant to [OperationAccount]
#[inline(never)]
pub fn buyer_add_info(program_id: &Pubkey, accounts: &[AccountInfo], 
	ipfs_hash_bytes: [u8;46]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get BUYER account
	let buyer_account_info = next_account_info(accounts_iter)?;

	// Check BUYER account validity
	is_signed_by_writable_account(buyer_account_info, "BUYER account is not a valid account.")?;
	
	// Get the OPERATIONACCOUNT account 
	let operation_account_info = next_account_info(accounts_iter)?;

	// Check OPERATIONACCOUNT account validity
	is_owned_and_writable(program_id, operation_account_info, "OPERATIONACCOUNT account is not a valid account.")?;


	// ================ Enforce configuration rules section


	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this a previously activated DATA account??? -> Reject
	if operation_account_data.status != OperationStatus::InDispute {
		msg!("Operation account incorrect state.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ======================= Enforce data validity using accounts data section

	// Bytes to string, must be valid utf-8 (base58btc ascii in practice)
	let ipfs_hash_str = match str::from_utf8(&ipfs_hash_bytes) {
		Ok(v) => v,
		Err(_e) => return Err(ProgramError::InvalidInstructionData),
	};

	if !ipfs_hash_str.starts_with("Qm") {
		msg!("Invalid IPFS hash.");
		return Err(ProgramError::InvalidArgument);
	}

	// ========================= Change state section

	// Set externally
	operation_account_data.buyer_ipfs_ext = ipfs_hash_bytes;

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Buyer added extra info.");

	Ok(())
}

/// Allows an arbiter to vote in an operation.
/// Note that this function expectes an InDispute/InVoting [OperationAccount].
///
///	Accounts:
///	1. ARBITER - Account of one of the arbiters, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn arbiter_vote(program_id: &Pubkey, accounts: &[AccountInfo], vote: bool) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get ARBITER account
	let arbiter_account_info = next_account_info(accounts_iter)?;

	// Check ARBITER account validity
	is_signed_by_writable_account(arbiter_account_info, "ARBITER account is not a valid account.")?;
	
	// Get the OPERATIONACCOUNT account 
	let operation_account_info = next_account_info(accounts_iter)?;

	// Check OPERATIONACCOUNT account validity
	is_owned_and_writable(program_id, operation_account_info, "OPERATIONACCOUNT account is not a valid account.")?;


	// ================ Enforce configuration rules section


	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this DATA account in an incorrect state??? -> Reject
	if operation_account_data.status != OperationStatus::InDispute &&
        operation_account_data.status != OperationStatus::InVoting {
		msg!("Operation account incorrect state.");
		return Err(ProgramError::InvalidAccountData)
	}


	// ======================= Enforce data validity using accounts data section

	if operation_account_data.arbiter1 == *arbiter_account_info.key {

		match operation_account_data.arbiter_vote_1 {
			VotingOptions::NoVote => {
				if vote == false {
					operation_account_data.arbiter_vote_1 = VotingOptions::Buyer;
				} else {
					operation_account_data.arbiter_vote_1 = VotingOptions::Seller;
				}
			},
			_ => {
				msg!("Already voted.");
				return Err(ProgramError::AccountAlreadyInitialized)               	
			}
		}

    } else if operation_account_data.arbiter2 == *arbiter_account_info.key {
        if operation_account_data.arbiter_vote_2 == VotingOptions::NoVote {
            if vote == false {
                operation_account_data.arbiter_vote_2 = VotingOptions::Buyer;
            } else {
                operation_account_data.arbiter_vote_2 = VotingOptions::Seller;
            }
        } else {
            msg!("Already voted.");
            return Err(ProgramError::AccountAlreadyInitialized)               
        }
    } else if operation_account_data.arbiter3 == *arbiter_account_info.key {
        if operation_account_data.arbiter_vote_3 == VotingOptions::NoVote {
            if vote == false {
                operation_account_data.arbiter_vote_3 = VotingOptions::Buyer;
            } else {
                operation_account_data.arbiter_vote_3 = VotingOptions::Seller;
            }
        } else {
            msg!("Already voted.");
            return Err(ProgramError::AccountAlreadyInitialized)               
        }
    } else {    
		msg!("Invalid Arbiter account.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.

	// Set externally

	operation_account_data.status = OperationStatus::InVoting;

    // All votes are mandatory
    if operation_account_data.arbiter_vote_1 != VotingOptions::NoVote && 
        operation_account_data.arbiter_vote_2 != VotingOptions::NoVote && 
        operation_account_data.arbiter_vote_3 != VotingOptions::NoVote
    {
        let mut buyer_claim = 0;
        let mut seller_claim = 0;
    
        // Vote count
        if operation_account_data.arbiter_vote_1 == VotingOptions::Buyer {
            buyer_claim = buyer_claim + 1;
        } else if operation_account_data.arbiter_vote_1 == VotingOptions::Seller {
            seller_claim = seller_claim + 1;
        }

        // Vote count
        if operation_account_data.arbiter_vote_2 == VotingOptions::Buyer {
            buyer_claim = buyer_claim + 1;
        } else if operation_account_data.arbiter_vote_2 == VotingOptions::Seller {
            seller_claim = seller_claim + 1;
        }
        
        // Vote count
        if operation_account_data.arbiter_vote_3 == VotingOptions::Buyer {
            buyer_claim = buyer_claim + 1;
        } else if operation_account_data.arbiter_vote_3 == VotingOptions::Seller {
            seller_claim = seller_claim + 1;
        }

        if buyer_claim > seller_claim {
            operation_account_data.status = OperationStatus::BuyerClaim;
            msg!("Buyer claim enabled.");
        } else {
            operation_account_data.status = OperationStatus::SellerClaim;
            msg!("Seller claims enabled.");
        }
    }

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Arbiter vote recorded.");

	Ok(())
}

/// Allows a Buyer/Seller to claim the dispute result
/// Note that this function expects either SellerClaim or BuyerClaim [OperationAccount].
///
///	Accounts:
///	1. PARTICIPANT - Account of the item seller/buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn participant_claim(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get PARTICIPANT account
	let participant_account_info = next_account_info(accounts_iter)?;

	// Check PARTICIPANT account validity
	is_signed_by_writable_account(participant_account_info, "PARTICIPANT account is not a valid account.")?;
	
	// Get the OPERATIONACCOUNT account 
	let operation_account_info = next_account_info(accounts_iter)?;

	// Check OPERATIONACCOUNT account validity
	is_owned_and_writable(program_id, operation_account_info, "OPERATIONACCOUNT account is not a valid account.")?;


	// ================ Enforce configuration rules section


	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this DATA account in an incorrect state??? -> Reject
	if operation_account_data.status == OperationStatus::SellerClaim {

		if *participant_account_info.key != operation_account_data.seller {
			msg!("Invalid Seller account.");
			return Err(ProgramError::InvalidAccountData)
		}	

    } else if operation_account_data.status == OperationStatus::BuyerClaim {

        if *participant_account_info.key != operation_account_data.buyer {
            msg!("Invalid Buyer account.");
            return Err(ProgramError::InvalidAccountData)
        }	
                    
    } else {
		msg!("Operation account not setup.");
		return Err(ProgramError::UninitializedAccount)
	}


	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.

	// Set internally; make sure everything not used is zeroed out anyway.
    // Debit from_account and credit to_account
    **operation_account_info.try_borrow_mut_lamports()? -= operation_account_data.value;
    **participant_account_info.try_borrow_mut_lamports()? += operation_account_data.value;

    operation_account_data.status = OperationStatus::DisputeResolved;
	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Dispute concluded.");

	Ok(())
}