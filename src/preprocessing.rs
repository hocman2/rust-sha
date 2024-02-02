use std::vec::Vec;
use std::convert::TryInto;

// These constants are here to make code more explicit, not to actually reparametrize the module
// Changing these values WILL break the logic (lots of indices are hardcoded, array sizes are assumed to be of a certain size, etc.)

const BYTE_SIZE: usize = 8;
const MESSAGE_BLOCK_SIZE: usize = 512;
const NUM_BYTES_128: usize = 128 / BYTE_SIZE;
const NUM_BYTES_512: usize = 512 / BYTE_SIZE;

#[allow(non_camel_case_types)]
type u512 = [u128;4];
#[allow(non_camel_case_types)]
type u128BytesArray = [u8; NUM_BYTES_128];
#[allow(non_camel_case_types)]
type u512BytesArray = [u8; NUM_BYTES_512];
#[allow(non_camel_case_types)]
type MessageBlockQuarters = (u128BytesArray, u128BytesArray, u128BytesArray, u128BytesArray);

fn bytes_to_be_u128(bytes: u128BytesArray) -> u128 {
    let mut be_128: u128 = 0;

    for i in 0..NUM_BYTES_128 {
        be_128 |= (bytes[i] as u128) << (15-i) * BYTE_SIZE;
    }

    be_128
}

// Returns 64 bytes as 4 quarters of 16 bytes
fn get_data_quarters(block_data: u512BytesArray) -> MessageBlockQuarters {
    (
        block_data.get(0..NUM_BYTES_128).expect("Failed to retrieve indices 0 to 15").try_into().unwrap(), 
        block_data.get(NUM_BYTES_128..2*NUM_BYTES_128).expect("Failed to retrieve indices 16 to 31").try_into().unwrap(),
        block_data.get(2*NUM_BYTES_128..3*NUM_BYTES_128).expect("Failed to retrieve indices 32 to 47").try_into().unwrap(),
        block_data.get(3*NUM_BYTES_128..4*NUM_BYTES_128).expect("Failed to retrieve indices 48 to 64").try_into().unwrap()
    )
}

fn bytes_to_be_u512(block_data: u512BytesArray) -> u512 {
    let mut block: u512 = [0, 0, 0, 0];

    // The 512 bits array is split into 4 128 bits arrays
    let block_quarters: MessageBlockQuarters = get_data_quarters(block_data);
        
    // We can easily convert a u128ByteArray into a u128, repeating this process 4 times gives us a u512
    block[0] = bytes_to_be_u128(block_quarters.0);
    block[1] = bytes_to_be_u128(block_quarters.1);
    block[2] = bytes_to_be_u128(block_quarters.2);
    block[3] = bytes_to_be_u128(block_quarters.3);

    block
}

fn get_msg_size_as_be_u128(message_size: usize) -> u128 {
    0 | u64::to_be(message_size as u64) as u128
}

// Returns a block that is fully padded, meaning the most significant bit is 1 and the 64 least significant bits are the message size in big endian
fn get_fully_padded_block(message_size: usize) -> u512 {
    [
        1 << 127,
        0,
        0,
        get_msg_size_as_be_u128(message_size)
    ]
}

// Given the message's last block, returns the padded block(s) as either a single block or two blocks (depends on number of available bits)
fn add_padding(msg_last_block: u512, last_block_data_size: usize, message_size: usize) -> (u512, Option<u512>) {
    let mut msg_last_block = msg_last_block;

    // Let's first find the index at which the last bits of data are written in the u512
    let access_index: usize = last_block_data_size / 128;
    let num_bitshift = last_block_data_size - (128 * access_index) - 1; // this is by how much we need to bitshift the 1 to arrive right at data end

    msg_last_block[access_index] |= 1 << num_bitshift;          // Append a 1 right after the message data
    msg_last_block[access_index] &= (1 << num_bitshift) - 1;    // set all bits below the 1 to 0s

    // This ensures the rest of the block is filled with 0s
    for i in access_index+1..4 {
        msg_last_block[i] = 0;
    }

    // We have room to put the message size as a 64 bit integer in the last block (we need to account for the 1 that was added)
    if last_block_data_size <= MESSAGE_BLOCK_SIZE - 64 - 1 {
        // Append the msg size to the last 64 bits
        msg_last_block[3] |= get_msg_size_as_be_u128(message_size);

        (msg_last_block, None)
    }
    // We need to create a new block with message size at the end
    else {
        (
            msg_last_block,
            Some([0,0,0,get_msg_size_as_be_u128(message_size)])
        )
    }
}

fn build_incomplete_block(msg: &[u8], num_complete_blocks: usize) -> u512 {

    // Find index of the first byte of the last block
    let start_idx = num_complete_blocks * MESSAGE_BLOCK_SIZE / BYTE_SIZE;
            
    let mut last_block: u512 = [0, 0, 0, 0];

    // a lil tracker needed to calculate bit offset
    let mut num_it = 0;
    
    for i in start_idx..msg.len() {
        let block_idx = num_it / NUM_BYTES_128; // determine which u128 of the block should we operate on

        let bit_offset = ((NUM_BYTES_128-1) - num_it % NUM_BYTES_128) * BYTE_SIZE;
        // ⬆️ this formula gives us by how much a byte should be offset to be appended after the previous one, taking into account
        // on which u128 element of the u512 we are at... amazing

        last_block[block_idx] |= (msg[i] as u128) << bit_offset;

        num_it += 1;
    }

    last_block
}

// Returns 512bits blocks from a message to hash (provided as a byte list)
pub fn cut_msg(msg: &[u8]) -> Vec<u512> {
    let mut blocks: Vec<u512> = Vec::new();

    let msg_size = msg.len()*8;
    let num_complete_blocks = msg_size / MESSAGE_BLOCK_SIZE;
    
    for i in 1..num_complete_blocks+1 {
        // 0 .. 64 -> 64 .. 128 -> 128 .. 192 etc.
        let block_data: u512BytesArray = msg.get(NUM_BYTES_512 * i-1 .. NUM_BYTES_512 * i).expect("Failed to split msg as a valid 64 sized slice").try_into().unwrap();
        
        let block = bytes_to_be_u512(block_data);
        blocks.push(block);
    }

    let incomplete_block_size = (msg.len() * BYTE_SIZE) % MESSAGE_BLOCK_SIZE;
    
    // If this is worth 0 this means our message bit size is a multiple of 512, we'll simply add a fully padded block to prevent length extension attacks
    if incomplete_block_size == 0 {
        blocks.push(get_fully_padded_block(msg_size))
    }
    // Otherwise we'll add padding to the msg last block
    else {
        let last_block = build_incomplete_block(msg, num_complete_blocks);
        let padded_blocks = add_padding(last_block, incomplete_block_size, msg_size);

        blocks.push(padded_blocks.0);
        if let Some(block) = padded_blocks.1 {
            blocks.push(block);
        }
    }

    blocks
}