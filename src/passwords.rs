use anyhow::Result;
use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub site: String,
    pub username: String,
    pub password: String,
}

pub struct Vault {
    dir: PathBuf,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        Self {
            dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("lumen-1_browser")
                .join("vault"),
        }
    }

    fn master_path(&self) -> PathBuf {
        self.dir.join("master")
    }

    fn credentials_path(&self) -> PathBuf {
        self.dir.join("credentials.enc")
    }

    pub fn configured(&self) -> bool {
        self.master_path().exists()
    }

    pub fn setup(&self, password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(anyhow::anyhow!(
                "Das Master-Passwort muss mindestens 8 Zeichen haben"
            ));
        }
        fs::create_dir_all(&self.dir)?;
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let mut hash = [0u8; 32];
        Argon2::default().hash_password_into(password.as_bytes(), &salt, &mut hash)?;
        let mut data = Vec::with_capacity(48);
        data.extend_from_slice(&salt);
        data.extend_from_slice(&hash);
        fs::write(self.master_path(), data)?;
        Ok(())
    }

    fn derive_key(&self, password: &str) -> Result<[u8; 32]> {
        let data = fs::read(self.master_path())?;
        if data.len() != 48 {
            return Err(anyhow::anyhow!("Ungültige Master-Passwort-Datei"));
        }
        let mut key = [0u8; 32];
        Argon2::default().hash_password_into(password.as_bytes(), &data[..16], &mut key)?;
        if key != data[16..] {
            return Err(anyhow::anyhow!("Falsches Master-Passwort"));
        }
        Ok(key)
    }

    pub fn unlock(&self, password: &str) -> Result<CredentialStore> {
        let key = self.derive_key(password)?;
        let path = self.credentials_path();
        let mut credentials = Vec::new();
        if path.exists() {
            let data = fs::read(&path)?;
            if data.len() < 12 {
                return Err(anyhow::anyhow!("Ungültiger Passwort-Tresor"));
            }
            let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
            let plaintext = cipher
                .decrypt(Nonce::from_slice(&data[..12]), &data[12..])
                .map_err(|_| {
                    anyhow::anyhow!("Passwort-Tresor konnte nicht entschlüsselt werden")
                })?;
            credentials = serde_json::from_slice(&plaintext)?;
        }
        Ok(CredentialStore {
            key,
            path,
            credentials,
        })
    }
}

pub struct CredentialStore {
    key: [u8; 32],
    path: PathBuf,
    credentials: Vec<Credential>,
}

impl CredentialStore {
    pub fn credentials(&self) -> &[Credential] {
        &self.credentials
    }

    pub fn add(&mut self, credential: Credential) -> Result<()> {
        self.credentials.push(credential);
        self.save()
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        if index < self.credentials.len() {
            self.credentials.remove(index);
            self.save()?;
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let plain = serde_json::to_vec(&self.credentials)?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&self.key));
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), plain.as_slice())
            .map_err(|_| anyhow::anyhow!("Verschlüsseln fehlgeschlagen"))?;
        let mut data = Vec::with_capacity(12 + ciphertext.len());
        data.extend_from_slice(&nonce);
        data.extend_from_slice(&ciphertext);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, data)?;
        Ok(())
    }
}
