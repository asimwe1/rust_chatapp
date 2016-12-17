use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::RwLock;
use std::fmt;

use config::Environment::*;
use config::{self, Environment, ConfigError};

use logger::LoggingLevel;
use toml::Value;

/// The core configuration structure.
pub struct Config {
    /// The address to serve on.
    pub address: String,
    /// The port to serve on.
    pub port: usize,
    /// How much information to log.
    pub log_level: LoggingLevel,
    /// The environment that this configuration corresponds to.
    pub env: Environment,
    session_key: RwLock<Option<String>>,
    extras: HashMap<String, Value>,
    filename: String,
}

macro_rules! parse {
    ($conf:expr, $name:expr, $val:expr, $method:ident, $expect: expr) => (
        $val.$method().ok_or_else(|| {
            $conf.bad_type($name, $val, $expect)
        })
    );
}

impl Config {
    pub fn default_for(env: Environment, filename: &str) -> config::Result<Config> {
        let file_path = Path::new(filename);
        if file_path.parent().is_none() {
            return Err(ConfigError::BadFilePath(filename.to_string(),
                "Configuration files must be rooted in a directory."));
        }

        Ok(match env {
            Development => {
                Config {
                    address: "localhost".to_string(),
                    port: 8000,
                    log_level: LoggingLevel::Normal,
                    session_key: RwLock::new(None),
                    extras: HashMap::new(),
                    env: env,
                    filename: filename.to_string(),
                }
            }
            Staging => {
                Config {
                    address: "0.0.0.0".to_string(),
                    port: 80,
                    log_level: LoggingLevel::Normal,
                    session_key: RwLock::new(None),
                    extras: HashMap::new(),
                    env: env,
                    filename: filename.to_string(),
                }
            }
            Production => {
                Config {
                    address: "0.0.0.0".to_string(),
                    port: 80,
                    log_level: LoggingLevel::Critical,
                    session_key: RwLock::new(None),
                    extras: HashMap::new(),
                    env: env,
                    filename: filename.to_string(),
                }
            }
        })
    }

    #[inline(always)]
    fn bad_type(&self, name: &str, val: &Value, expect: &'static str) -> ConfigError {
        let id = format!("{}.{}", self.env, name);
        ConfigError::BadType(id, expect, val.type_str(), self.filename.clone())
    }

    pub fn set(&mut self, name: &str, val: &Value) -> config::Result<()> {
        if name == "address" {
            let address_str = parse!(self, name, val, as_str, "a string")?;
            if address_str.contains(':') {
                return Err(self.bad_type(name, val, "an IP address with no port"));
            } else if format!("{}:{}", address_str, 80).to_socket_addrs().is_err() {
                return Err(self.bad_type(name, val, "a valid IP address"));
            }

            self.address = address_str.to_string();
        } else if name == "port" {
            let port = parse!(self, name, val, as_integer, "an integer")?;
            if port < 0 {
                return Err(self.bad_type(name, val, "an unsigned integer"));
            }

            self.port = port as usize;
        } else if name == "session_key" {
            let key = parse!(self, name, val, as_str, "a string")?;
            if key.len() != 32 {
                return Err(self.bad_type(name, val, "a 192-bit base64 string"));
            }

            self.session_key = RwLock::new(Some(key.to_string()));
        } else if name == "log" {
            let level_str = parse!(self, name, val, as_str, "a string")?;
            self.log_level = match level_str.parse() {
                Ok(level) => level,
                Err(_) => return Err(self.bad_type(name, val,
                                "log level ('normal', 'critical', 'debug')"))
            };
        } else {
            self.extras.insert(name.into(), val.clone());
        }

        Ok(())
    }

    #[inline(always)]
    pub fn take_session_key(&self) -> Option<String> {
        let mut key = self.session_key.write().expect("couldn't lock session key");
        key.take()
    }

    #[inline(always)]
    pub fn extras<'a>(&'a self) -> impl Iterator<Item=(&'a String, &'a Value)> {
        self.extras.iter()
    }

    pub fn get_str<'a>(&'a self, name: &str) -> config::Result<&'a str> {
        let value = self.extras.get(name).ok_or_else(|| ConfigError::NotFound)?;
        parse!(self, name, value, as_str, "a string")
    }

    pub fn get_int(&self, name: &str) -> config::Result<i64> {
        let value = self.extras.get(name).ok_or_else(|| ConfigError::NotFound)?;
        parse!(self, name, value, as_integer, "an integer")
    }

    pub fn get_bool(&self, name: &str) -> config::Result<bool> {
        let value = self.extras.get(name).ok_or_else(|| ConfigError::NotFound)?;
        parse!(self, name, value, as_bool, "a boolean")
    }

    pub fn get_float(&self, name: &str) -> config::Result<f64> {
        let value = self.extras.get(name).ok_or_else(|| ConfigError::NotFound)?;
        parse!(self, name, value, as_float, "a float")
    }

    pub fn root(&self) -> &Path {
        match Path::new(self.filename.as_str()).parent() {
            Some(parent) => parent,
            None => panic!("root(): filename {} has no parent", self.filename)
        }
    }

    // Builder pattern below, mostly for testing.

    #[inline(always)]
    pub fn address(mut self, var: String) -> Self {
        self.address = var;
        self
    }

    #[inline(always)]
    pub fn port(mut self, var: usize) -> Self {
        self.port = var;
        self
    }

    #[inline(always)]
    pub fn log_level(mut self, var: LoggingLevel) -> Self {
        self.log_level = var;
        self
    }

    #[inline(always)]
    pub fn session_key(mut self, var: String) -> Self {
        self.session_key = RwLock::new(Some(var));
        self
    }

    #[inline(always)]
    pub fn env(mut self, var: Environment) -> Self {
        self.env = var;
        self
    }

    #[inline(always)]
    pub fn extra(mut self, name: &str, value: &Value) -> Self {
        self.extras.insert(name.into(), value.clone());
        self
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config[{}] {{ address: {}, port: {}, log_level: {:?} }}",
               self.env, self.address, self.port, self.log_level)
    }
}

impl PartialEq for Config {
    fn eq(&self, other: &Config) -> bool {
        &*self.session_key.read().unwrap() == &*other.session_key.read().unwrap()
            && self.address == other.address
            && self.port == other.port
            && self.log_level == other.log_level
            && self.env == other.env
            && self.extras == other.extras
            && self.filename == other.filename
    }
}

