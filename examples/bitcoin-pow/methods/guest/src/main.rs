#![no_main]
#![no_std]

use crypto_bigint::{Encoding, U256};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub const NUM_BLOCKS: u32 = 10;
pub const PREV_BLOCK_HASH: [u8; 32] = [
    111, 226, 140, 10, 182, 241, 179, 114, 193, 166, 162, 70, 174, 99, 247, 79, 147, 30, 131, 101,
    225, 90, 8, 156, 104, 214, 25, 0, 0, 0, 0, 0,
];

macro_rules! double_sha256_hash {
    ($($data:expr),+) => {{
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        // First round of SHA256 hashing
        $(
            hasher.update($data);
        )+
        let first_hash_result = hasher.finalize_reset();
        // Second round of SHA256 hashing
        hasher.update(first_hash_result);
        let result: [u8; 32] = hasher.finalize().try_into().expect("SHA256 should produce a 32-byte output");
        result
    }};
}

pub fn validate_threshold_and_add_work(
    block_header_bits: [u8; 4],
    block_hash: [u8; 32],
    old_work: U256,
) -> U256 {
    // Step 1: Decode the target from the 'bits' field
    let target = decode_compact_target(block_header_bits);

    // Step 2: Compare the block hash with the target
    check_hash_valid(block_hash, target);

    // Step 3: Calculate work
    let work = calculate_work(target);

    old_work.wrapping_add(&work)
}

pub fn decode_compact_target(bits: [u8; 4]) -> [u8; 32] {
    let mut bits = bits;
    bits.reverse();

    let mut target = [0u8; 32];
    let exponent = bits[0] as usize;
    let value = ((bits[1] as u32) << 16) | ((bits[2] as u32) << 8) | (bits[3] as u32);

    if exponent <= 3 {
        // If the target size is 3 bytes or less, place the value at the end
        let start_index = 4 - exponent;
        for i in 0..exponent {
            target[31 - i] = (value >> (8 * (start_index + i))) as u8;
        }
    } else {
        // If the target size is more than 3 bytes, place the value at the beginning and shift accordingly
        for i in 0..3 {
            target[exponent - 3 + i] = (value >> (8 * i)) as u8;
        }
    }

    target
}

fn check_hash_valid(hash: [u8; 32], target: [u8; 32]) {
    // for loop from 31 to 0
    for i in (0..32).rev() {
        if hash[i] < target[i] {
            // The hash is valid because a byte in hash is less than the corresponding byte in target
            return;
        } else if hash[i] > target[i] {
            // The hash is invalid because a byte in hash is greater than the corresponding byte in target
            panic!("Hash is not valid");
        }
        // If the bytes are equal, continue to the next byte
    }
    // If we reach this point, all bytes are equal, so the hash is valid
}

pub fn calculate_work(target: [u8; 32]) -> U256 {
    let target_plus_one = U256::from_le_bytes(target).saturating_add(&U256::ONE);
    let work = U256::MAX.wrapping_div(&target_plus_one);
    work
}

fn main() {
    let k: u32 = env::read();
    let mut total_work = U256::ZERO;
    let mut curr_prev_block_hash = PREV_BLOCK_HASH;
    for _ in 0..k {
        let curr_version: i32 = env::read();
        let curr_merkle_root: [u8; 32] = env::read();
        let curr_time: u32 = env::read();
        let curr_bits: u32 = env::read();
        let curr_nonce: u32 = env::read();
        total_work = validate_threshold_and_add_work(
            curr_bits.to_le_bytes(),
            curr_prev_block_hash,
            total_work,
        );
        curr_prev_block_hash = double_sha256_hash!(
            &curr_version.to_le_bytes(),
            &curr_prev_block_hash,
            &curr_merkle_root,
            &curr_time.to_le_bytes(),
            &curr_bits.to_le_bytes(),
            &curr_nonce.to_le_bytes()
        );
    }
    let mut dummy_groth16_proof: [[u8; 33]; 4] = [[0u8; 33]; 4];
    for i in 0..4 {
        for j in 0..33 {
            dummy_groth16_proof[i][j] = env::read();
        }
    }
    let dummy_challenge_period: u32 = env::read();
    let curr = env::cycle_count();
    tracing::info!("Cycle count: {}", curr);
    let mut dummy_groth16_proof_bytes: [u8; 132] = [0u8; 132];
    for i in 0..4 {
        for j in 0..33 {
            let idx = 33 * i + j;
            dummy_groth16_proof_bytes[idx] = dummy_groth16_proof[i][j];
        }
    }
    let mut env_dummy_groth16_proof = [0u32; 33];
    for i in 0..33 {
        env_dummy_groth16_proof[i] = (dummy_groth16_proof_bytes[4 * i] as u32) + ((dummy_groth16_proof_bytes[4 * i + 1] as u32) << 8) + ((dummy_groth16_proof_bytes[4 * i + 2] as u32) << 16) + ((dummy_groth16_proof_bytes[4 * i + 3] as u32) << 24);
    }
    let mut env_curr_prev_block_hash: [u32; 8] = [0u32; 8];
    for i in 0..8 {
        env_curr_prev_block_hash[i] = (curr_prev_block_hash[4 * i] as u32) + ((curr_prev_block_hash[4 * i + 1] as u32) << 8) + ((curr_prev_block_hash[4 * i + 2] as u32) << 16) + ((curr_prev_block_hash[4 * i + 3] as u32) << 24);
    }
    let mut env_total_work: [u32; 8] = [0u32; 8];
    let total_work_bytes: [u8; 32] = total_work.to_be_bytes();
    for i in 0..8 {
        env_total_work[i] = (total_work_bytes[4 * i] as u32) + ((total_work_bytes[4 * i + 1] as u32) << 8) + ((total_work_bytes[4 * i + 2] as u32) << 16) + ((total_work_bytes[4 * i + 3] as u32) << 24);
    }
    let mut env_groth16_proof_32_bytes = [0u32; 32];
    env_groth16_proof_32_bytes.copy_from_slice(&env_dummy_groth16_proof[0..32]);
    let env_groth16_proof_last = env_dummy_groth16_proof[32];
    // Outputs:
    // env::commit(&env_groth16_proof_32_bytes);
    // env::commit(&env_groth16_proof_last);
    env::commit(&env_total_work);
    env::commit(&env_curr_prev_block_hash);
    // env::commit(&dummy_challenge_period);
}
