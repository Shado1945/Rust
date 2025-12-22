use crate::config::argon_config::ArgonConfig;
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
        config.validate()?;
        Ok(Self { config })
    }

    fn create_argon2(&self) -> Result<Argon2<'static>> {
        let algorithm = match self.config.variant.as_str() {
            "argon2id" => Algorithm::Argon2id,
            "argon2i" => Algorithm::Argon2i,
            "argon2d" => Algorithm::Argon2d,
            _ => return Err(anyhow!("Invalid Argon2 variant")),
        };

        let params = Params::new(
            self.config.memory_cost_kb,
            self.config.iterations,
            self.config.parallelism,
            Some(self.config.output_length as u32),
        )
        .map_err(|e| anyhow!("Failed to create Argon2 parameters: {}", e))?;

        let argon2 = if let Some(secret) = &self.config.secret_key {
            Argon2::new_with_secret(secret.as_bytes(), algorithm, Version::V0x13, params)
                .map_err(|e| anyhow!("Failed to create Argon2 with secret: {}", e))?
        } else {
            Argon2::new(algorithm, Version::V0x13, params)
        };

        Ok(argon2)
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
                Some(config.output_length as u32),
            )
            .map_err(|e| anyhow!("Failed to create params: {}", e))?;

            let argon2 = if let Some(secret) = &config.secret_key {
                Argon2::new_with_secret(secret.as_bytes(), algorithm, Version::V0x13, params)
                    .map_err(|e| anyhow!("Failed to create Argon2 with secret: {}", e))?
            } else {
                Argon2::new(algorithm, Version::V0x13, params)
            };

            let hash = argon2.hash_password(password.as_bytes(), &salt)?;
            Ok(hash.to_string())
        })
        .await?
    }

    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let config = self.config.clone();
        let password = password.to_string();
        let hash = hash.to_string();

        tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash)?;

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
                Some(config.output_length as u32),
            )
            .map_err(|e| anyhow!("Failed to create params: {}", e))?;

            let argon2 = if let Some(secret) = &config.secret_key {
                Argon2::new_with_secret(secret.as_bytes(), algorithm, Version::V0x13, params)
                    .map_err(|e| anyhow!("Failed to create Argon2 with secret: {}", e))?
            } else {
                Argon2::new(algorithm, Version::V0x13, params)
            };

            match argon2.verify_password(password.as_bytes(), &parsed_hash) {
                Ok(()) => Ok(true),
                Err(PasswordHashError::Password) => Ok(false),
                Err(e) => Err(e.into()),
            }
        })
        .await?
    }

    pub fn needs_rehash(&self, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;

        if let Some(params) = parsed_hash.params {
            let hash_memory = params.get_decimal("m").unwrap_or(0);
            let hash_iterations = params.get_decimal("t").unwrap_or(0);
            let hash_parallelism = params.get_decimal("p").unwrap_or(0);

            if hash_memory < self.config.memory_cost_kb as u64
                || hash_iterations < self.config.iterations as u64
                || hash_parallelism < self.config.parallelism as u64
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get_config(&self) -> &Argon2Config {
        &self.config
    }
}
