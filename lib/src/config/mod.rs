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

#[derive(Debug, PartialEq, Clone)]
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

/// Read the Rocket config file from the current directory or any of its
/// parents. If there is no such file, return the default config. A returned
/// config will have its active environment set to whatever was passed in with
/// the rocket config env variable. If there is a problem doing any of this,
/// print a nice error message and bail.
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

#[cfg(test)]
mod test {
    use std::env;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use super::{RocketConfig, CONFIG_FILENAME, ConfigError};
    use super::environment::CONFIG_ENV;
    use super::Environment::*;
    use super::config::Config;

    use ::toml::Value;
    use ::logger::LoggingLevel;

    lazy_static! {
        static ref ENV_LOCK: Mutex<usize> = Mutex::new(0);
    }

    macro_rules! check_config {
        ($rconfig:expr => { $($param:tt)+ }) => (
            match $rconfig {
                Ok(config) => assert_eq!(config.active(), &Config { $($param)+ }),
                Err(e) => panic!("Config {} failed: {:?}", stringify!($rconfig), e)
            }
        );

        ($rconfig:expr, $econfig:expr) => (
            match $rconfig {
                Ok(config) => assert_eq!(config.active(), &$econfig),
                Err(e) => panic!("Config {} failed: {:?}", stringify!($rconfig), e)
            }
        );

        ($env:expr, $rconfig:expr, $econfig:expr) => (
            match $rconfig.clone() {
                Ok(config) => assert_eq!(config.get($env), &$econfig),
                Err(e) => panic!("Config {} failed: {:?}", stringify!($rconfig), e)
            }
        );
    }

    #[test]
    fn test_defaults() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();

        // First, without an environment. Should get development defaults.
        env::remove_var(CONFIG_ENV);
        check_config!(RocketConfig::active_default(), Config::default_for(Development));

        // Now with an explicit dev environment.
        for env in &["development", "dev"] {
            env::set_var(CONFIG_ENV, env);
            check_config!(RocketConfig::active_default(), Config::default_for(Development));
        }

        // Now staging.
        for env in &["stage", "staging"] {
            env::set_var(CONFIG_ENV, env);
            check_config!(RocketConfig::active_default(), Config::default_for(Staging));
        }

