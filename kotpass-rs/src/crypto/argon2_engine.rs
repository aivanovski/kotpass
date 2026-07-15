use argon2::{Algorithm, Argon2, AssociatedData, ParamsBuilder, Version};

use crate::error::CryptoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Argon2Variant {
    Argon2d,
    Argon2i,
    Argon2id,
}

impl Argon2Variant {
    pub fn id(self) -> u32 {
        match self {
            Self::Argon2d => 0x00,
            Self::Argon2i => 0x01,
            Self::Argon2id => 0x02,
        }
    }

    fn algorithm(self) -> Algorithm {
        match self {
            Self::Argon2d => Algorithm::Argon2d,
            Self::Argon2i => Algorithm::Argon2i,
            Self::Argon2id => Algorithm::Argon2id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Argon2Version {
    Ver10,
    Ver13,
}

impl Argon2Version {
    pub fn id(self) -> u32 {
        match self {
            Self::Ver10 => 0x10,
            Self::Ver13 => 0x13,
        }
    }

    pub fn from_id(id: u32) -> Self {
        match id {
            0x13 => Self::Ver13,
            _ => Self::Ver10,
        }
    }

    fn version(self) -> Version {
        match self {
            Self::Ver10 => Version::V0x10,
            Self::Ver13 => Version::V0x13,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argon2Engine {
    variant: Argon2Variant,
    version: Argon2Version,
    salt: Vec<u8>,
    secret: Option<Vec<u8>>,
    additional: Option<Vec<u8>>,
    iterations: u32,
    parallelism: u32,
    memory: u32,
}

impl Argon2Engine {
    pub fn new(
        variant: Argon2Variant,
        version: Argon2Version,
        salt: impl Into<Vec<u8>>,
        secret: Option<Vec<u8>>,
        additional: Option<Vec<u8>>,
        iterations: u32,
        parallelism: u32,
        memory: u32,
    ) -> Self {
        Self {
            variant,
            version,
            salt: salt.into(),
            secret,
            additional,
            iterations,
            parallelism,
            memory,
        }
    }

    pub fn generate_bytes(
        &self,
        password: &[u8],
        out: &mut [u8],
        out_offset: usize,
        out_len: usize,
    ) -> Result<usize, CryptoError> {
        if out_len < 4 {
            return Err(CryptoError::InvalidDataLength(
                "Output length less than 4".to_owned(),
            ));
        }
        let end = out_offset
            .checked_add(out_len)
            .ok_or_else(|| CryptoError::InvalidDataLength("Output buffer too short".to_owned()))?;
        if end > out.len() {
            return Err(CryptoError::InvalidDataLength(
                "Output buffer too short".to_owned(),
            ));
        }

        let mut builder = ParamsBuilder::new();
        builder
            .m_cost(self.memory)
            .t_cost(self.iterations)
            .p_cost(self.parallelism)
            .output_len(out_len);
        if let Some(additional) = &self.additional {
            builder.data(AssociatedData::new(additional).map_err(argon2_error)?);
        }
        let params = builder.build().map_err(argon2_error)?;

        let argon2 = if let Some(secret) = &self.secret {
            Argon2::new_with_secret(
                secret,
                self.variant.algorithm(),
                self.version.version(),
                params,
            )
            .map_err(argon2_error)?
        } else {
            Argon2::new(self.variant.algorithm(), self.version.version(), params)
        };

        argon2
            .hash_password_into(password, &self.salt, &mut out[out_offset..end])
            .map_err(argon2_error)?;

        Ok(out_len)
    }

    pub fn generate_vec(&self, password: &[u8], out_len: usize) -> Result<Vec<u8>, CryptoError> {
        let mut out = vec![0; out_len];
        self.generate_bytes(password, &mut out, 0, out_len)?;
        Ok(out)
    }
}

fn argon2_error(error: argon2::Error) -> CryptoError {
    CryptoError::InvalidKey(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::crypto::{Argon2Engine, Argon2Variant, Argon2Version};

    const PASSWORD: [u8; 32] = [1; 32];
    const SALT: [u8; 16] = [2; 16];
    const SECRET: [u8; 8] = [3; 8];
    const ADDITIONAL: [u8; 12] = [4; 12];

    #[test]
    fn derives_argon2d_kotlin_vector() {
        assert_argon2_vector(
            Argon2Variant::Argon2d,
            "512b391b6f1162975371d30919734294f868e3be3984f3c1a13a4db9fabe4acb",
        );
    }

    #[test]
    fn derives_argon2i_kotlin_vector() {
        assert_argon2_vector(
            Argon2Variant::Argon2i,
            "c814d9d1dc7f37aa13f0d77f2494bda1c8de6b016dd388d29952a4c4672b6ce8",
        );
    }

    #[test]
    fn derives_argon2id_kotlin_vector() {
        assert_argon2_vector(
            Argon2Variant::Argon2id,
            "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659",
        );
    }

    fn assert_argon2_vector(variant: Argon2Variant, expected_hex: &str) {
        let engine = Argon2Engine::new(
            variant,
            Argon2Version::Ver13,
            SALT,
            Some(SECRET.to_vec()),
            Some(ADDITIONAL.to_vec()),
            3,
            4,
            32,
        );

        assert_eq!(
            hex::encode(engine.generate_vec(&PASSWORD, 32).unwrap()),
            expected_hex
        );
    }
}
