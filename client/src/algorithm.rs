//! State representations of recognized encryption algorithms

use {
    spl_discriminator::{ArrayDiscriminator, SplDiscriminate},
    spl_keyring_program::tlv::{
        KeystoreEntry, KeystoreEntryConfig, KeystoreEntryConfigEntry, KeystoreEntryKey,
    },
};

/// A trait for defining recognized encryption algorithms
pub trait EncryptionAlgorithm: SplDiscriminate {
    /// The length of the encryption key in bytes
    const KEY_LENGTH: usize;
    /// Returns the key
    fn key(&self) -> Vec<u8>;
    /// Returns the config
    fn config(&self) -> Box<dyn Configurations>;
    /// Converts an encryption algorithm to a buffer
    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(Self::SPL_DISCRIMINATOR_SLICE);
        buffer.extend_from_slice(&Self::KEY_LENGTH.to_le_bytes());
        buffer.extend_from_slice(&self.key());
        buffer.extend_from_slice(&self.config().to_buffer());
        buffer
    }
    /// Converts an encryption algorithm to a keystore entry
    fn to_keystore_entry(&self) -> KeystoreEntry {
        KeystoreEntry {
            key: KeystoreEntryKey {
                discriminator: Self::SPL_DISCRIMINATOR,
                key_length: Self::KEY_LENGTH as u32,
                key: self.key(),
            },
            config: self.config().to_keystore_entry_config(),
        }
    }
}

/// A trait representing the configurations of an encryption algorithm
pub trait Configurations {
    /// Converts configurations to a buffer
    fn to_buffer(&self) -> Vec<u8>;
    /// Converts configurations to a `KeystoreEntryConfig`
    fn to_keystore_entry_config(&self) -> Option<KeystoreEntryConfig>;
}

/// Struct representing "no configurations" required for a particular encryption
/// algorithm
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NoConfigurations;

impl Configurations for NoConfigurations {
    fn to_buffer(&self) -> Vec<u8> {
        vec![0]
    }
    fn to_keystore_entry_config(&self) -> Option<KeystoreEntryConfig> {
        None
    }
}

/// Curve25519 encryption algorithm
#[derive(Clone, Debug, PartialEq, SplDiscriminate)]
#[discriminator_hash_input("spl_keyring_program:key:Curve25519")]
pub struct Curve25519([u8; 32]);
impl Curve25519 {
    /// Create a new instance of Curve25519
    pub fn new(key: [u8; 32]) -> Self {
        Self(key)
    }
}

impl EncryptionAlgorithm for Curve25519 {
    const KEY_LENGTH: usize = 32;

    fn key(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn config(&self) -> Box<dyn Configurations> {
        Box::<NoConfigurations>::default()
    }
}

/// Rsa encryption algorithm
#[derive(Clone, Debug, PartialEq, SplDiscriminate)]
#[discriminator_hash_input("spl_keyring_program:key:RSA")]
pub struct Rsa([u8; 64]);
impl Rsa {
    /// Create a new instance of Rsa
    pub fn new(key: [u8; 64]) -> Self {
        Self(key)
    }
}

impl EncryptionAlgorithm for Rsa {
    const KEY_LENGTH: usize = 64;

    fn key(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn config(&self) -> Box<dyn Configurations> {
        Box::<NoConfigurations>::default()
    }
}

/// ComplexAlgorithm encryption algorithm
#[derive(Clone, Debug, Default, PartialEq, SplDiscriminate)]
#[discriminator_hash_input("spl_keyring_program:key:ComplexAlgorithm")]
pub struct ComplexAlgorithm {
    key: [u8; 32],
    config: ComplexAlgorithmConfigurations,
}
impl ComplexAlgorithm {
    /// Create a new instance of ComplexAlgorithm
    pub fn new(key: [u8; 32], nonce: [u8; 12], aad: [u8; 12]) -> Self {
        Self {
            key,
            config: ComplexAlgorithmConfigurations { nonce, aad },
        }
    }
}

impl EncryptionAlgorithm for ComplexAlgorithm {
    const KEY_LENGTH: usize = 32;

    fn key(&self) -> Vec<u8> {
        self.key.to_vec()
    }

    fn config(&self) -> Box<dyn Configurations> {
        Box::new(self.config.clone())
    }
}

/// ComplexAlgorithm configurations
#[derive(Clone, Debug, Default, PartialEq, SplDiscriminate)]
#[discriminator_hash_input("spl_keyring_program:configuration:ComplexAlgorithm")]
pub struct ComplexAlgorithmConfigurations {
    /// The nonce used for encryption
    pub nonce: [u8; 12],
    /// The additional authenticated data
    pub aad: [u8; 12],
}

impl Configurations for ComplexAlgorithmConfigurations {
    fn to_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&b"nonce"[..8]);
        buffer.extend_from_slice(&self.nonce.len().to_le_bytes());
        buffer.extend_from_slice(&self.nonce);
        buffer.extend_from_slice(&b"aad"[..8]);
        buffer.extend_from_slice(&self.aad.len().to_le_bytes());
        buffer.extend_from_slice(&self.aad);
        buffer
    }

    fn to_keystore_entry_config(&self) -> Option<KeystoreEntryConfig> {
        // 8 Bytes
        let nonce_discriminator = {
            let mut buffer = [0; 8];
            buffer.copy_from_slice(&b"nonce"[..8]);
            ArrayDiscriminator::new(buffer)
        };
        // 8 Bytes
        let aad_discriminator = {
            let mut buffer = [0; 8];
            buffer.copy_from_slice(&b"aad"[..8]);
            ArrayDiscriminator::new(buffer)
        };
        Some(KeystoreEntryConfig(vec![
            KeystoreEntryConfigEntry {
                key: nonce_discriminator,
                value_length: self.nonce.len() as u32,
                value: self.nonce.to_vec(),
            },
            KeystoreEntryConfigEntry {
                key: aad_discriminator,
                value_length: self.aad.len() as u32,
                value: self.aad.to_vec(),
            },
        ]))
    }
}
