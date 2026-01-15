//! Configuration loading from environment variables.

use anyhow::{Context, Result};

/// Classe Viva credentials loaded from environment.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    /// Load credentials from environment variables.
    ///
    /// Expects `CLASSEVIVA_USER` and `CLASSEVIVA_PASSWORD` to be set,
    /// either in the environment or in a `.env` file.
    pub fn from_env() -> Result<Self> {
        // Load .env file if present (ignore errors if not found)
        let _ = dotenvy::dotenv();

        let username = std::env::var("CLASSEVIVA_USER")
            .context("CLASSEVIVA_USER environment variable not set")?;

        let password = std::env::var("CLASSEVIVA_PASSWORD")
            .context("CLASSEVIVA_PASSWORD environment variable not set")?;

        Ok(Self { username, password })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Environment variable tests are inherently racy when run in parallel.
    // These tests verify the logic but may interact with a real .env file.
    // Use `cargo test -- --test-threads=1` for deterministic results.

    #[test]
    fn test_credentials_loads_from_env() {
        // This test verifies that credentials can be loaded when env vars are set.
        // It doesn't override existing vars to avoid test pollution.
        std::env::set_var("CLASSEVIVA_USER", "test_user");
        std::env::set_var("CLASSEVIVA_PASSWORD", "test_pass");

        let creds = Credentials::from_env().unwrap();
        // After setting env vars, they should be used (even if .env was loaded)
        assert_eq!(creds.username, "test_user");
        assert_eq!(creds.password, "test_pass");
    }

    #[test]
    fn test_credentials_struct() {
        // Test the struct can be created directly
        let creds = Credentials {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert_eq!(creds.username, "user");
        assert_eq!(creds.password, "pass");
    }
}
