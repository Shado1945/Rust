use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgonConfig {
    pub memory_cost_kb: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub output_length: usize,
    pub variant: String,
    pub secret_key: Option<String>,
}

impl Default for ArgonConfig {
    fn default() -> Self {
        Self {
            memory_cost_kb: 19456,
            iterations: 2,
            parallelism: 1,
            output_length: 32,
            variant: "argon2id".to_string(),
            secret_key: None,
        }
    }
}

impl ArgonConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Ok(memory) = env::var("ARGON_MEMORY_COST_KB") {
            if let Ok(parsed) = memory.parse::<u32>() {
                config.memory_cost_kb = parsed;
            }
        }

        if let Ok(iterations) = env::var("ARGON_ITERATIONS") {
            if let Ok(parsed) = iterations.parse::<u32>() {
                config.iterations = parsed;
            }
        }

        if let Ok(parallelism) = env::var("ARGON_PARALLELISM") {
            if let Ok(parsed) = parallelism.parse::<u32>() {
                config.parallelism = parsed;
            }
        }

        if let Ok(variant) = env::var("ARGON_VARIANT") {
            config.variant = variant;
        }

        if let Ok(secret) = env::var("ARGON_SECRET_KEY") {
            config.secret_key = Some(secret);
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.memory_cost_kb < 1024 {
            return Err("Memory cost must be at least 1024 KB (1 MB)".to_string());
        }

        if self.memory_cost_kb > 1_048_576 {
            return Err("Memory cost is too high (max 1 GB)".to_string());
        }

        if self.iterations < 1 {
            return Err("Iterations must be at least 1".to_string());
        }

        if self.iterations > 10 {
            return Err("Iterations too high (max 10)".to_string());
        }

        if self.parallelism < 1 {
            return Err("Parallelism must be at least 1".to_string());
        }

        if self.parallelism > 8 {
            return Err("Parallelism too high (max 8)".to_string());
        }

        if self.output_length < 16 {
            return Err("Output length must be at least 16 bytes".to_string());
        }

        if self.output_length > 64 {
            return Err("Output length too high (max 64 bytes)".to_string());
        }

        match self.variant.as_str() {
            "argon2id" | "argon2i" | "argon2d" => Ok(()),
            _ => Err(
                "Invalid Argon2 variant. Must be 'argon2id', 'argon2i', or 'argon2d'".to_string(),
            ),
        }
    }

    #[cfg(feature = "dev")]
    pub fn development() -> Self {
        Self {
            memory_cost_kb: 4096,
            iterations: 1,
            parallelism: 1,
            output_length: 32,
            variant: "argon2id".to_string(),
            secret_key: None,
        }
    }

    #[cfg(feature = "prod")]
    pub fn production() -> Self {
        Self {
            memory_cost_kb: 65536,
            iterations: 3,
            parallelism: 4,
            output_length: 32,
            variant: "argon2id".to_string(),
            secret_key: None,
        }
    }
}
