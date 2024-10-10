use core::str;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
    Key, // Or `Aes128Gcm`
    Nonce,
};
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use sqlx::PgPool;

pub struct Account {
    pub id: i32,
    pub keypair: Keypair,
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    pub die_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AccountClaims {
    pub sub: String,
    pub exp: usize,
}

impl Account {
    pub fn new_unique(masterkey: &[u8], duration: chrono::Duration) -> anyhow::Result<Self> {
        let keypair = Keypair::new();
        let (ciphertext, nonce) = seal(keypair.to_base58_string().as_bytes(), masterkey)?;

        Ok(Self {
            id: 0,
            keypair,
            ciphertext,
            nonce,
            die_at: chrono::Utc::now() + duration,
        })
    }

    pub async fn save(&mut self, pool: &PgPool) -> anyhow::Result<i32> {
        let rec = sqlx::query!(
            "INSERT INTO accounts(ciphertext,nonce,die_at) values ($1,$2,$3) returning id",
            self.ciphertext,
            self.nonce,
            self.die_at.timestamp()
        )
        .fetch_one(pool)
        .await?;
        self.id = rec.id;
        Ok(rec.id)
    }

    pub async fn find_one(pool: &PgPool, id: i32, masterkey: &[u8]) -> anyhow::Result<Self> {
        dbg!("{}", chrono::Utc::now().timestamp());
        dbg!(id);
        let rec = sqlx::query!(
            "SELECT ciphertext, nonce FROM accounts WHERE id = $1 and die_at > $2",
            id,
            chrono::Utc::now().timestamp()
        )
        .fetch_one(pool)
        .await?;
        let secret = unseal(&rec.ciphertext, masterkey, &rec.nonce)?;
        let keypair = Keypair::from_base58_string(str::from_utf8(&secret)?);
        Ok(Self {
            id,
            keypair,
            ciphertext: vec![],
            nonce: vec![],
            die_at: chrono::Utc::now(),
        })
    }

    pub fn access_token(&self, jwt_secret: &[u8]) -> anyhow::Result<String> {
        let claims = AccountClaims {
            sub: self.id.to_string(),
            exp: self.die_at.timestamp() as usize,
        };
        let token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret),
        )?;

        Ok(token)
    }

    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }
}

fn seal(secret: &[u8], key: &[u8]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    // The encryption key can be generated randomly:
    // let key = Aes256Gcm::generate_key(OsRng);

    // // Transformed from a byte array:
    // let key: &[u8; 32] = &[42; 32];
    // let key: &Key<Aes256Gcm> = key.into();

    // // Note that you can get byte array from slice using the `TryInto` trait:
    // let key: &[u8] = &[42; 32];
    // let key: [u8; 32] = key.try_into()?;

    // Alternatively, the key can be transformed directly from a byte slice
    // (panicks on length mismatch):
    let key = Key::<Aes256Gcm>::from_slice(key);

    let cipher = Aes256Gcm::new(key);

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let ciphertext = cipher.encrypt(&nonce, secret).map_err(|e| {
        anyhow::anyhow!(e).context("error while trying to encrypt a plaintext message")
    })?;

    Ok((ciphertext, nonce.to_vec()))
}

fn unseal(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> anyhow::Result<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let secret = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|e| {
            anyhow::anyhow!(e).context("error while trying to decrypt a ciphertext message")
        })?;

    Ok(secret)
}
