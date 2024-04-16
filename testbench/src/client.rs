use std::time::Duration;

use reqwest::blocking::{ClientBuilder, RequestBuilder};
use rocket::http::{ext::IntoOwned, uri::{Absolute, Uri}};

use crate::{Result, Error, Server};

#[derive(Debug)]
pub struct Client {
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn default() -> Client {
        Client::build()
            .try_into()
            .expect("default builder ok")
    }

    pub fn build() -> ClientBuilder {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .cookie_store(true)
            .tls_info(true)
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
    }

    pub fn get(&self, server: &Server, url: &str) -> Result<RequestBuilder> {
        let uri = match Uri::parse_any(url).map_err(|e| e.into_owned())? {
            Uri::Origin(uri) => {
                let proto = if server.tls { "https" } else { "http" };
                let uri = format!("{proto}://127.0.0.1:{}{uri}", server.port);
                Absolute::parse_owned(uri)?
            }
            Uri::Absolute(mut uri) => {
                if let Some(auth) = uri.authority() {
                    let mut auth = auth.clone();
                    auth.set_port(server.port);
                    uri.set_authority(auth);
                }

                uri
            }
            uri => return Err(Error::InvalidUri(uri.into_owned())),
        };

        Ok(self.client.get(uri.to_string()))
    }
}

impl From<reqwest::blocking::Client> for Client {
    fn from(client: reqwest::blocking::Client) -> Self {
        Client { client }
    }
}

impl TryFrom<ClientBuilder> for Client {
    type Error = Error;

    fn try_from(builder: ClientBuilder) -> Result<Self, Self::Error> {
        Ok(Client { client: builder.build()? })
    }
}
