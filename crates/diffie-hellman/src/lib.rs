use aes_gcm::{
    aead::{generic_array::GenericArray, Aead},
    Aes256Gcm, KeyInit,
};
use anyhow::anyhow;
use dstack_core::InnerCryptoHelper;
use x25519_dalek::StaticSecret;

pub struct Crypto;

impl Crypto {
    pub fn new() -> Self {
        Self {}
    }
}

/// Cryptographic helpers for diffie-hellman secret sharing.
/// This should be moved to a default and either be derived or implemented in a wrapped object.
impl InnerCryptoHelper for Crypto {
    type Pubkey = x25519_dalek::PublicKey;
    type Secret = x25519_dalek::StaticSecret;
    type EncryptedMessage = Vec<u8>;

    /// Generates a random keypair.
    fn get_keypair(&self) -> anyhow::Result<(Self::Pubkey, Self::Secret)> {
        let secret = StaticSecret::random();
        let pubkey = x25519_dalek::PublicKey::from(&secret);

        Ok((pubkey, secret))
    }

    /// Decrypts [`message: Self::EncryptedMessage`]:
    /// 1. computes a shared secret (diffie hellman) between the provided public key (shared state pubkey) and secret key.
    /// 2. builds an aes encryption key from that shared secret ensuring that we're
    /// able to decrypt messages signed with the shared secret (note that
    /// shared(S_b, P_a) = shared(S_a, P_b) where S is secret and P is pubkey).
    /// 3. Decrypts the provided message using the provided nonce.
    /// 4. Builds [`Self::Secret`] from the decryption result.
    fn decrypt_secret<N: AsRef<[u8]>>(
        &self,
        nonce: N,
        message: Self::EncryptedMessage,
        pubkeys: Vec<Self::Pubkey>,
        secrets: Vec<Self::Secret>,
    ) -> anyhow::Result<Self::Secret> {
        let expected_shared_pubkey_bytes = pubkeys[0].as_bytes();
        let chiper = {
            let expected_shared_pubkey =
                x25519_dalek::PublicKey::from(*expected_shared_pubkey_bytes);
            let p2p_secret = secrets[0].diffie_hellman(&expected_shared_pubkey);

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(p2p_secret.as_bytes());
            Aes256Gcm::new(key)
        };
        let decrypted = chiper
            .decrypt(GenericArray::from_slice(nonce.as_ref()), message.as_ref())
            .map_err(|e| anyhow!(e))?;
        let shared_secret_bytes: [u8; 32] = decrypted.try_into().unwrap();

        Ok(StaticSecret::from(shared_secret_bytes))
    }

    /// Encrypts [`secret: Self::Secret`]:
    /// 1. computes a shared secret (diffie hellman) between the shared [`secret`]
    /// and the provided public key.
    /// 2. builds an aes encryption key from the shared secret ensuring that we're
    /// able to encrypt messages signed with the shared secret (note that here holds the condition
    /// shared(S_b, P_a) = shared(S_a, P_b) where S is secret and P is pubkey). Only the secret
    /// of the TDX-generated (this condition holds thanks to quote verification) [`pubkeys[0]`]
    /// will be able to compute a shared secret with the global shared pubkey.
    /// 3. We encrypt [`secret`] itself using the previously built key and return the result.
    fn encrypt_secret<N: AsRef<[u8]>>(
        &self,
        nonce: N,
        secret: Self::Secret,
        pubkeys: Vec<Self::Pubkey>,
    ) -> anyhow::Result<Self::EncryptedMessage> {
        let expected_shared_pubkey_bytes = pubkeys[0].as_bytes();
        let chiper = {
            let expected_shared_pubkey =
                x25519_dalek::PublicKey::from(*expected_shared_pubkey_bytes);
            let p2p_secret = secret.diffie_hellman(&expected_shared_pubkey);

            let key = aes_gcm::Key::<Aes256Gcm>::from_slice(p2p_secret.as_bytes());
            Aes256Gcm::new(key)
        };
        let encrypted = chiper
            .encrypt(
                GenericArray::from_slice(nonce.as_ref()),
                secret.as_bytes().as_ref(),
            )
            .map_err(|e| anyhow!(e))?;
        Ok(encrypted)
    }
}
