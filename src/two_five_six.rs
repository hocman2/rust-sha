mod constants;

use crate::preprocessing;
use crate::hasher::Hasher;

// Shadows f32::constants::SQRT_X from std library
use constants::SQRT_2;
use constants::SQRT_3;
use constants::SQRT_5;
use constants::SQRT_7;
use constants::SQRT_11;
use constants::SQRT_13;
use constants::SQRT_17;
use constants::SQRT_19;

#[inline(always)]
fn sig_lc_0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ x >> 3
}

#[inline(always)]
fn sig_lc_1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ x >> 10
}

#[inline(always)]
fn sig_uc_0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

#[inline(always)]
fn sig_uc_1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

const FINAL_HASH_BYTES: usize = 32; // <-- 256 bits in bytes

pub fn hash(message: &[u8]) -> [u8;FINAL_HASH_BYTES] {
        
    let blocks = preprocessing::blockify_msg_512(message);

    // Initial hash value
    let h: [u32; 8] = [
        SQRT_2, SQRT_3, SQRT_5,
        SQRT_7, SQRT_11, SQRT_13, 
        SQRT_17, SQRT_19
        ];

    let hasher: Hasher<u32, FINAL_HASH_BYTES, 64> = Hasher::new(constants::K);
    
    let h = hasher.hash(h, blocks, sig_lc_0, sig_lc_1, sig_uc_0, sig_uc_1);

    // turn into byte array
    let mut hash: [u8; FINAL_HASH_BYTES] = [0; FINAL_HASH_BYTES];
    for i in 0..8 {
        let bytes = u32::to_be_bytes(h[i]);
        hash[i*4] = bytes[0];
        hash[i*4+1] = bytes[1];
        hash[i*4+2] = bytes[2];
        hash[i*4+3] = bytes[3];
    }
    
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility function
    fn hash_to_str(hash: [u8; 32]) -> String {
        let mut hash_str = String::from("");
    
        for i in 0..32 {
            hash_str += &format!("{:02x?}", hash[i]);
        }
    
        return hash_str;
    }

    #[test]
    fn test_hash() {
        assert_eq!(hash_to_str(hash(b"Hello, World!")), "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
        assert_eq!(hash_to_str(hash(b"prouttestcaca123123123123!")), "3b766381d00172424686287d7ec9d1154617cacac9953b1847db035e4c8479c1");
        assert_eq!(hash_to_str(hash(b"Lorem ipsum dolor sit amet, consectetuer adipiscing elit. Aenean commodo ligula eget dolor. Aenean massa. Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Donec quam felis, ultricies nec, pellentesque eu, pretium quis, sem. Nulla consequat massa quis enim. Donec pede justo, fringilla vel, aliquet nec, vulputate eget, arcu. In enim justo, rhoncus ut, imperdiet a, venenatis vitae, justo. Nullam dictum felis eu pede mollis pretium. Integer tincidunt. Cras dapibus. Vivamus elementum semper nisi. Aenean vulputate eleifend tellus. Aenean leo ligula, porttitor eu, consequat vitae, eleifend ac, enim. Aliquam lorem ante, dapibus in, viverra quis, feugiat a, tellus. Phasellus viverra nulla ut metus varius laoreet. Quisque rutrum. Aenean imperdiet. Etiam ultricies nisi vel augue. Curabitur ullamcorper ultricies nisi. Nam eget dui. Etiam rhoncus. Maecenas tempus, tellus eget condimentum rhoncus, sem quam semper libero, sit amet adipiscing sem neque sed ipsum. Nam quam nunc, blandit vel, luctus pulvinar, hendrerit id, lorem. Maecenas nec odio et ante tincidunt tempus. Donec vitae sapien ut libero venenatis faucibus. Nullam quis ante. Etiam sit amet orci eget eros faucibus tincidunt. Duis leo. Sed fringilla mauris sit amet nibh. Donec sodales sagittis magna. Sed consequat, leo eget bibendum sodales, augue velit cursus nunc,")), "4d0fcee44bd65ea0a0c983da992b053d6f5d94a25e91eae6a783f59fb5ef0cc1");
        assert_eq!(hash_to_str(hash(b"It is a long established fact that a reader will be distracted by the readable content of a page when looking at its layout. The point of using Lorem Ipsum is that it has a more-or-less normal distribution of letters, as opposed to using 'Content here, content here', making it look like readable English. Many desktop publishing packages and web page editors now use Lorem Ipsum as their default model text, and a search for 'lorem ipsum' will uncover many web sites still in their infancy. Various versions have evolved over the years, sometimes by accident, sometimes on purpose (injected humour and the like).")), "703190109e4e00d7d5a61fa3df9919da8dd57a3eb53c5b321b4841bad7212ed8");
    }
}