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
	ZERO_ACCOUNT, SCAError
};

use crate::account::{
	is_owned_and_writable,
	is_signed_by_writable_account,
};

/// Initializes an Operation. Note that this function expectes a CLOSED [OperationAccount].
/// It resets the account before using it, to make absolutely sure it's empty.
///
/// operation_data: A tuple conformant to [OperationInstruction::InitializeOperation]
///
///	Accounts:
///	1. SELLER - Account of the item seller, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Initialized here, reused elsewhere. Comformant to [OperationAccount]
#[inline(never)]
pub fn initialize_operation(program_id: &Pubkey, accounts: &[AccountInfo], 
	operation_data: (u64, TokenVersion,  [u8;46])) -> ProgramResult {

	// Destructure operation data tuple
	let (value,  token_version, ipfs_hash_bytes) = operation_data;

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


	// Get the estimated creation time for tracking operation validity, etc
	// This is in unixepoch seconds; in tests the time is days off, so it needs to be validated further 
	let unix_timestamp = match Clock::get() {
		Err(_e) => {
			msg!("WARN: Could not get a valid Clock. Using zero time.");
			0
		},
		Ok(clock) => clock.unix_timestamp,
	};

	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this a previously activated DATA account??? -> Reject
	if operation_account_data.status != OperationStatus::Closed {
		msg!("Operation account already in use.");
		return Err(ProgramError::AccountAlreadyInitialized)
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

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::Opened;
	operation_account_data.created_at = unix_timestamp;
	operation_account_data.buyer = Default::default(); // Not known yet at this point.

	operation_account_data.arbiter1 = Default::default(); // Not known yet at this point.
	operation_account_data.arbiter2 = Default::default(); // Not known yet at this point.
	operation_account_data.arbiter3 = Default::default(); // Not known yet at this point.

	operation_account_data.seller_approved = false;
	operation_account_data.buyer_approved = false;

	// Set externally
	operation_account_data.token_version = token_version;
	operation_account_data.value = value;
	operation_account_data.seller = *seller_account_info.key;
	operation_account_data.ipfs = ipfs_hash_bytes;

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Operation successfully initialized!");

	Ok(())
}

/// Allows a Buyer to confirm his participation in an operation.
/// Note that this function expectes an OPENED [OperationAccount].
///
///	Accounts:
///	1. BUYER - Account of the item buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn register_buyer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

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


	// CHECK: Is this DATA account in an incorrect state??? -> Reject
	if operation_account_data.status != OperationStatus::Opened {
		msg!("Operation account not setup.");
		return Err(ProgramError::UninitializedAccount)
	}

	if *buyer_account_info.key == operation_account_data.seller {
		msg!("Invalid Buyer account.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ======================= Enforce data validity using accounts data section


	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::BuyerRegistered;

	// Set externally
	operation_account_data.buyer = *buyer_account_info.key;

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Buyer registered to operation successfully.");

	Ok(())
}

/// Allows an arbiter to confirm his participation in an operation.
/// Note that this function expectes an BuyerRegistered [OperationAccount].
///
///	Accounts:
///	1. ARBITER - Account of one of the arbiters, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn register_arbiter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

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
	if operation_account_data.status != OperationStatus::BuyerRegistered {
		msg!("Operation account does not have a Buyer.");
		return Err(ProgramError::InvalidAccountData)
	}

	if *arbiter_account_info.key == operation_account_data.seller ||
		*arbiter_account_info.key == operation_account_data.buyer {
		msg!("Invalid Arbiter account.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ======================= Enforce data validity using accounts data section

	if operation_account_data.arbiter1 == *arbiter_account_info.key ||
		operation_account_data.arbiter2 == *arbiter_account_info.key ||
		operation_account_data.arbiter3 == *arbiter_account_info.key {
		msg!("Invalid Arbiter account.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.

	// Set externally

	if operation_account_data.arbiter1.to_bytes() == ZERO_ACCOUNT  {
		operation_account_data.arbiter1 = *arbiter_account_info.key;
	} else if operation_account_data.arbiter2.to_bytes() == ZERO_ACCOUNT  {
		operation_account_data.arbiter2 = *arbiter_account_info.key;
	} else if operation_account_data.arbiter3.to_bytes() == ZERO_ACCOUNT  {
		operation_account_data.arbiter3 = *arbiter_account_info.key;
	} else {
		msg!("Arbiters already filled.");
		return Err(ProgramError::AccountAlreadyInitialized)
	}

	if operation_account_data.arbiter1.to_bytes() != ZERO_ACCOUNT &&
		operation_account_data.arbiter2.to_bytes() != ZERO_ACCOUNT &&
		operation_account_data.arbiter3.to_bytes() != ZERO_ACCOUNT {
		operation_account_data.status = OperationStatus::ArbitersRegistered;
	}

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Arbiter registered to operation successfully.");

	Ok(())
}

/// Allows a Buyer/Seller to confirm approval of arbiters
/// Note that this function expectes an ArbitersRegistered [OperationAccount].
///
///	Accounts:
///	1. PARTICIPANT - Account of the item seller/buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn participant_approves_arbiters(program_id: &Pubkey, accounts: &[AccountInfo], 
	is_seller: bool) -> ProgramResult {

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
	if operation_account_data.status != OperationStatus::ArbitersRegistered {
		msg!("Operation account not setup.");
		return Err(ProgramError::UninitializedAccount)
	}


	// ======================= Enforce data validity using accounts data section

	if is_seller == true {
		if *participant_account_info.key != operation_account_data.seller {
			msg!("Invalid Seller account.");
			return Err(ProgramError::InvalidAccountData)
		}	
		operation_account_data.seller_approved = true;
	} else {
		if *participant_account_info.key != operation_account_data.buyer {
			msg!("Invalid Buyer account.");
			return Err(ProgramError::InvalidAccountData)
		}
		operation_account_data.buyer_approved = true;
	}

	// ========================= Change state section

	// Set internally; make sure everything not used is zeroed out anyway.

	if operation_account_data.seller_approved == true &&
		operation_account_data.buyer_approved == true {
			operation_account_data.status = OperationStatus::ArbitersApproved;
	}

	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Participant approved operation successfully.");

	Ok(())
}

/// Allows a Buyer to make his token deposit in an operation.
/// Note that this function expects an ArbitersApproved [OperationAccount].
///
///	Accounts:
///	1. BUYER - Account of the item buyer, who also pays for this transaction.
///	2. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn buyer_deposit(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

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

	let system_program_account_info = next_account_info(accounts_iter)?;

	// ================ Enforce configuration rules section

	if check_id(system_program_account_info.key) != true {
		msg!("Invalid System program");
		return Err(ProgramError::InvalidArgument)
	}

	// Load the account so that we can read it and/or modify it.
	let mut operation_account_data = OperationAccount::try_from_slice(&operation_account_info.data.borrow())?;

	// ================ Enforce previous state section


	// CHECK: Is this DATA account in an incorrect state??? -> Reject
	if operation_account_data.status != OperationStatus::ArbitersApproved {
		msg!("Operation account not setup.");
		return Err(ProgramError::UninitializedAccount)
	}

	if *buyer_account_info.key != operation_account_data.buyer {
		msg!("Invalid Buyer account.");
		return Err(ProgramError::InvalidAccountData)
	}

	// ======================= Enforce data validity using accounts data section

	// Get deposit for buyer account
	let rent_exemption_balance = match Rent::get() {
		Err(_e) => return Err(ProgramError::Custom(SCAError::RentError as u32)),
		Ok(rent) => rent.minimum_balance(buyer_account_info.data_len()),
	};

	// Does the from account have enough lamports to transfer? 
	// Alternatively, thsi could be a specific acct created for the express purpose
	// of moving lamports here, in which case would need to match exactly.
	if buyer_account_info.lamports() < (rent_exemption_balance + operation_account_data.value) {
		return Err(ProgramError::InsufficientFunds);
	}

	// ========================= Change state section

	/* 
	// Won't work because program does not own Buyer acct.
	// Debit from_account and credit to_account
	**buyer_account_info.try_borrow_mut_lamports()? -= operation_account_data.value;
	**operation_account_info.try_borrow_mut_lamports()? += operation_account_data.value;
	*/

	let instruction_transfer = transfer(
		buyer_account_info.key, // Payer
		operation_account_info.key, // Recipient
		operation_account_data.value
	);

	invoke(
		&instruction_transfer,
		&[buyer_account_info.clone(), operation_account_info.clone(), system_program_account_info.clone()],
	)?;

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::BuyerDeposited;

	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Buyer deposit token value ok.");

	Ok(())
}

/// Allows a Buyer to release his token deposit in an operation to the seller.
/// Note that this function expects a BuyerDeposit [OperationAccount].
///
///	Accounts:
///	1. BUYER - Account of the item buyer, who also pays for this transaction.
///	2. SELLER - Account of the item seller
///	3. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn buyer_release(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get BUYER account
	let buyer_account_info = next_account_info(accounts_iter)?;

	// Check BUYER account validity
	is_signed_by_writable_account(buyer_account_info, "BUYER account is not a valid account.")?;

	//	Get SELLER account
	let seller_account_info = next_account_info(accounts_iter)?;

	// Check SELLER account validity
	// Seller is not a signer here, and the pubkey is already stored.
	
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

	if *buyer_account_info.key != operation_account_data.buyer {
		msg!("Invalid Buyer account.");
		return Err(ProgramError::InvalidAccountData)
	}

	if *seller_account_info.key != operation_account_data.seller {
		msg!("Invalid Seller account.");
		return Err(ProgramError::InvalidAccountData)
	}	
	// ======================= Enforce data validity using accounts data section


	// ========================= Change state section

	// Debit from_account and credit to_account
	**operation_account_info.try_borrow_mut_lamports()? -= operation_account_data.value;
	**seller_account_info.try_borrow_mut_lamports()? += operation_account_data.value;

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::ReleaseRefund;

	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Buyer release token value ok.");

	Ok(())
}

/// Allows a Seller to return the token deposit in an operation to the buyer.
/// Note that this function expects a BuyerDeposit [OperationAccount].
///
///	Accounts:
///	1. SELLER - Account of the item seller, who also pays for this transaction.
///	2. BUYER - Account of the item buyer, who also pays for this transaction.
///	3. OPERATIONACCOUNT - Represents the ongoing operation. Comformant to [OperationAccount]
#[inline(never)]
pub fn seller_refund(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {

	// Iterating accounts is safer than indexing
	let accounts_iter = &mut accounts.iter();

	// ================ Validate accounts section

	//	Get SELLER account
	let seller_account_info = next_account_info(accounts_iter)?;

	// Check SELLER account validity
	is_signed_by_writable_account(seller_account_info, "SELLER account is not a valid account.")?;

	//	Get BUYER account
	let buyer_account_info = next_account_info(accounts_iter)?;

	// Check BUYER account validity
	// Buyer is not a signer here, and the pubkey is already stored.
	
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

	if *buyer_account_info.key != operation_account_data.buyer {
		msg!("Invalid Buyer account.");
		return Err(ProgramError::InvalidAccountData)
	}

	if *seller_account_info.key != operation_account_data.seller {
		msg!("Invalid Seller account.");
		return Err(ProgramError::InvalidAccountData)
	}	
	// ======================= Enforce data validity using accounts data section


	// ========================= Change state section

	// Debit from_account and credit to_account
	**operation_account_info.try_borrow_mut_lamports()? -= operation_account_data.value;
	**buyer_account_info.try_borrow_mut_lamports()? += operation_account_data.value;

	// Set internally; make sure everything not used is zeroed out anyway.
	operation_account_data.status = OperationStatus::ReleaseRefund;

	// Set externally

	// Save
	operation_account_data.serialize(&mut &mut operation_account_info.data.borrow_mut()[..])?;
	msg!("Seller refund token value ok.");

	Ok(())
}
