// I'll need to find a way to make this less repetitive without scrificing clarity and speed of execution
// But at the same time there will always be two variants for SHA2: 32bits and 64bits so making these functions generic isn't very relevant

use std::vec::Vec;
use std::convert::TryInto;
use std::mem::size_of;

const BYTE_SIZE: usize = 8;
const NUM_BYTES_512: usize = 512 / BYTE_SIZE;
const NUM_BYTES_1024: usize = 1024 / BYTE_SIZE;

fn bytes_to_512_blocks(bytes: &[u8]) -> Vec<[u32;16]> {
    let num_blocks = bytes.len() / NUM_BYTES_512;

    let mut blocks: Vec<[u32;16]> = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks {
        let mut bytes_for_block: [u32;16] = [0;16];
        for j in 0..16 {
            let start_idx = j*4 + NUM_BYTES_512 * i;
            bytes_for_block[j] = u32::from_be_bytes(bytes[start_idx..start_idx+4].try_into().unwrap());
        }
        
        blocks.push(bytes_for_block);
    }

    blocks
}

fn bytes_to_1024_blocks(bytes: &[u8]) -> Vec<[u64;16]> {
    let num_blocks = bytes.len() / NUM_BYTES_1024;

    let mut blocks: Vec<[u64;16]> = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks {
        let mut bytes_for_block: [u64;16] = [0;16];
        for j in 0..16 {
            let start_idx = j*8 + NUM_BYTES_1024 * i;
            bytes_for_block[j] = u64::from_be_bytes(bytes[start_idx..start_idx+8].try_into().unwrap());
        }
        
        blocks.push(bytes_for_block);
    }

    blocks
}

// Returns 512bits blocks from a message to hash (provided as a byte list)
pub fn blockify_msg_512(msg: &[u8]) -> Vec<[u32;16]> {
    const BLOCK_BITS: u32 = 512;
    const REQUIRED_FREE_SPACE: u32 = 448;

    let num_bits_msg = msg.len() * BYTE_SIZE;
    let num_zeros_padding = (u32::wrapping_sub(REQUIRED_FREE_SPACE, num_bits_msg as u32 + 1)) % BLOCK_BITS;

    let msg_size_as_bytes = (num_bits_msg as u64).to_be_bytes();
    let num_0_bytes = (num_zeros_padding as usize - 7) / BYTE_SIZE; // -7 because 7 zeros are already in the byte that hold the 1
    
    let total_padded_size = msg.len() +
    1 +             // The byte that holds the one padded after the msg 
    num_0_bytes +   // all 0s
    size_of::<u64>();             

    let mut padded_msg: Vec<u8> = Vec::with_capacity(total_padded_size);
    padded_msg.extend_from_slice(msg);
    padded_msg.push(0b1000_0000); // <- just append a 1 after message
    padded_msg.resize(padded_msg.len() + num_0_bytes, 0); // push the 0s
    padded_msg.extend_from_slice(&msg_size_as_bytes);   // push the msg size

    bytes_to_512_blocks(&padded_msg)
}

pub fn blockify_msg_1024(msg: &[u8]) -> Vec<[u64;16]> {    
    const BLOCK_BITS: u64 = 1024;
    const REQUIRED_FREE_SPACE: u64 = 896;

    let num_bits_msg = msg.len() * BYTE_SIZE;
    let num_zeros_padding = (u64::wrapping_sub(REQUIRED_FREE_SPACE, num_bits_msg as u64 + 1)) % BLOCK_BITS;

    let msg_size_as_bytes = (num_bits_msg as u128).to_be_bytes();
    let num_0_bytes = (num_zeros_padding as usize - 7) / BYTE_SIZE; // -7 because 7 zeros are already in the byte that hold the 1
    
    let total_padded_size = msg.len() +
    1 +             // The byte that holds the one padded after the msg 
    num_0_bytes +   // all 0s
    size_of::<u128>();
    
    let mut padded_msg: Vec<u8> = Vec::with_capacity(total_padded_size);
    padded_msg.extend_from_slice(msg);
    padded_msg.push(0b1000_0000); // <- just append a 1 after message
    padded_msg.resize(padded_msg.len() + num_0_bytes, 0); // push the 0s
    padded_msg.extend_from_slice(&msg_size_as_bytes);   // push the msg size

    bytes_to_1024_blocks(&padded_msg)
}