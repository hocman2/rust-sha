mod preprocessing;
mod constants;

use constants::K;
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
        let s_0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
        let s_1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
        w[i] = s_1.wrapping_add(w[i-7]).wrapping_add(s_0).wrapping_add(w[i-16]);
    }

    w
}

fn compress_block(a:u32, b:u32, c:u32, d:u32, e:u32, f:u32, g:u32, h:u32, w: &[u32; 64]) -> (u32, u32, u32, u32, u32, u32, u32, u32) {
    // Make mutable
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
        let tmp_1 = h.wrapping_add(s_1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
        let s_0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
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