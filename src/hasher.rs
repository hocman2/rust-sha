use num::{traits::{ToBytes, WrappingAdd}, PrimInt};
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
        let mut h = h.clone();

        for i in 0..MSG_SCHEDULE_SIZE {
            let tmp_1 = h[7].wrapping_add(&sig_1(h[4])).wrapping_add(&Self::choice(h[4], h[5], h[6])).wrapping_add(&self.k[i]).wrapping_add(&w[i]);
            let tmp_2 = sig_0(h[0]).wrapping_add(&Self::majority(h[0], h[1], h[2]));

            h[7] = h[6];
            h[6] = h[5];
            h[5] = h[4];
            h[4] = h[3].wrapping_add(&tmp_1);
            h[3] = h[2];
            h[2] = h[1];
            h[1] = h[0];
            h[0] = tmp_1.wrapping_add(&tmp_2);
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
                h[0] = h_comp[0].wrapping_add(&h[0]);
                h[1] = h_comp[1].wrapping_add(&h[1]);
                h[2] = h_comp[2].wrapping_add(&h[2]);
                h[3] = h_comp[3].wrapping_add(&h[3]);
                h[4] = h_comp[4].wrapping_add(&h[4]);
                h[5] = h_comp[5].wrapping_add(&h[5]);
                h[6] = h_comp[6].wrapping_add(&h[6]);
                h[7] = h_comp[7].wrapping_add(&h[7]);
            }
        
            h
    }
}