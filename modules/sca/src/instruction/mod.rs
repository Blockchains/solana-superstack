//!	# instruction
//!	This crate is tasked with the serialization of instructions sent to the program
//!
//!  

use solana_program::{
	program_error::ProgramError,
	pubkey::Pubkey,};
use std::convert::TryInto;
use std::default::Default;

use crate::scatype::{
	TokenVersion, 
};

/// The supported instruction set
#[derive(PartialEq, Debug)]
pub enum OperationInstruction {
	/// Encapsulates the variables needed to create an operation.
	/// Value, TokenVersion, IPFSCID 
	InitializeOperation((u64, TokenVersion, [u8;46])),
	/// Buyer registers his own address to indicate participation in the operation.
	RegisterBuyer,
	/// Arbiter registers his own address to indicate participation in the operation.
	RegisterArbiter,
	/// Seller/Buyer indicates their approval of arbiters.
	ParticipantApprovesArbiters(bool),
	/// Buyer deposits the agreed token amount.
	BuyerDeposit,
	/// Buyer accepts the item and releases token to the seller
	BuyerRelease,
	/// Seller cancels the operation and returns token to the buyer
	SellerRefund,
	/// A participant has requested dispute resolution
	StartDispute,
	/// Seller is providing additional info.
	SellerAddInfo([u8;46]),
	/// Buyer is providing additional info
	BuyerAddInfo([u8;46]),
	/// Arbiter votes on dispute, and if all votes are in, result is calculated.
	ArbiterVote(bool),
	/// Dispute winner claims token value
	ParticipantClaim,
}

impl OperationInstruction {

	/// Separates the 1st byte to determine the instruction to execute, and uses the rest to setup the appropriate variables for the corresponding instruction
    pub fn unpack_instruction_data(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        let (instruction, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
		
		// Check the correct instruction_data size and valid values for each instruction
        match instruction {
            0 => OperationInstruction::initialize_operation_builder(&data),
			1 => Ok(OperationInstruction::RegisterBuyer),
			2 => Ok(OperationInstruction::RegisterArbiter),
			3 => OperationInstruction::participant_approves_arbiters_builder(&data),
			4 => Ok(OperationInstruction::BuyerDeposit),
			5 => Ok(OperationInstruction::BuyerRelease),
			6 => Ok(OperationInstruction::SellerRefund),
			7 => Ok(OperationInstruction::StartDispute),
			8 => OperationInstruction::seller_add_info_builder(&data),
			9 => OperationInstruction::buyer_add_info_builder(&data),
			10 => OperationInstruction::arbiter_vote_builder(&data),
			11 => Ok(OperationInstruction::ParticipantClaim),
            _ => return Err(ProgramError::InvalidInstructionData),
        }
    }

	/// Returns an [OperationInstruction::InitializeOperation] with appropriate variables already validated and filled
	fn initialize_operation_builder(data: &[u8]) -> Result<Self, ProgramError> {

		if 	data.len() != 55 {			
			return Err(ProgramError::InvalidInstructionData);
		}

		// ================================= 0: value
		// Expecting 8 bytes in &data
		let data_bytes:[u8;8] = match 
			data[0..8]
			.try_into() {
				Err(_e) => return Err(ProgramError::InvalidInstructionData),
				Ok(b) => b,
		};
		let value:u64 = u64::from_le_bytes(data_bytes);

		// ================================= 8: token_version
		// Expecting 1 byte in &data
		let token_version:TokenVersion;

		match data[8] {
			0x00 => {
				token_version = TokenVersion::Sol;
			}
			_ => return Err(ProgramError::InvalidInstructionData),
		}

		// ================================= 8: ipfs_cid
		// Expecting 46 bytes in &data
		let ipfs_hash_bytes:[u8;46] = match 
			data[9..55]
			.try_into() {
				Err(_e) => return Err(ProgramError::InvalidInstructionData),
				Ok(b) => b,
		};

		Ok(OperationInstruction::InitializeOperation((value, token_version, ipfs_hash_bytes)))
	}

	/// Returns an [OperationInstruction::ParticipantApproves] with appropriate variables already validated and filled
	fn participant_approves_arbiters_builder(data: &[u8]) -> Result<Self, ProgramError> {

		if 	data.len() != 1 {			
			return Err(ProgramError::InvalidInstructionData);
		}

		// ================================= 0: is_seller
		// Expecting 1 byte in &data
		let is_seller: bool;

		match data[0] {
			0x00 => {
				is_seller = false;
			}
			0x01 => {
				is_seller = true;
			}
			_ => return Err(ProgramError::InvalidInstructionData),
		}

		Ok(OperationInstruction::ParticipantApprovesArbiters(is_seller))
	}

	/// Returns an [OperationInstruction::SellerAddInfo] with appropriate variables already validated and filled
	fn seller_add_info_builder(data: &[u8]) -> Result<Self, ProgramError> {

		if 	data.len() != 46 {			
			return Err(ProgramError::InvalidInstructionData);
		}

		// ================================= 0: ipfs_cid
		// Expecting 46 bytes in &data
		let ipfs_hash_bytes:[u8;46] = match 
			data[0..46]
			.try_into() {
				Err(_e) => return Err(ProgramError::InvalidInstructionData),
				Ok(b) => b,
		};

		Ok(OperationInstruction::SellerAddInfo(ipfs_hash_bytes))
	}

	/// Returns an [OperationInstruction::BuyerAddInfo] with appropriate variables already validated and filled
	fn buyer_add_info_builder(data: &[u8]) -> Result<Self, ProgramError> {

		if 	data.len() != 46 {			
			return Err(ProgramError::InvalidInstructionData);
		}

		// ================================= 0: ipfs_cid
		// Expecting 46 bytes in &data
		let ipfs_hash_bytes:[u8;46] = match 
			data[0..46]
			.try_into() {
				Err(_e) => return Err(ProgramError::InvalidInstructionData),
				Ok(b) => b,
		};

		Ok(OperationInstruction::BuyerAddInfo(ipfs_hash_bytes))
	}	

	/// Returns an [OperationInstruction::ArbiterVote] with appropriate variables already validated and filled
	fn arbiter_vote_builder(data: &[u8]) -> Result<Self, ProgramError> {

		if 	data.len() != 1 {			
			return Err(ProgramError::InvalidInstructionData);
		}

		// ================================= 0: vote
		// Expecting 1 byte in &data
		let vote: bool;

		match data[0] {
			0x00 => {
				vote = false;
			}
			0x01 => {
				vote = true;
			}
			_ => return Err(ProgramError::InvalidInstructionData),
		}

		Ok(OperationInstruction::ArbiterVote(vote))
	}
}