        // Finally, production.
        for env in &["prod", "production"] {
            env::set_var(CONFIG_ENV, env);
            check_config!(RocketConfig::active_default(), Config::default_for(Production));
        }
    }

    #[test]
    fn test_bad_environment_vars() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();

        for env in &["", "p", "pr", "pro", "prodo", " prod", "dev ", "!dev!", "🚀 "] {
            env::set_var(CONFIG_ENV, env);
            let err = ConfigError::BadEnv(env.to_string());
            assert!(RocketConfig::active_default().err().map_or(false, |e| e == err));
        }

        // Test that a bunch of invalid environment names give the right error.
        env::remove_var(CONFIG_ENV);
        for env in &["p", "pr", "pro", "prodo", "bad", "meow", "this", "that"] {
            let toml_table = format!("[{}]\n", env);
            let err = ConfigError::BadEntry(env.to_string(), CONFIG_FILENAME.into());
            assert!(RocketConfig::parse(toml_table, CONFIG_FILENAME)
                    .err().map_or(false, |e| e == err));
        }
    }

    #[test]
    fn test_good_full_config_files() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        let config_str = r#"
            address = "1.2.3.4"
            port = 7810
            log = "critical"
            session_key = "01234567890123456789012345678901"
            template_dir = "mine"
            json = true
            pi = 3.14
        "#;

        let mut extra: HashMap<String, Value> = HashMap::new();
        extra.insert("template_dir".to_string(), Value::String("mine".into()));
        extra.insert("json".to_string(), Value::Boolean(true));
        extra.insert("pi".to_string(), Value::Float(3.14));

        let expected = Config {
            address: "1.2.3.4".to_string(),
            port: 7810,
            log_level: LoggingLevel::Critical,
            session_key: Some("01234567890123456789012345678901".to_string()),
            extra: extra
        };

        let dev_config = ["[dev]", config_str].join("\n");
        let parsed = RocketConfig::parse(dev_config, CONFIG_FILENAME);
        check_config!(Development, parsed, expected);
        check_config!(Staging, parsed, Config::default_for(Staging));
        check_config!(Production, parsed, Config::default_for(Production));

        let stage_config = ["[stage]", config_str].join("\n");
        let parsed = RocketConfig::parse(stage_config, CONFIG_FILENAME);
        check_config!(Staging, parsed, expected);
        check_config!(Development, parsed, Config::default_for(Development));
        check_config!(Production, parsed, Config::default_for(Production));

        let prod_config = ["[prod]", config_str].join("\n");
        let parsed = RocketConfig::parse(prod_config, CONFIG_FILENAME);
        check_config!(Production, parsed, expected);
        check_config!(Development, parsed, Config::default_for(Development));
        check_config!(Staging, parsed, Config::default_for(Staging));
    }

    #[test]
    fn test_good_address_values() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::set_var(CONFIG_ENV, "dev");

        check_config!(RocketConfig::parse(r#"
                          [development]
                          address = "localhost"
                      "#.to_string(), CONFIG_FILENAME) => {
                          address: "localhost".to_string(),
                          ..Config::default_for(Development)
                      });

        check_config!(RocketConfig::parse(r#"
                          [dev]
                          address = "127.0.0.1"
                      "#.to_string(), CONFIG_FILENAME) => {
                          address: "127.0.0.1".to_string(),
                          ..Config::default_for(Development)
                      });

        check_config!(RocketConfig::parse(r#"
                          [dev]
                          address = "0.0.0.0"
                      "#.to_string(), CONFIG_FILENAME) => {
                          address: "0.0.0.0".to_string(),
                          ..Config::default_for(Development)
                      });
    }


    #[test]
    fn test_bad_address_values() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        assert!(RocketConfig::parse(r#"
            [development]
            address = 0000
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [development]
            address = true
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [development]
            address = "_idk_"
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [staging]
            address = "1.2.3.4:100"
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [production]
            address = "1.2.3.4.5.6"
        "#.to_string(), CONFIG_FILENAME).is_err());
    }

    #[test]
    fn test_good_port_values() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::set_var(CONFIG_ENV, "stage");

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          port = 100
                      "#.to_string(), CONFIG_FILENAME) => {
                          port: 100,
                          ..Config::default_for(Staging)
                      });

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          port = 6000
                      "#.to_string(), CONFIG_FILENAME) => {
                          port: 6000,
                          ..Config::default_for(Staging)
                      });
    }

    #[test]
    fn test_bad_port_values() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        assert!(RocketConfig::parse(r#"
            [development]
            port = true
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [production]
            port = "hello"
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [staging]
            port = -1
        "#.to_string(), CONFIG_FILENAME).is_err());
    }

    #[test]
    fn test_good_log_levels() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::set_var(CONFIG_ENV, "stage");

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          log = "normal"
                      "#.to_string(), CONFIG_FILENAME) => {
                          log_level: LoggingLevel::Normal,
                          ..Config::default_for(Staging)
                      });


        check_config!(RocketConfig::parse(r#"
                          [stage]
                          log = "debug"
                      "#.to_string(), CONFIG_FILENAME) => {
                          log_level: LoggingLevel::Debug,
                          ..Config::default_for(Staging)
                      });

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          log = "critical"
                      "#.to_string(), CONFIG_FILENAME) => {
                          log_level: LoggingLevel::Critical,
                          ..Config::default_for(Staging)
                      });
    }

    #[test]
    fn test_bad_log_level_values() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        assert!(RocketConfig::parse(r#"
            [dev]
            log = false
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [development]
            log = 0
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [prod]
            log = "no"
        "#.to_string(), CONFIG_FILENAME).is_err());
    }

    #[test]
    fn test_good_session_key() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::set_var(CONFIG_ENV, "stage");

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          session_key = "VheMwXIBygSmOlZAhuWl2B+zgvTN3WW5"
                      "#.to_string(), CONFIG_FILENAME) => {
                          session_key: Some("VheMwXIBygSmOlZAhuWl2B+zgvTN3WW5".into()),
                          ..Config::default_for(Staging)
                      });

        check_config!(RocketConfig::parse(r#"
                          [stage]
                          session_key = "adL5fFIPmZBrlyHk2YT4NLV3YCk2gFXz"
                      "#.to_string(), CONFIG_FILENAME) => {
                          session_key: Some("adL5fFIPmZBrlyHk2YT4NLV3YCk2gFXz".into()),
                          ..Config::default_for(Staging)
                      });
    }

    #[test]
    fn test_bad_session_key() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        assert!(RocketConfig::parse(r#"
            [dev]
            session_key = true
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [dev]
            session_key = 1283724897238945234897
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [dev]
            session_key = "abcv"
        "#.to_string(), CONFIG_FILENAME).is_err());
    }

    #[test]
    fn test_bad_toml() {
        // Take the lock so changing the environment doesn't cause races.
        let _env_lock = ENV_LOCK.lock().unwrap();
        env::remove_var(CONFIG_ENV);

        assert!(RocketConfig::parse(r#"
            [dev
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [dev]
            1.2.3 = 2
        "#.to_string(), CONFIG_FILENAME).is_err());

        assert!(RocketConfig::parse(r#"
            [dev]
            session_key = "abcv" = other
        "#.to_string(), CONFIG_FILENAME).is_err());
    }
}
