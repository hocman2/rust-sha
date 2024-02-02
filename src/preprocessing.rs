use std::vec::Vec;
use std::convert::TryInto;

// These constants are here to make code more explicit, not to actually reparametrize the module
// Changing these values WILL break the logic (lots of indices are hardcoded, array sizes are assumed to be of a certain size, etc.)

#[allow(non_camel_case_types)]
type usizeBits = usize;
#[allow(non_camel_case_types)]
type usizeBytes = usize;
#[allow(non_camel_case_types)]
type u512 = [u128;4];
#[allow(non_camel_case_types)]
type u128BytesArray = [u8; NUM_BYTES_128];
#[allow(non_camel_case_types)]
type u512BytesArray = [u8; NUM_BYTES_512];
#[allow(non_camel_case_types)]
type MessageBlockQuarters = (u128BytesArray, u128BytesArray, u128BytesArray, u128BytesArray);

const BYTE_SIZE: usizeBits = 8;
const MESSAGE_BLOCK_SIZE: usizeBits = 512;
const NUM_BYTES_128: usizeBytes = 128 / BYTE_SIZE;
const NUM_BYTES_512: usizeBytes = 512 / BYTE_SIZE;

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

// Returns a u128 that contains a u64 with message size in big endian in the least significant bits
// 0b000...[100101010100] <-- this block is the message size in big endian
fn get_msg_size_as_be_u64(message_size: usizeBytes) -> u128 {
    0 | u64::to_be(message_size as u64) as u128
}

// Returns a block that is fully padded, meaning the most significant bit is 1 and the 64 least significant bits are the message size in big endian
fn get_fully_padded_block(message_size: usizeBytes) -> u512 {
    [
        1 << 127,
        0,
        0,
        get_msg_size_as_be_u64(message_size)
    ]
}

// Given the message's last block, returns the padded block(s) as either a single block or two blocks (depends on number of available bits)
fn add_padding(msg_last_block: u512, last_block_data_size: usizeBits, message_size: usizeBytes) -> (u512, Option<u512>) {
    let mut msg_last_block = msg_last_block;

    // Let's first find the index at which the last bits of data are written in the u512
    let access_index: usize = last_block_data_size / 128;

    // this is by how much we need to bitshift the 1 to arrive right at data end
    let num_bitshift = MESSAGE_BLOCK_SIZE - last_block_data_size;
    let num_bitshift = num_bitshift - 128 * (num_bitshift / 128) - 1;

    msg_last_block[access_index] |= 1 << num_bitshift; // Append a 1 right after the message data

    // We have room to put the message size as a 64 bit integer in the last block (we need to account for the 1 that was added)
    if last_block_data_size <= MESSAGE_BLOCK_SIZE - 64 - 1 {
        // Append the msg size to the last 64 bits
        msg_last_block[3] |= get_msg_size_as_be_u64(message_size);

        (msg_last_block, None)
    }
    // We need to create a new block with message size at the end
    else {
        (
            msg_last_block,
            Some([0,0,0,get_msg_size_as_be_u64(message_size)])
        )
    }
}

