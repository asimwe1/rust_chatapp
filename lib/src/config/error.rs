use super::Environment;

use term_painter::Color::White;
use term_painter::ToStyle;

#[derive(Debug, PartialEq, Clone)]
pub struct ParsingError {
    pub byte_range: (usize, usize),
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub desc: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfigError {
    BadCWD,
    NotFound,
    IOError,
    /// (environment_name)
    BadEnv(String),
    /// (environment_name, filename)
    BadEntry(String, String),
    /// (entry_name, expected_type, actual_type, filename)
    BadType(String, &'static str, &'static str, String),
    /// (toml_source_string, filename, error_list)
    ParseError(String, String, Vec<ParsingError>),
}

impl ConfigError {
    pub fn pretty_print(&self) {
        use self::ConfigError::*;

        let valid_envs = Environment::valid();
        match *self {
            BadCWD => error!("couldn't get current working directory"),
            NotFound => error!("config file was not found"),
            IOError => error!("failed reading the config file: IO error"),
            BadEntry(ref name, ref filename) => {
                error!("[{}] is not a known configuration environment", name);
                info_!("in {}", White.paint(filename));
                info_!("valid environments are: {}", White.paint(valid_envs));
            }
            BadEnv(ref name) => {
                error!("'{}' is not a valid ROCKET_ENV value", name);
                info_!("valid environments are: {}", White.paint(valid_envs));
            }
            BadType(ref name, ref expected, ref actual, ref filename) => {
                error!("'{}' key could not be parsed", name);
                info_!("in {}", White.paint(filename));
                info_!("expected value to be {}, but found {}",
                       White.paint(expected), White.paint(actual));
            }
            ParseError(ref source, ref filename, ref errors) => {
                for error in errors {
                    let (lo, hi) = error.byte_range;
                    let (line, col) = error.start;
                    let error_source = &source[lo..hi];

                    error!("config file could not be parsed as TOML");
                    info_!("at {}:{}:{}", White.paint(filename), line + 1, col + 1);
                    trace_!("'{}' - {}", error_source, White.paint(&error.desc));
                }
            }
        }
    }
}
