use std::vec::Vec;
use std::convert::TryInto;

const BYTE_SIZE: usize = 8;
const NUM_BYTES_512: usize = 512 / BYTE_SIZE;

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

// Returns 512bits blocks from a message to hash (provided as a byte list)
pub fn blockify_msg(msg: &[u8]) -> Vec<[u32;16]> {    
    let num_bits_msg = msg.len() * BYTE_SIZE;
    let num_zeros_padding = (u32::wrapping_sub(448, num_bits_msg as u32 + 1)) % 512;

    let msg_size_as_bytes = (num_bits_msg as u64).to_be_bytes();
    let num_0_bytes = (num_zeros_padding as usize - 7) / BYTE_SIZE; // -7 because 7 zeros are already in the byte that hold the 1
    
    let total_padded_size = msg.len() +
    1 +             // The byte that holds the one padded after the msg 
    num_0_bytes +   // all 0s
    8;              // 8 bytes for the msg size
    
    let mut padded_msg: Vec<u8> = Vec::with_capacity(total_padded_size);
    padded_msg.extend_from_slice(msg);
    padded_msg.push(0b1000_0000); // <- just append a 1 after message
    padded_msg.resize(padded_msg.len() + num_0_bytes, 0); // push the 0s
    padded_msg.extend_from_slice(&msg_size_as_bytes);   // push the msg size

    bytes_to_512_blocks(&padded_msg)
}