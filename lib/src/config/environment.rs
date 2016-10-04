use super::ConfigError;

use std::fmt;
use std::str::FromStr;
use std::env;

use self::Environment::*;

pub const CONFIG_ENV: &'static str = "ROCKET_ENV";

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn active() -> Result<Environment, ConfigError> {
        let env_str = env::var(CONFIG_ENV).unwrap_or(Development.to_string());
        env_str.parse().map_err(|_| ConfigError::BadEnv(env_str))
    }

    /// Returns a string with a comma-seperated list of valid environments.
    pub fn valid() -> &'static str {
        "development, staging, production"
    }
}

impl FromStr for Environment {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let env = match s {
            "dev" | "devel" | "development" => Development,
            "stage" | "staging" => Staging,
            "prod" | "production" => Production,
            _ => return Err(()),
        };

        Ok(env)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Development => write!(f, "development"),
            Staging => write!(f, "staging"),
            Production => write!(f, "production"),
        }
    }
}