fn build_incomplete_block(msg: &[u8], num_complete_blocks: u32) -> u512 {

    // Find index of the first byte of the last block
    let start_idx = num_complete_blocks as usize * MESSAGE_BLOCK_SIZE/ BYTE_SIZE;
            
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

    let msg_size: usizeBits = msg.len()*BYTE_SIZE;
    let num_complete_blocks: usize = msg_size / MESSAGE_BLOCK_SIZE;
    
    for i in 1..num_complete_blocks+1 {
        // 0 .. 64 -> 64 .. 128 -> 128 .. 192 etc.
        let block_data: u512BytesArray = msg.get(NUM_BYTES_512 * i-1 .. NUM_BYTES_512 * i).expect("Failed to split msg as a valid 64 sized slice").try_into().unwrap();
        
        let block: u512 = bytes_to_be_u512(block_data);
        blocks.push(block);
    }

    let incomplete_block_size: usizeBits = msg_size % MESSAGE_BLOCK_SIZE;
    
    // If this is worth 0 this means our message bit size is a multiple of 512, we'll simply add a fully padded block to prevent length extension attacks
    if incomplete_block_size == 0 {
        blocks.push(get_fully_padded_block(msg.len()))
    }
    // Otherwise we'll add padding to the msg last block
    else {
        let last_block: u512 = build_incomplete_block(msg, num_complete_blocks as u32);
        let padded_blocks: (u512, Option<u512>) = add_padding(last_block, incomplete_block_size, msg.len());

        blocks.push(padded_blocks.0);
        if let Some(block) = padded_blocks.1 {
            blocks.push(block);
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_be_u128_test() {
        let bytes: [u8;NUM_BYTES_128] = [
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 1,
        ];
        assert_eq!(bytes_to_be_u128(bytes), 1);

        let bytes: [u8; 16] = [0xFF; 16];
        assert_eq!(bytes_to_be_u128(bytes), u128::MAX);

        let bytes: [u8; 16] = [
            0x12, 0x34, 0x56, 0x78,
            0x9A, 0xBC, 0xDE, 0xF0,
            0x11, 0x22, 0x33, 0x44,
            0x55, 0x66, 0x77, 0x88,
        ];
        assert_eq!(bytes_to_be_u128(bytes), 0x123456789ABCDEF01122334455667788);

        let bytes: [u8; 16] = [0; 16];
        assert_eq!(bytes_to_be_u128(bytes), 0);
    }

    #[test]
    fn get_data_quarters_test() {
        let mut bytes: [u8;NUM_BYTES_512] = [0;NUM_BYTES_512];
        bytes[0] = 0xFF;
        bytes[16] = 0xEE;
        bytes[32] = 0xDD;
        bytes[48] = 0xCC;

        let expected_result: MessageBlockQuarters = (
            bytes[0..16].try_into().unwrap(),
            bytes[16..32].try_into().unwrap(),
            bytes[32..48].try_into().unwrap(),
            bytes[48..64].try_into().unwrap(),
        );
        assert_eq!(get_data_quarters(bytes), expected_result);
    }

    #[test]
    // This test is a bit weak and should be made with more cases
    fn bytes_to_be_u512_test() {
        let mut bytes: [u8;NUM_BYTES_512] = [0;NUM_BYTES_512];
        bytes[0] = 0xFF;
        bytes[16] = 0xEE;
        bytes[32] = 0xDD;
        bytes[48] = 0xCC;

        let result = bytes_to_be_u512(bytes);
        assert_eq!(result[0], 0xFF_000000000000000000000000000000);
        assert_eq!(result[1], 0xEE_000000000000000000000000000000);
        assert_eq!(result[2], 0xDD_000000000000000000000000000000);
        assert_eq!(result[3], 0xCC_000000000000000000000000000000);
    }

    #[test]
    fn get_msg_size_as_be_u64_test() {
        assert_eq!(get_msg_size_as_be_u64(255), 0x0000000000000000_FF00000000000000);
    }

    #[test]
    fn get_fully_padded_block_test() {
        let expected_result:u512 = [0x80000000000000000000000000000000, 0, 0, get_msg_size_as_be_u64(10)];
        assert_eq!(get_fully_padded_block(10), expected_result);
    }

    #[test]
    fn build_incomplete_block_test() {
        // We first need a mock message
        // All values are hardcoded, it makes it easier to predict what we expect
        // and it's less bug prone than writing an algorithm

        const MSG_SIZE: usizeBytes = 148;
        const NUM_COMPLETE_BLOCKS: usize = MSG_SIZE / NUM_BYTES_512;
        const NUM_BYTES_FILL: usize = MSG_SIZE - (NUM_BYTES_512 * NUM_COMPLETE_BLOCKS);

        let mut msg: [u8; MSG_SIZE] = [0;MSG_SIZE];

        // This is the content of the incomplete block
        let incomplete_block: [u8; NUM_BYTES_FILL] = [
            0xCA, 0xCA, 0xF0, 0x0F, 0x34,
            0xEF, 0xAB, 0x3A, 0x10, 0xAA,
            0xBE, 0x33, 0x8C, 0xFF, 0x8A,
            0x3A, 0x28, 0x88, 0xAC, 0xBD
        ];

        // Actually write content to the message
        for i in 0..NUM_BYTES_FILL {
            msg[MSG_SIZE-NUM_BYTES_FILL+i] = incomplete_block[i];
        }

        // Hand validate the result
        let result_block: u512 = build_incomplete_block(&msg, 2);
        assert_eq!(result_block[0], 0xCA_CA_F0_0F_34_EF_AB_3A_10_AA_BE_33_8C_FF_8A_3A);
        assert_eq!(result_block[1], 0x28_88_AC_BD_000000000000000000000000);
    }

    #[test]
    fn add_padding_test() {
        // Let's build a mock message that is exactly 520 bits
        let mut msg: [u8;65] = [0;65];
        msg[64] = 0xAB;
        let last_block: u512 = build_incomplete_block(&msg, 1);
        let padded_blocks: (u512, Option<u512>) = add_padding(last_block, 8, 65);
        
        assert_eq!(padded_blocks.1, None); // We should have enough room in the last block to append the size
        assert_eq!(padded_blocks.0[0], 0xAB_000000000000000000000000000000 | 1 << 119);
        assert_eq!(padded_blocks.0[3], 0x0000000000000000_4100000000000000); // or get_msg_size_as_be_u64(65)

        // Let's rebuild a mock message that is 504 bits now
        let msg: [u8; 63] = [0;63];
        let mut last_block: u512 = build_incomplete_block(&msg, 0);
        let padded_blocks: (u512, Option<u512>) = add_padding(last_block, 504, 63);

        // Append one to the last block
        last_block[3] |= 1 << 7;
        assert_eq!(padded_blocks.0, last_block);
        assert_eq!(padded_blocks.1, Some([0, 0, 0, get_msg_size_as_be_u64(63)]));
    }
}