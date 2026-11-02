//!	# account
//!	This crate is tasked with verifying the validity of accounts.
//!
//!  

use solana_program::{
	pubkey::Pubkey,
	account_info::AccountInfo,
    msg,
    program_error::ProgramError,
};

use crate::scatype::{
	ZERO_ACCOUNT
};

/// Checks that the given account is both writable and owned by the program. 
///
/// The account must be owned by the program in order to modify its data.  
/// The account must be marked as writable by the transaction.   
/// The account must NOT be an executable.  
/// The account must NOT be a signer.  
pub fn is_owned_and_writable(program_id: &Pubkey, account: &AccountInfo, message: &str) -> Result<(), ProgramError> {

	if 	account.owner != program_id {
		msg!(message);
		return Err(ProgramError::IllegalOwner);
	}
	
	if	account.is_writable &&
		!account.executable && 
        !account.is_signer {
		Ok(())
	} else {
		msg!("The provided account must be a writable data account.");
		msg!(message);
		Err(ProgramError::InvalidArgument)
	}

}

/// Checks that the given account is signing this transaction.
///
/// The account must be marked as writable by the transaction.   
/// The account must be a wallet address.    
/// The account must be a signer to the transaction.   
pub fn is_signed_by_writable_account(account: &AccountInfo, message: &str) -> Result<(), ProgramError> {

	// System owned only	
	if account.owner.to_bytes() != ZERO_ACCOUNT {
		msg!(message);
		return Err(ProgramError::IllegalOwner);
	}
	
	if 	!account.is_signer {
		msg!(message);
		return Err(ProgramError::MissingRequiredSignature);
	} 

	if	account.is_writable &&
		!account.executable {
		Ok(())
	} else {
		msg!("The provided account must be a writable wallet.");
		msg!(message);
		Err(ProgramError::InvalidArgument)
	}

}
