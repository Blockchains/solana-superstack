//!	# types - Solana Universal Trading Types
//!	This crate contains the Solana Universal Trading specific type definitions, as required by the Solana program
//!
//! The available types are:
//! TokenVersion -> [TokenVersion]
//! OperationStatus -> [OperationStatus]
//! OperationAccount -> [OperationAccount]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    pubkey::Pubkey,
	clock::UnixTimestamp,
};

/// Special Zero account that owns all keypairs
pub const ZERO_ACCOUNT:[u8;32] = [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]; 

/// The type of funding in use by the Operation. In principle, only SOL supported.
#[derive(PartialEq, BorshSerialize, BorshDeserialize, Debug)]
pub enum TokenVersion {
	/// The native Solana token.
	Sol,
}

/// The options for dispute voting in use by the Operation.
#[derive(PartialEq, BorshSerialize, BorshDeserialize, Debug)]
pub enum VotingOptions {
	NoVote,
	Buyer,
	Seller,
}

/// The status of the Operation account as the operation progresses.
#[derive(PartialEq, BorshSerialize, BorshDeserialize, Debug)]
pub enum OperationStatus {
	/// Account just created, ready for initialization.
	Closed,
	/// Account in use by an Operation. 
	Opened,
	/// Buyer has agreed to participate.
	BuyerRegistered,
	/// Arbiters regsitered
	ArbitersRegistered,
	/// Parties approve
	ArbitersApproved,
	/// Buyer has made the deposit
	BuyerDeposited,
	/// Token value refunded or released
	ReleaseRefund,
	/// Dispute started
	InDispute,
	/// Arbiters are voting
	InVoting,
	/// Voting ended, Claim Result for Seller
	SellerClaim,
	/// Voting ended, Claim Result for Buyer
	BuyerClaim,
	/// Dispute concluded
	DisputeResolved, 
}

/// Defines the type for the Operation state stored in an account.
///
/// Every Operation will have 1 OperationAccount to hold its state.
/// The important operation rules are:
/// 1. Accounts cannot be reused, so they should have their rent withdrawn after an operation finishes.
#[derive(PartialEq, BorshSerialize, BorshDeserialize, Debug)]
pub struct OperationAccount {
	/// Status of the operation.
    pub status: OperationStatus,

	/// Approximate time of creation.
	pub created_at: UnixTimestamp, // Actually i64

	/// Sol/Etc
	pub token_version: TokenVersion,

	/// The token amount for the purchase.
	pub value: u64,

	/// Public key of the seller.
	pub seller: Pubkey,

	/// Public key of the buyer.
	pub buyer: Pubkey,

	/// IPFS hash
	pub ipfs: [u8;46],

	/// Public key of the arbiter.
	pub arbiter1: Pubkey,

	/// Public key of the arbiter.
	pub arbiter2: Pubkey,

	/// Public key of the arbiter.
	pub arbiter3: Pubkey,

	/// Seller approves arbiters
	pub seller_approved: bool,

	/// Buyer approves arbiters
	pub buyer_approved: bool,

	/// Seller additional IPFS hash
	pub seller_ipfs_ext: [u8;46],

	/// Buyer additional IPFS hash
	pub buyer_ipfs_ext: [u8;46],

	/// Public key of the arbiter.
	pub arbiter_vote_1: VotingOptions,

	/// Public key of the arbiter.
	pub arbiter_vote_2: VotingOptions,

	/// Public key of the arbiter.
	pub arbiter_vote_3: VotingOptions,
}

/// List of errors specific to the SCA operation 
#[derive(PartialEq, Debug)]
pub enum SCAError {

	/// Failure to retrieve a Rent object
	RentError = 1,
}

