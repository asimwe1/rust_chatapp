/// A base `Config` for any `Poolable` type.
///
/// For the following configuration:
///
/// ```toml
/// [global.databases.my_database]
/// url = "postgres://root:root@localhost/my_database"
/// pool_size = 10
/// timeout = 5
/// ```
///
/// ...`Config::from("my_database", rocket)` would return the following struct:
///
/// ```rust
/// # use rocket_contrib::databases::Config;
/// Config {
///     url: "postgres://root:root@localhost/my_database".into(),
///     pool_size: 10,
///     timeout: 5
/// };
/// ```
///
/// If you want to implement your own custom database adapter (or other
/// database-like struct that can be pooled by `r2d2`) and need some more
/// configurations options, you may need to define a custom `Config` struct.
/// Note, however, that the configuration values in `Config` are required.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    /// Connection URL specified in the Rocket configuration.
    pub url: String,
    /// Initial pool size. Defaults to the number of Rocket workers * 2.
    pub pool_size: u32,
    /// How long to wait, in seconds, for a new connection before timing out.
    /// Defaults to `5`.
    // FIXME: Use `time`.
    pub timeout: u8,
}

use serde::{Serialize, Deserialize};
use rocket::figment::{self, Figment, providers::Serialized};

impl Config {
    /// Retrieves the database configuration for the database named `name`.
    ///
    /// This function is primarily used by the generated code from the
    /// `#[database]` attribute.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "diesel_sqlite_pool")] {
    /// # use rocket::figment::{Figment, providers::{Format, Toml}};
    /// // Assume that these are the contents of `Rocket.toml`:
    /// # let toml = Toml::string(r#"
    /// [global.databases]
    /// my_db = { url = "db/db.sqlite", pool_size = 25 }
    /// my_other_db = { url = "mysql://root:root@localhost/database" }
    /// # "#).nested();
    ///
    /// use rocket_contrib::databases::Config;
    ///
    /// fn pool(rocket: &rocket::Rocket) {
    ///     let config = Config::from("my_db", rocket).unwrap();
    ///     assert_eq!(config.url, "db/db.sqlite");
    ///     assert_eq!(config.pool_size, 25);
    ///
    ///     let config = Config::from("my_other_db", rocket).unwrap();
    ///     assert_eq!(config.url, "mysql://root:root@localhost/database");
    ///     assert_eq!(config.pool_size, (rocket.config().workers * 2) as u32);
    ///
    ///     let config = Config::from("unknown_db", rocket);
    ///     assert!(config.is_err())
    /// }
    /// #
    /// # let config = Figment::from(rocket::Config::default()).merge(toml);
    /// # let rocket = rocket::custom(config);
    /// # pool(&rocket);
    /// # }
    /// ```
    pub fn from(db_name: &str, rocket: &rocket::Rocket) -> Result<Config, figment::Error> {
        let db_key = format!("databases.{}", db_name);
        let key = |name: &str| format!("{}.{}", db_key, name);
        Figment::from(rocket.figment())
            .merge(Serialized::default(&key("pool_size"), rocket.config().workers * 2))
            .merge(Serialized::default(&key("timeout"), 5))
            .extract_inner::<Self>(&db_key)
    }
}

