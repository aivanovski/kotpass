use sha2::{Digest, Sha256, Sha512};

pub fn sha256(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).to_vec()
}

pub fn sha512(bytes: &[u8]) -> Vec<u8> {
    Sha512::digest(bytes).to_vec()
}

pub fn clear(bytes: &mut [u8]) {
    bytes.fill(0);
}

pub fn constant_time_equals(left: &[u8], right: &[u8]) -> bool {
    let length = left.len().min(right.len());
    let mut not_equal = left.len() ^ right.len();

    for i in 0..length {
        not_equal |= usize::from(left[i] ^ right[i]);
    }

    for byte in &left[length..] {
        not_equal |= usize::from(*byte ^ !*byte);
    }

    not_equal == 0
}

#[cfg(test)]
mod tests {
    use super::{clear, constant_time_equals, sha256, sha512};

    #[test]
    fn hashes_with_sha256() {
        assert_eq!(
            hex::encode(sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hashes_with_sha512() {
        assert_eq!(
            hex::encode(sha512(b"abc")),
            concat!(
                "ddaf35a193617abacc417349ae204131",
                "12e6fa4e89a97ea20a9eeee64b55d39a",
                "2192992a274fc1a836ba3c23a3feebbd",
                "454d4423643ce80e2a9ac94fa54ca49f"
            )
        );
    }

    #[test]
    fn clears_bytes_in_place() {
        let mut bytes = vec![1, 2, 3, 4];

        clear(&mut bytes);

        assert_eq!(bytes, vec![0, 0, 0, 0]);
    }

    #[test]
    fn compares_equal_slices() {
        assert!(constant_time_equals(b"same", b"same"));
        assert!(constant_time_equals(&[], &[]));
    }

    #[test]
    fn rejects_different_content() {
        assert!(!constant_time_equals(b"same", b"sale"));
    }

    #[test]
    fn rejects_different_lengths() {
        assert!(!constant_time_equals(b"same", b"same\0"));
        assert!(!constant_time_equals(b"same\0", b"same"));
    }
}
