mod error;
mod environment;
mod config;

use std::fs::{self, File};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process;
use std::env;

pub use self::error::{ConfigError, ParsingError};
pub use self::environment::Environment;
use self::Environment::*;
use self::config::Config;

use toml::{self, Table};
use logger::{self, LoggingLevel};

const CONFIG_FILENAME: &'static str = "Rocket.toml";

#[derive(Debug)]
pub struct RocketConfig {
    pub active_env: Environment,
    config: HashMap<Environment, Config>,
}

impl RocketConfig {
    /// Iteratively search for `file` in `pwd` and its parents, returning the path
    /// to the file or an Error::NoKey if the file couldn't be found.
    fn find() -> Result<PathBuf, ConfigError> {
        let cwd = env::current_dir().map_err(|_| ConfigError::BadCWD)?;
        let mut current = cwd.as_path();

        loop {
            let manifest = current.join(CONFIG_FILENAME);
            if fs::metadata(&manifest).is_ok() {
                return Ok(manifest)
            }

            match current.parent() {
                Some(p) => current = p,
                None => break,
            }
        }

        Err(ConfigError::NotFound)
    }

    fn set(&mut self, env: Environment, kvs: &Table, filename: &str)
            -> Result<(), ConfigError> {
        let config = self.config.entry(env).or_insert(Config::default_for(env));
        for (key, value) in kvs {
            if let Err(expected) = config.set(key, value) {
                let name = format!("{}.{}", env, key);
                return Err(ConfigError::BadType(
                    name, expected, value.type_str(), filename.to_string()
                ))
            }
        }

        Ok(())
    }

    pub fn get(&self, env: Environment) -> &Config {
        if let Some(config) = self.config.get(&env) {
            config
        } else {
            panic!("No value from environment: {:?}", env);
        }
    }

    pub fn active(&self) -> &Config {
        self.get(self.active_env)
    }

    fn parse(src: String, filename: &str) -> Result<RocketConfig, ConfigError> {
        // Parse the source as TOML, if possible.
        let mut parser = toml::Parser::new(&src);
        let toml = parser.parse().ok_or(ConfigError::ParseError(
            src.clone(), filename.into(),
            parser.errors.iter().map(|error| ParsingError {
                byte_range: (error.lo, error.hi),
                start: parser.to_linecol(error.lo),
                end: parser.to_linecol(error.hi),
                desc: error.desc.clone(),
            }).collect()
        ))?;

        // Create a config with the defaults, but the set the env to the active
        let mut config = RocketConfig::active_default()?;

        // Parse the values from the TOML file.
        for (entry, value) in toml {
            // Parse the environment from the table entry name.
            let env = entry.as_str().parse().map_err(|_| {
                ConfigError::BadEntry(entry.clone(), filename.into())
            })?;

            // Each environment must be a table.
            let kv_pairs = match value.as_table() {
                Some(table) => table,
                None => return Err(ConfigError::BadType(
                    entry, "a table", value.type_str(), filename.into()
                ))
            };

            // Set the environment configuration from the kv pairs.
            config.set(env, &kv_pairs, filename)?;
        }

        Ok(config)
    }

    pub fn read() -> Result<RocketConfig, ConfigError> {
        // Find the config file, starting from the `cwd` and working backwords.
        let file = RocketConfig::find()?;

        // Try to open the config file for reading.
        let mut handle = File::open(&file).map_err(|_| ConfigError::IOError)?;

        // Read the configure file to a string for parsing.
        let mut contents = String::new();
        handle.read_to_string(&mut contents).map_err(|_| ConfigError::IOError)?;

        // Parse the contents from the file.
        RocketConfig::parse(contents, &file.to_string_lossy())
    }

    pub fn active_default() -> Result<RocketConfig, ConfigError> {
        let mut default = RocketConfig::default();
        default.active_env = Environment::active()?;
        Ok(default)
    }
}

pub fn read_or_default() -> RocketConfig {
    let bail = |e: ConfigError| -> ! {
        logger::init(LoggingLevel::Debug);
        e.pretty_print();
        process::exit(1)
    };

    use self::ConfigError::*;
    RocketConfig::read().unwrap_or_else(|e| {
        match e {
            ParseError(..) | BadEntry(..) | BadEnv(..) | BadType(..)  => bail(e),
            IOError | BadCWD => warn!("failed reading Rocket.toml. using defaults"),
            NotFound => { /* try using the default below */ }
        }

        RocketConfig::active_default().unwrap_or_else(|e| bail(e))
    })
}

impl Default for RocketConfig {
    fn default() -> RocketConfig {
        RocketConfig {
            active_env: Environment::Development,
            config: {
                let mut default_config = HashMap::new();
                default_config.insert(Development, Config::default_for(Development));
                default_config.insert(Staging, Config::default_for(Staging));
                default_config.insert(Production, Config::default_for(Production));
                default_config
            },
        }
    }
}
