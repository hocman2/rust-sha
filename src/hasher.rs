use num::{traits::{ToBytes, WrappingAdd}, PrimInt};

// These are the index for the working vars since they are laid out in an array
#[allow(non_upper_case_globals)]
const a_: usize = 0;
#[allow(non_upper_case_globals)]
const b_: usize = 1;
#[allow(non_upper_case_globals)]
const c_: usize = 2;
#[allow(non_upper_case_globals)]
const d_: usize = 3;
#[allow(non_upper_case_globals)]
const e_: usize = 4;
#[allow(non_upper_case_globals)]
const f_: usize = 5;
#[allow(non_upper_case_globals)]
const g_: usize = 6;
#[allow(non_upper_case_globals)]
const h_: usize = 7;

// A generic struct with functions for SHA hashing.
// Generic parameters were only tested for SHA-compliant values (<u32, 32, 64> and <u64; 64; 80>)
pub struct Hasher<T, const HASH_SIZE_BYTES: usize, const MSG_SCHEDULE_SIZE: usize>
where T: 
    PrimInt +
    WrappingAdd +
    ToBytes
{
    k: [T;MSG_SCHEDULE_SIZE],
}

impl<T, const HASH_SIZE_BYTES: usize, const MSG_SCHEDULE_SIZE: usize> Hasher<T, HASH_SIZE_BYTES, MSG_SCHEDULE_SIZE>
where T:
    PrimInt +
    WrappingAdd +
    ToBytes

{
    pub fn new(k: [T; MSG_SCHEDULE_SIZE]) -> Self {
        Self { k }
    }

    // These are some pretty standard bitwise functions that are used throughout hasing process
    #[inline(always)]
    fn choice(a: T, b: T, c: T) -> T {
        return (a & b) ^ ((!a) & c)
    }

    #[inline(always)]
    fn majority(a: T, b: T, c: T) -> T {
        return (a & b) ^ (a & c) ^ (b & c);
    }

    pub fn create_message_schedule(&self, block: [T;16], sig_0: fn (x: T) -> T, sig_1: fn (x: T) -> T) -> [T; MSG_SCHEDULE_SIZE] {
        let mut w: [T; MSG_SCHEDULE_SIZE] = [T::zero(); MSG_SCHEDULE_SIZE];

        // Place the block data in the first 16 u32
        for i in 0..16 {
            w[i] = block[i];
        }
    
        // Expand data to the whole message schedule array
        for i in 16..MSG_SCHEDULE_SIZE {
            // Luckily this is the same formula for both sha-256 and sha-512
            w[i] = sig_1(w[i-2]).wrapping_add(&w[i-7]).wrapping_add(&sig_0(w[i-15])).wrapping_add(&w[i-16]);
        }
    
        w
    }

    //h are the "a b c d e f g h" vars from the original implementation
    pub fn compress_block(&self, h: &[T;8], w: [T; MSG_SCHEDULE_SIZE], sig_0: fn (x: T) -> T, sig_1: fn (x: T) -> T) -> [T;8] {
        let mut h = h.clone(); // Actually initialize the working variables

        for i in 0..MSG_SCHEDULE_SIZE {
            let tmp_1 = h[h_].wrapping_add(&sig_1(h[e_])).wrapping_add(&Self::choice(h[e_], h[f_], h[g_])).wrapping_add(&self.k[i]).wrapping_add(&w[i]);
            let tmp_2 = sig_0(h[a_]).wrapping_add(&Self::majority(h[a_], h[b_], h[c_]));

            h[h_] = h[g_];
            h[g_] = h[f_];
            h[f_] = h[e_];
            h[f_] = h[d_].wrapping_add(&tmp_1);
            h[f_] = h[c_];
            h[c_] = h[b_];
            h[b_] = h[a_];
            h[a_] = tmp_1.wrapping_add(&tmp_2);
        }

        h
    } 

    pub fn hash(&self, h: [T;8], blocks: Vec<[T;16]>,     
        sig_lc_0: fn (x: T) -> T,
        sig_lc_1: fn (x: T) -> T,
        sig_uc_0: fn (x: T) -> T,
        sig_uc_1: fn (x: T) -> T) -> [T; 8] {

            let mut h = h;
            for block in blocks {
                let w = self.create_message_schedule(block, sig_lc_0, sig_lc_1);
                let h_comp = self.compress_block(&h, w, sig_uc_0, sig_uc_1);
        
                // Add the compressed block to the current hash
                // 🖕 loops
                h[0] = h_comp[a_].wrapping_add(&h[0]);
                h[1] = h_comp[b_].wrapping_add(&h[1]);
                h[2] = h_comp[c_].wrapping_add(&h[2]);
                h[3] = h_comp[d_].wrapping_add(&h[3]);
                h[4] = h_comp[e_].wrapping_add(&h[4]);
                h[5] = h_comp[f_].wrapping_add(&h[5]);
                h[6] = h_comp[g_].wrapping_add(&h[6]);
                h[7] = h_comp[h_].wrapping_add(&h[7]);
            }
        
            h
    }
}