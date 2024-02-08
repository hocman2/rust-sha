mod constants;

use crate::preprocessing;
use crate::hasher::Hasher;

// Shadows f64::constants::SQRT_X from std library
use constants::SQRT_2;
use constants::SQRT_3;
use constants::SQRT_5;
use constants::SQRT_7;
use constants::SQRT_11;
use constants::SQRT_13;
use constants::SQRT_17;
use constants::SQRT_19;

#[inline(always)]
fn sig_lc_0(x: u64) -> u64 {
    x.rotate_right(1) ^ x.rotate_right(8) ^ x >> 7
}

#[inline(always)]
fn sig_lc_1(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ x >> 6
}

#[inline(always)]
fn sig_uc_0(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

#[inline(always)]
fn sig_uc_1(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

const FINAL_HASH_BYTES: usize = 64; // <-- 512 bits in bytes

pub fn hash(message: &[u8]) -> [u8;FINAL_HASH_BYTES] {
        
    let blocks = preprocessing::blockify_msg_1024(message);

    // Initial hash value
    let h: [u64; 8] = [
        SQRT_2, SQRT_3, SQRT_5,
        SQRT_7, SQRT_11, SQRT_13, 
        SQRT_17, SQRT_19
        ];
    let hasher: Hasher<u64, 64, 80> = Hasher::new(constants::K);
    let h = hasher.hash(h, blocks, sig_lc_0, sig_lc_1, sig_uc_0, sig_uc_1);

    // Prepare the final hash as a byte array
    let mut hash: [u8; FINAL_HASH_BYTES] = [0; FINAL_HASH_BYTES];
    for i in 0..8 {
        let bytes = u64::to_be_bytes(h[i]);
        hash[i*8] = bytes[0];
        hash[i*8+1] = bytes[1];
        hash[i*8+2] = bytes[2];
        hash[i*8+3] = bytes[3];
        hash[i*8+4] = bytes[4];
        hash[i*8+5] = bytes[5];
        hash[i*8+6] = bytes[6];
        hash[i*8+7] = bytes[7];
    }
    
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility function
    fn hash_to_str(hash: [u8; 64]) -> String {
        let mut hash_str = String::from("");
    
        for i in 0..64 {
            hash_str += &format!("{:02x?}", hash[i]);
        }
    
        return hash_str;
    }

    #[test]
    fn test_hash() {
        assert_eq!(hash_to_str(hash(b"Hello, World!")), "374d794a95cdcfd8b35993185fef9ba368f160d8daf432d08ba9f1ed1e5abe6cc69291e0fa2fe0006a52570ef18c19def4e617c33ce52ef0a6e5fbe318cb0387");
        assert_eq!(hash_to_str(hash(b"prouttestcaca123123123123!")), "f29791dbf4bb98a5f5d94bff3837db53da7c6722b279f405fb0efe246520c4b0313864b06b20c816766880dedd7e01940937305431cb5b48249cd846e72de3f3");
        assert_eq!(hash_to_str(hash(b"Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, justo. Nullam dictum felis eu pede mollis pretium. Integer tincidunt. Cras dapibus. Vivamus elementum semper nisi. Aenean vulputate eleifend tellus. Aenean leo ligula, porttitor eu, consequat vitae, eleifend ac, enim. Aliquam lorem ante, dapibus in, viverra quis, feugiat a, tellus. Phasellus viverra nulla ut metus varius laoreet. Quisque rutrum. Aenean imperdiet. Etiam ultricies nisi vel augue. Curabitur ullamcorper ultricies nisi. Nam eget dui. Etiam rhoncus. Maecenas tempus, tellus eget condimentum rhoncus, sem quam semper libero, sit amet adipiscing sem neque sed ipsum. Nam quam nunc, blandit vel, luctus pulvinar, hendrerit id, lorem. Maecenas nec odio et ante tincidunt tempus. Donec vitae sapien ut libero venenatis faucibus. Nullam quis ante. Etiam sit amet orci eget eros faucibus tincidunt. Duis leo. Sed fringilla mauris sit amet nibh. Donec sodales sagittis magna. Sed consequat, leo eget bibendum sodales, augue velit cursus nunc,")), "32dc3fd7d262ec2a9912e45f009fe61f572093e04f23157c5bfc4b84535ee35be12e4504dd7e211f0832220df65e3d629e441b1726ef31f0a6bfd3531646bfab");
        assert_eq!(hash_to_str(hash(b"It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like).")), "6d96ae87d4e9ef63c1246c9073e3888ec3821b888b13b93c2d95d8fd447d28286f3c863d119955ec52111f82c2f0b46c158c2edf57ab8f7d61eac88e8bd50f87");
    }
}