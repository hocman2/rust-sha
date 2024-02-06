mod preprocessing;
mod constants;

use constants::ROUND_CONSTANTS;
// Shadows f32::constants::SQRT_X from std library
use constants::SQRT_2;
use constants::SQRT_3;
use constants::SQRT_5;
use constants::SQRT_7;
use constants::SQRT_11;
use constants::SQRT_13;
use constants::SQRT_17;
use constants::SQRT_19;

fn create_message_schedule(block: [u32;16]) -> [u32; 64] {
    let mut w: [u32; 64] = [0; 64];

    // Place the block data in the first 16 u32
    for i in 0..16 {
        w[i] = block[i];
    }

    // Expand data to the whole message schedule array
    for i in 16..64 {
        let s_0 = u32::rotate_right(w[i-15], 7) ^ u32::rotate_right(w[i-15], 18) ^ (w[i-15] >> 3);
        let s_1 = u32::rotate_right(w[i-2], 17) ^ u32::rotate_right(w[i-2], 19) ^ (w[i-2] >> 10);
        w[i] = s_1.wrapping_add(w[i-7]).wrapping_add(s_0).wrapping_add(w[i-16]);
    }

    w
}

fn compress_block(a:u32, b:u32, c:u32, d:u32, e:u32, f:u32, g:u32, h:u32, w: &[u32; 64]) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
    let mut a = a;
    let mut b = b;
    let mut c = c;
    let mut d = d;
    let mut e = e;
    let mut f = f;
    let mut g = g;
    let mut h = h;

    for i in 0..64 {
        let s_1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let tmp_1 = h.wrapping_add(s_1).wrapping_add(ch).wrapping_add(ROUND_CONSTANTS[i]).wrapping_add(w[i]);
        let s_0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b ^ c);
        let tmp_2 = s_0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(tmp_1);
        d = c;
        c = b;
        b = a;
        a = tmp_1.wrapping_add(tmp_2);
    }

    (a, b, c, d, e, f, g, h)
}

pub fn hash(message: &[u8]) -> [u8;32] {
    // Initial hash value
    let mut h_: [u32; 8] = [
        SQRT_2, SQRT_3, SQRT_5,
        SQRT_7, SQRT_11, SQRT_13, 
        SQRT_17, SQRT_19
        ];

    let blocks = preprocessing::blockify_msg(message);

    for block in blocks {
        let w = create_message_schedule(block);

        let a = h_[0];
        let b = h_[1];
        let c = h_[2];
        let d = h_[3];
        let e = h_[4];
        let f = h_[5];
        let g = h_[6];
        let h = h_[7];

        let (a, b, c, d, e, f, g, h) = compress_block(a,b,c,d,e,f,g,h, &w);

        // Add the compressed block to the current hash
        h_[0] = h_[0].wrapping_add(a);
        h_[1] = h_[1].wrapping_add(b);
        h_[2] = h_[2].wrapping_add(c);
        h_[3] = h_[3].wrapping_add(d);
        h_[4] = h_[4].wrapping_add(e);
        h_[5] = h_[5].wrapping_add(f);
        h_[6] = h_[6].wrapping_add(g);
        h_[7] = h_[7].wrapping_add(h);
    }

    // Prepare the final hash as a byte array
    let mut hash: [u8; 32] = [0; 32];
    for i in 0..8 {
        let bytes = u32::to_be_bytes(h_[i]);
        hash[i*4] = bytes[0];
        hash[i*4+1] = bytes[1];
        hash[i*4+2] = bytes[2];
        hash[i*4+3] = bytes[3];
    }
    
    hash
}