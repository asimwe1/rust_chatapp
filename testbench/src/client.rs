use std::time::Duration;
use std::sync::Once;
use std::process::Stdio;
use std::io::{self, Read};

use rocket::fairing::AdHoc;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::{self, Absolute, Uri};
use rocket::serde::{Deserialize, Serialize};
use rocket::{Build, Rocket};

use procspawn::SpawnError;
use thiserror::Error;
use ipc_channel::ipc::{IpcOneShotServer, IpcReceiver, IpcSender};

static DEFAULT_CONFIG: &str = r#"
    [default]
    address = "tcp:127.0.0.1"
    workers = 2
    port = 0
    cli_colors = false
    secret_key = "itlYmFR2vYKrOmFhupMIn/hyB6lYCCTXz4yaQX89XVg="

    [default.shutdown]
    grace = 1
    mercy = 1
"#;

#[derive(Debug)]
#[allow(unused)]
pub struct Client {
    client: reqwest::blocking::Client,
    server: procspawn::JoinHandle<()>,
    tls: bool,
    port: u16,
    rx: IpcReceiver<Message>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("join/kill failed: {0}")]
    JoinError(#[from] SpawnError),
    #[error("kill failed: {0}")]
    TermFailure(#[from] nix::errno::Errno),
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid URI: {0}")]
    Uri(#[from] uri::Error<'static>),
    #[error("the URI is invalid")]
    InvalidUri,
    #[error("bad request: {0}")]
    Request(#[from] reqwest::Error),
    #[error("IPC failure: {0}")]
    Ipc(#[from] ipc_channel::ipc::IpcError),
    #[error("liftoff failed")]
    Liftoff(String, String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum Message {
    Liftoff(bool, u16),
    Failure,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
#[must_use]
pub struct Token(String);

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Token {
    fn configure(&self, toml: &str, rocket: Rocket<Build>) -> Rocket<Build> {
        use rocket::figment::{Figment, providers::{Format, Toml}};

        let toml = toml.replace("{CRATE}", env!("CARGO_MANIFEST_DIR"));
        let config = Figment::from(rocket.figment())
            .merge(Toml::string(DEFAULT_CONFIG).nested())
            .merge(Toml::string(&toml).nested());

        let server = self.0.clone();
        rocket.configure(config)
            .attach(AdHoc::on_liftoff("Liftoff", move |rocket| Box::pin(async move {
                let tcp = rocket.endpoints().find_map(|e| e.tcp()).unwrap();
                let tls = rocket.endpoints().any(|e| e.is_tls());
                let sender = IpcSender::<Message>::connect(server).unwrap();
                let _ = sender.send(Message::Liftoff(tls, tcp.port()));
                let _ = sender.send(Message::Liftoff(tls, tcp.port()));
            })))
    }

    pub fn rocket(&self, toml: &str) -> Rocket<Build> {
        self.configure(toml, rocket::build())
    }

    pub fn configured_launch(self, toml: &str, rocket: Rocket<Build>) {
        let rocket = self.configure(toml, rocket);
        if let Err(e) = rocket::execute(rocket.launch()) {
            let sender = IpcSender::<Message>::connect(self.0).unwrap();
            let _ = sender.send(Message::Failure);
            let _ = sender.send(Message::Failure);
            e.pretty_print();
            std::process::exit(1);
        }
    }

    pub fn launch(self, rocket: Rocket<Build>) {
        self.configured_launch(DEFAULT_CONFIG, rocket)
    }
}
pub fn start(f: fn(Token)) -> Result<Client> {
    static INIT: Once = Once::new();
    INIT.call_once(procspawn::init);

    let (ipc, server) = IpcOneShotServer::new()?;
    let mut server = procspawn::Builder::new()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn(Token(server), f);

    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .tls_info(true)
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let (rx, _) = ipc.accept().unwrap();
    match rx.recv() {
        Ok(Message::Liftoff(tls, port)) => Ok(Client { client, server, tls, port, rx }),
        Ok(Message::Failure) => {
            let stdout = server.stdout().unwrap();
            let mut out = String::new();
            stdout.read_to_string(&mut out)?;

            let stderr = server.stderr().unwrap();
            let mut err = String::new();
            stderr.read_to_string(&mut err)?;
            Err(Error::Liftoff(out, err))
        }
        Err(e) => Err(e.into()),
    }

}

pub fn default() -> Result<Client> {
    start(|token| token.launch(rocket::build()))
}

impl Client {
    pub fn read_stdout(&mut self) -> Result<String> {
        let Some(stdout) = self.server.stdout() else {
            return Ok(String::new());
        };

        let mut string = String::new();
        stdout.read_to_string(&mut string)?;
        Ok(string)
    }

    pub fn read_stderr(&mut self) -> Result<String> {
        let Some(stderr) = self.server.stderr() else {
            return Ok(String::new());
        };

        let mut string = String::new();
        stderr.read_to_string(&mut string)?;
        Ok(string)
    }

    pub fn kill(&mut self) -> Result<()> {
        Ok(self.server.kill()?)
    }

    pub fn terminate(&mut self) -> Result<()> {
        use nix::{sys::signal, unistd::Pid};

        let pid = Pid::from_raw(self.server.pid().unwrap() as i32);
        Ok(signal::kill(pid, signal::SIGTERM)?)
    }

    pub fn wait(&mut self) -> Result<()> {
        match self.server.join_timeout(Duration::from_secs(5)) {
            Ok(_) => Ok(()),
            Err(e) if e.is_remote_close() => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get(&self, url: &str) -> Result<reqwest::blocking::RequestBuilder> {
        let uri = match Uri::parse_any(url).map_err(|e| e.into_owned())? {
            Uri::Origin(uri) => {
                let proto = if self.tls { "https" } else { "http" };
                let uri = format!("{proto}://127.0.0.1:{}{uri}", self.port);
                Absolute::parse_owned(uri)?
            }
            Uri::Absolute(uri) => uri,
            _ => return Err(Error::InvalidUri),
        };

        Ok(self.client.get(uri.to_string()))
    }
}
