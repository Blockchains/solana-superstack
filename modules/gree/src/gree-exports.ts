// Here we export some useful types and functions for interacting with the Anchor program.
import { PublicKey } from '@solana/web3.js';
import type { Gree } from '../target/types/gree';
import { IDL as GreeIDL } from '../target/types/gree';

// Re-export the generated IDL and type
export { Gree, GreeIDL };

// After updating your program ID (e.g. after running `anchor keys sync`) update the value below.
export const programId = new PublicKey(
  'Hx1mCvAWL7PfTTFdkCHmyEAycLWMyybAe4eQLvWZJKDr'
);
