use rand::{Rng, TryRng};

pub fn next_bytes<R>(rng: &mut R, length: usize) -> Vec<u8>
where
    R: Rng + ?Sized,
{
    let mut bytes = vec![0; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn secure_random_bytes(length: usize) -> Result<Vec<u8>, rand::rngs::SysError> {
    let mut bytes = vec![0; length];
    rand::rngs::SysRng.try_fill_bytes(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use rand::TryRng;

    use super::{next_bytes, secure_random_bytes};

    struct CountingRng(u8);

    impl TryRng for CountingRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            let bytes = next_bytes(self, 4);
            Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            let bytes = next_bytes(self, 8);
            Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dst {
                *byte = self.0;
                self.0 = self.0.wrapping_add(1);
            }
            Ok(())
        }
    }

    #[test]
    fn fills_bytes_from_rng() {
        let mut rng = CountingRng(3);

        assert_eq!(next_bytes(&mut rng, 5), vec![3, 4, 5, 6, 7]);
        assert_eq!(next_bytes(&mut rng, 3), vec![8, 9, 10]);
    }

    #[test]
    fn handles_empty_output() {
        let mut rng = CountingRng(3);

        assert!(next_bytes(&mut rng, 0).is_empty());
    }

    #[test]
    fn creates_secure_random_bytes_with_requested_length() {
        assert_eq!(secure_random_bytes(16).unwrap().len(), 16);
    }
}
