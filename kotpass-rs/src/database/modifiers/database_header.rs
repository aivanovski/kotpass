use rand::Rng;

use crate::{
    crypto::{CipherProvider, secure_random::next_bytes},
    error::FormatError,
};

use super::super::{
    KeePassDatabase,
    header::{DatabaseHeader, KdfParameters},
};

impl KeePassDatabase {
    pub fn regenerate_vectors(
        &mut self,
        cipher_providers: &[&dyn CipherProvider],
    ) -> Result<&mut KeePassDatabase, FormatError> {
        let mut rng = rand::rng();
        self.regenerate_vectors_with_rng(&mut rng, cipher_providers)
    }

    pub fn regenerate_vectors_with_rng<R: Rng + ?Sized>(
        &mut self,
        rng: &mut R,
        cipher_providers: &[&dyn CipherProvider],
    ) -> Result<&mut KeePassDatabase, FormatError> {
        let iv_length = cipher_providers
            .iter()
            .find(|provider| provider.uuid() == self.header().cipher_id())
            .map(|provider| provider.iv_len())
            .ok_or_else(|| {
                FormatError::InvalidHeader(format!(
                    "Unsupported cipher ID ({}).",
                    self.header().cipher_id()
                ))
            })?;

        match self {
            Self::Ver3x { header, .. } => {
                let DatabaseHeader::Ver3x {
                    master_seed,
                    encryption_iv,
                    transform_seed,
                    inner_random_stream_key,
                    stream_start_bytes,
                    ..
                } = header
                else {
                    unreachable!("v3 database must contain v3 header")
                };
                *master_seed = next_bytes(rng, 32);
                *encryption_iv = next_bytes(rng, iv_length);
                *transform_seed = next_bytes(rng, 32);
                *inner_random_stream_key = next_bytes(rng, 32);
                *stream_start_bytes = next_bytes(rng, 32);
            }
            Self::Ver4x {
                header,
                inner_header,
                ..
            } => {
                let DatabaseHeader::Ver4x {
                    master_seed,
                    encryption_iv,
                    kdf_parameters,
                    ..
                } = header
                else {
                    unreachable!("v4 database must contain v4 header")
                };
                *master_seed = next_bytes(rng, 32);
                *encryption_iv = next_bytes(rng, iv_length);
                match kdf_parameters {
                    KdfParameters::Aes { seed, .. } => {
                        *seed = next_bytes(rng, 32);
                    }
                    KdfParameters::Argon2 { salt, .. } => {
                        *salt = next_bytes(rng, 32);
                    }
                }
                inner_header.random_stream_key = next_bytes(rng, 64);
            }
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use rand::TryRng;

    use crate::{
        crypto::{BaseCipher, CipherProvider, EncryptedValue},
        database::{
            Credentials, KeePassDatabase,
            header::{DatabaseHeader, KdfParameters},
        },
        error::FormatError,
        model::Meta,
    };

    struct CountingRng(u8);

    impl TryRng for CountingRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            let mut bytes = [0; 4];
            self.try_fill_bytes(&mut bytes)?;
            Ok(u32::from_le_bytes(bytes))
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            let mut bytes = [0; 8];
            self.try_fill_bytes(&mut bytes)?;
            Ok(u64::from_le_bytes(bytes))
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            for byte in dst {
                *byte = self.0;
                self.0 = self.0.wrapping_add(1);
            }
            Ok(())
        }
    }

    fn credentials() -> Credentials {
        Credentials::from_passphrase(&EncryptedValue::from_string("1").unwrap()).unwrap()
    }

    fn providers() -> Vec<&'static dyn CipherProvider> {
        vec![&BaseCipher::Aes]
    }

    #[test]
    fn regenerates_v3_vectors() {
        let mut database =
            KeePassDatabase::create_ver3x("Root", Meta::default(), credentials()).unwrap();
        let mut rng = CountingRng(1);

        database
            .regenerate_vectors_with_rng(&mut rng, &providers())
            .unwrap();

        let DatabaseHeader::Ver3x {
            master_seed,
            encryption_iv,
            transform_seed,
            inner_random_stream_key,
            stream_start_bytes,
            ..
        } = database.header()
        else {
            panic!("expected v3 header");
        };
        assert_eq!(master_seed, &(1..=32).collect::<Vec<_>>());
        assert_eq!(encryption_iv, &(33..=48).collect::<Vec<_>>());
        assert_eq!(transform_seed.len(), 32);
        assert_eq!(inner_random_stream_key.len(), 32);
        assert_eq!(stream_start_bytes.len(), 32);
    }

    #[test]
    fn regenerates_v4_vectors_and_kdf_salt() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let mut rng = CountingRng(1);

        database
            .regenerate_vectors_with_rng(&mut rng, &providers())
            .unwrap();

        let KeePassDatabase::Ver4x {
            header,
            inner_header,
            ..
        } = &database
        else {
            panic!("expected v4 database");
        };
        let DatabaseHeader::Ver4x {
            master_seed,
            encryption_iv,
            kdf_parameters,
            ..
        } = header
        else {
            panic!("expected v4 header");
        };

        assert_eq!(master_seed, &(1..=32).collect::<Vec<_>>());
        assert_eq!(encryption_iv, &(33..=48).collect::<Vec<_>>());
        match kdf_parameters {
            KdfParameters::Argon2 { salt, .. } => {
                assert_eq!(salt, &(49..=80).collect::<Vec<_>>());
            }
            KdfParameters::Aes { .. } => panic!("expected argon2"),
        }
        assert_eq!(inner_header.random_stream_key.len(), 64);
    }

    #[test]
    fn rejects_unknown_cipher_provider() {
        let mut database =
            KeePassDatabase::create_ver4x("Root", Meta::default(), credentials()).unwrap();
        let mut rng = CountingRng(1);

        let error = database
            .regenerate_vectors_with_rng(&mut rng, &[])
            .unwrap_err();

        assert!(matches!(error, FormatError::InvalidHeader(_)));
    }
}
