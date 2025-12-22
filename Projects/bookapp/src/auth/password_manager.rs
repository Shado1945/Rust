use crate::config::argon_config::ArgonConfig;
use anyhow::{Result, anyhow};
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{
        Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};

pub struct PasswordManager {
    config: ArgonConfig,
}

impl PasswordManager {
    pub fn new(config: ArgonConfig) -> Result<Self> {
        config
            .validate()
            .map_err(|e| anyhow!("Invalid Argon configuration: {}", e))?;
        Ok(Self { config })
    }

    pub async fn hash_password(&self, password: &str) -> Result<String> {
        let config = self.config.clone();
        let password = password.to_string();

        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);

            let algorithm = match config.variant.as_str() {
                "argon2id" => Algorithm::Argon2id,
                "argon2i" => Algorithm::Argon2i,
                "argon2d" => Algorithm::Argon2d,
                _ => return Err(anyhow!("Invalid variant")),
            };

            let params = Params::new(
                config.memory_cost_kb,
                config.iterations,
                config.parallelism,
                Some(config.output_length),
            )
            .map_err(|e| anyhow!("Failed to create params: {}", e))?;

            // FIX: Create Argon2 WITHOUT secret for now
            // Remove secret support since it's causing lifetime issues
            let argon2 = Argon2::new(algorithm, Version::V0x13, params);

            let hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| anyhow!("Hashing failed: {}", e))?;
            Ok(hash.to_string())
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
    }

    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let config = self.config.clone();
        let password = password.to_string();
        let hash = hash.to_string();

        tokio::task::spawn_blocking(move || {
            let parsed_hash =
                PasswordHash::new(&hash).map_err(|e| anyhow!("Failed to parse hash: {}", e))?;

            let algorithm = match config.variant.as_str() {
                "argon2id" => Algorithm::Argon2id,
                "argon2i" => Algorithm::Argon2i,
                "argon2d" => Algorithm::Argon2d,
                _ => return Err(anyhow!("Invalid variant")),
            };

            let params = Params::new(
                config.memory_cost_kb,
                config.iterations,
                config.parallelism,
                Some(config.output_length),
            )
            .map_err(|e| anyhow!("Failed to create params: {}", e))?;

            // FIX: Create Argon2 WITHOUT secret for now
            let argon2 = Argon2::new(algorithm, Version::V0x13, params);

            match argon2.verify_password(password.as_bytes(), &parsed_hash) {
                Ok(()) => Ok(true),
                Err(PasswordHashError::Password) => Ok(false),
                Err(e) => Err(anyhow!("Verification error: {}", e)),
            }
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
    }

    pub fn needs_rehash(&self, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Failed to parse hash: {}", e))?;

        match parsed_hash.params {
            params => {
                let hash_memory = params.get_decimal("m").unwrap_or(0);
                let hash_iterations = params.get_decimal("t").unwrap_or(0);
                let hash_parallelism = params.get_decimal("p").unwrap_or(0);

                if hash_memory < self.config.memory_cost_kb
                    || hash_iterations < self.config.iterations
                    || hash_parallelism < self.config.parallelism
                {
                    return Ok(true);
                }
            }
            _ => (),
        }

        Ok(false)
    }

    pub fn get_config(&self) -> &ArgonConfig {
        &self.config
    }
}
