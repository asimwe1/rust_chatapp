use std::process::ExitCode;
use std::time::Duration;

use rocket::listener::unix::UnixListener;
use rocket::tokio::net::TcpListener;
use rocket::yansi::Paint;
use rocket::{get, routes, Build, Rocket, State};
use reqwest::{tls::TlsInfo, Identity};
use testbench::*;

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

static TLS_CONFIG: &str = r#"
    [default.tls]
    certs = "{ROCKET}/examples/tls/private/rsa_sha256_cert.pem"
    key = "{ROCKET}/examples/tls/private/rsa_sha256_key.pem"
"#;

trait RocketExt {
    fn default() -> Self;
    fn tls_default() -> Self;
    fn configure_with_toml(self, toml: &str) -> Self;
}

impl RocketExt for Rocket<Build> {
    fn default() -> Self {
        rocket::build().configure_with_toml(DEFAULT_CONFIG)
    }

    fn tls_default() -> Self {
        rocket::build()
            .configure_with_toml(DEFAULT_CONFIG)
            .configure_with_toml(TLS_CONFIG)
    }

    fn configure_with_toml(self, toml: &str) -> Self {
        use rocket::figment::{Figment, providers::{Format, Toml}};

        let toml = toml.replace("{ROCKET}", rocket::fs::relative!("../"));
        let config = Figment::from(self.figment())
            .merge(Toml::string(&toml).nested());

        self.configure(config)
    }
}

fn read(path: &str) -> Result<Vec<u8>> {
    let path = path.replace("{ROCKET}", rocket::fs::relative!("../"));
    Ok(std::fs::read(path)?)
}

fn cert(path: &str) -> Result<Vec<u8>> {
    let mut data = std::io::Cursor::new(read(path)?);
    let cert = rustls_pemfile::certs(&mut data).last();
    Ok(cert.ok_or(Error::MissingCertificate)??.to_vec())
}

fn run_fail() -> Result<()> {
    use rocket::fairing::AdHoc;

    let server = spawn! {
        let fail = AdHoc::try_on_ignite("FailNow", |rocket| async { Err(rocket) });
        Rocket::default().attach(fail)
    };

    if let Err(Error::Liftoff(stdout, _)) = server {
        assert!(stdout.contains("Rocket failed to launch due to failing fairings"));
        assert!(stdout.contains("FailNow"));
    } else {
        panic!("unexpected result: {server:#?}");
    }

    Ok(())
}

fn infinite() -> Result<()> {
    use rocket::response::stream::TextStream;

    let mut server = spawn! {
        #[get("/")]
        fn infinite() -> TextStream![&'static str] {
            TextStream! {
                loop {
                    yield rocket::futures::future::pending::<&str>().await;
                }
            }
        }

        Rocket::default().mount("/", routes![infinite])
    }?;

    let client = Client::default();
    client.get(&server, "/")?.send()?;
    server.terminate()?;

    let stdout = server.read_stdout()?;
    assert!(stdout.contains("Rocket has launched on http"));
    assert!(stdout.contains("GET /"));
    assert!(stdout.contains("Graceful shutdown completed"));
    Ok(())
}

fn tls_info() -> Result<()> {
    let mut server = spawn! {
        #[get("/")]
        fn hello_world() -> &'static str {
            "Hello, world!"
        }

        Rocket::tls_default().mount("/", routes![hello_world])
    }?;

    let client = Client::default();
    let response = client.get(&server, "/")?.send()?;
    let tls = response.extensions().get::<TlsInfo>().unwrap();
    assert!(!tls.peer_certificate().unwrap().is_empty());
    assert_eq!(response.text()?, "Hello, world!");

    server.terminate()?;
    let stdout = server.read_stdout()?;
    assert!(stdout.contains("Rocket has launched on https"));
    assert!(stdout.contains("Graceful shutdown completed"));
    assert!(stdout.contains("GET /"));
    Ok(())
}

fn tls_resolver() -> Result<()> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use rocket::tls::{Resolver, TlsConfig, ClientHello, ServerConfig};

    struct CountingResolver {
        config: Arc<ServerConfig>,
        counter: Arc<AtomicUsize>,
    }

    #[rocket::async_trait]
    impl Resolver for CountingResolver {
        async fn init(rocket: &Rocket<Build>) -> rocket::tls::Result<Self> {
            let config: TlsConfig = rocket.figment().extract_inner("tls")?;
            let config = Arc::new(config.server_config().await?);
            let counter = rocket.state::<Arc<AtomicUsize>>().unwrap().clone();
            Ok(Self { config, counter })
        }

        async fn resolve(&self, _: ClientHello<'_>) -> Option<Arc<ServerConfig>> {
            self.counter.fetch_add(1, Ordering::Release);
            Some(self.config.clone())
        }
    }

    let server = spawn! {
        #[get("/count")]
        fn count(counter: &State<Arc<AtomicUsize>>) -> String {
            counter.load(Ordering::Acquire).to_string()
        }

        let counter = Arc::new(AtomicUsize::new(0));
        Rocket::tls_default()
            .manage(counter)
            .mount("/", routes![count])
            .attach(CountingResolver::fairing())
    }?;

    let client = Client::default();
    let response = client.get(&server, "/count")?.send()?;
    assert_eq!(response.text()?, "1");

    // Use a new client so we get a new TLS session.
    let client = Client::default();
    let response = client.get(&server, "/count")?.send()?;
    assert_eq!(response.text()?, "2");
    Ok(())
}

fn test_mtls(mandatory: bool) -> Result<()> {
    let server = spawn!(mandatory: bool => {
        let mtls_config = format!(r#"
            [default.tls.mutual]
            ca_certs = "{{ROCKET}}/examples/tls/private/ca_cert.pem"
            mandatory = {mandatory}
        "#);

        #[get("/")]
        fn hello(cert: rocket::mtls::Certificate<'_>) -> String {
            format!("{}:{}[{}] {}", cert.serial(), cert.version(), cert.issuer(), cert.subject())
        }

        #[get("/", rank = 2)]
        fn hi() -> &'static str {
            "Hello!"
        }

        Rocket::tls_default()
            .configure_with_toml(&mtls_config)
            .mount("/", routes![hello, hi])
    })?;

    let pem = read("{ROCKET}/examples/tls/private/client.pem")?;
    let client: Client = Client::build()
        .identity(Identity::from_pem(&pem)?)
        .try_into()?;

    let response = client.get(&server, "/")?.send()?;
    assert_eq!(response.text()?,
        "611895682361338926795452113263857440769284805738:2\
            [C=US, ST=CA, O=Rocket CA, CN=Rocket Root CA] \
            C=US, ST=California, L=Silicon Valley, O=Rocket, \
            CN=Rocket TLS Example, Email=example@rocket.local");

    let client = Client::default();
    let response = client.get(&server, "/")?.send();
    if mandatory {
        assert!(response.unwrap_err().is_request());
    } else {
        assert_eq!(response?.text()?, "Hello!");
    }

    Ok(())
}

fn tls_mtls() -> Result<()> {
    test_mtls(false)?;
    test_mtls(true)
}

fn sni_resolver() -> Result<()> {
    use std::sync::Arc;
    use std::collections::HashMap;

    use rocket::http::uri::Host;
    use rocket::tls::{Resolver, TlsConfig, ClientHello, ServerConfig};

    struct SniResolver {
        default: Arc<ServerConfig>,
        map: HashMap<Host<'static>, Arc<ServerConfig>>
    }

    #[rocket::async_trait]
    impl Resolver for SniResolver {
        async fn init(rocket: &Rocket<Build>) -> rocket::tls::Result<Self> {
            let default: TlsConfig = rocket.figment().extract_inner("tls")?;
            let sni: HashMap<Host<'_>, TlsConfig> = rocket.figment().extract_inner("tls.sni")?;

            let default = Arc::new(default.server_config().await?);
            let mut map = HashMap::new();
            for (host, config) in sni {
                let config = config.server_config().await?;
                map.insert(host, Arc::new(config));
            }

            Ok(SniResolver { default, map })
        }

        async fn resolve(&self, hello: ClientHello<'_>) -> Option<Arc<ServerConfig>> {
            if let Some(Ok(host)) = hello.server_name().map(Host::parse) {
                if let Some(config) = self.map.get(&host) {
                    return Some(config.clone());
                }
            }

            Some(self.default.clone())
        }
    }

    static SNI_TLS_CONFIG: &str = r#"
        [default.tls]
        certs = "{ROCKET}/examples/tls/private/rsa_sha256_cert.pem"
        key = "{ROCKET}/examples/tls/private/rsa_sha256_key.pem"

        [default.tls.sni."sni1.dev"]
        certs = "{ROCKET}/examples/tls/private/ecdsa_nistp256_sha256_cert.pem"
        key = "{ROCKET}/examples/tls/private/ecdsa_nistp256_sha256_key_pkcs8.pem"

        [default.tls.sni."sni2.dev"]
        certs = "{ROCKET}/examples/tls/private/ed25519_cert.pem"
        key = "{ROCKET}/examples/tls/private/ed25519_key.pem"
    "#;

    let server = spawn! {
        #[get("/")] fn index() { }

        Rocket::default()
            .configure_with_toml(SNI_TLS_CONFIG)
            .mount("/", routes![index])
            .attach(SniResolver::fairing())
    }?;

    let client: Client = Client::build()
        .resolve("unknown.dev", server.socket_addr())
        .resolve("sni1.dev", server.socket_addr())
        .resolve("sni2.dev", server.socket_addr())
        .try_into()?;

    let response = client.get(&server, "https://unknown.dev")?.send()?;
    let tls = response.extensions().get::<TlsInfo>().unwrap();
    let expected = cert("{ROCKET}/examples/tls/private/rsa_sha256_cert.pem")?;
    assert_eq!(tls.peer_certificate().unwrap(), expected);

    let response = client.get(&server, "https://sni1.dev")?.send()?;
    let tls = response.extensions().get::<TlsInfo>().unwrap();
    let expected = cert("{ROCKET}/examples/tls/private/ecdsa_nistp256_sha256_cert.pem")?;
    assert_eq!(tls.peer_certificate().unwrap(), expected);

    let response = client.get(&server, "https://sni2.dev")?.send()?;
    let tls = response.extensions().get::<TlsInfo>().unwrap();
    let expected = cert("{ROCKET}/examples/tls/private/ed25519_cert.pem")?;
    assert_eq!(tls.peer_certificate().unwrap(), expected);
    Ok(())
}

fn tcp_unix_listener_fail() -> Result<()> {
    let server = spawn! {
        Rocket::default().configure_with_toml("[default]\naddress = 123")
    };

    if let Err(Error::Liftoff(stdout, _)) = server {
        assert!(stdout.contains("expected valid TCP (ip) or unix (path)"));
        assert!(stdout.contains("default.address"));
    } else {
        panic!("unexpected result: {server:#?}");
    }

    let server = Server::spawn((), |(token, _)| {
        let rocket = Rocket::default().configure_with_toml("[default]\naddress = \"unix:foo\"");
        token.launch_with::<TcpListener>(rocket)
    });

    if let Err(Error::Liftoff(stdout, _)) = server {
        assert!(stdout.contains("invalid tcp endpoint: unix:foo"));
    } else {
        panic!("unexpected result: {server:#?}");
    }

    let server = Server::spawn((), |(token, _)| {
        token.launch_with::<UnixListener>(Rocket::default())
    });

    if let Err(Error::Liftoff(stdout, _)) = server {
        assert!(stdout.contains("invalid unix endpoint: tcp:127.0.0.1:8000"));
    } else {
        panic!("unexpected result: {server:#?}");
    }

    Ok(())
}

macro_rules! tests {
    ($($f:ident),* $(,)?) => {[
        $(Test {
            name: stringify!($f),
            run: |_: ()| $f().map_err(|e| e.to_string()),
        }),*
    ]};
}

#[derive(Copy, Clone)]
struct Test {
    name: &'static str,
    run: fn(()) -> Result<(), String>,
}

static TESTS: &[Test] = &tests![
    run_fail, infinite, tls_info, tls_resolver, tls_mtls, sni_resolver,
    tcp_unix_listener_fail
];

fn main() -> ExitCode {
    procspawn::init();

    let filter = std::env::args().nth(1).unwrap_or_default();
    let filtered = TESTS.into_iter().filter(|test| test.name.contains(&filter));

    println!("running {}/{} tests", filtered.clone().count(), TESTS.len());
    let handles = filtered.map(|test| (test, std::thread::spawn(|| {
        let name = test.name;
        let start = std::time::SystemTime::now();
        let mut proc = procspawn::spawn((), test.run);
        let result = loop {
            match proc.join_timeout(Duration::from_secs(10)) {
                Err(e) if e.is_timeout() => {
                    let elapsed = start.elapsed().unwrap().as_secs();
                    println!("{name} has been running for {elapsed} seconds...");

                    if elapsed >= 30 {
                        println!("{name} timeout");
                        break Err(e);
                    }
                },
                result => break result,
            }
        };

        match result.as_ref().map_err(|e| e.panic_info()) {
            Ok(Ok(_)) => println!("test {name} ... {}", "ok".green()),
            Ok(Err(e)) => println!("test {name} ... {}\n  {e}", "fail".red()),
            Err(Some(_)) => println!("test {name} ... {}", "panic".red().underline()),
            Err(None) => println!("test {name} ... {}", "error".magenta()),
        }

        matches!(result, Ok(Ok(())))
    })));

    let mut success = true;
    for (_, handle) in handles {
        success &= handle.join().unwrap_or(false);
    }

    match success {
        true => ExitCode::SUCCESS,
        false => {
            println!("note: use `NOCAPTURE=1` to see test output");
            ExitCode::FAILURE
        }
    }
}

// TODO: Implement an `UpdatingResolver`. Expose `SniResolver` and
// `UpdatingResolver` in a `contrib` library or as part of `rocket`.
//
// struct UpdatingResolver {
//     timestamp: AtomicU64,
//     config: ArcSwap<ServerConfig>
// }
//
// #[crate::async_trait]
// impl Resolver for UpdatingResolver {
//     async fn resolve(&self, _: ClientHello<'_>) -> Option<Arc<ServerConfig>> {
//         if let Either::Left(path) = self.tls_config.certs() {
//             let metadata = tokio::fs::metadata(&path).await.ok()?;
//             let modtime = metadata.modified().ok()?;
//             let timestamp = modtime.duration_since(UNIX_EPOCH).ok()?.as_secs();
//             let old_timestamp = self.timestamp.load(Ordering::Acquire);
//             if timestamp > old_timestamp {
//                 let new_config = self.tls_config.to_server_config().await.ok()?;
//                 self.server_config.store(Arc::new(new_config));
//                 self.timestamp.store(timestamp, Ordering::Release);
//             }
//         }
//
//         Some(self.server_config.load_full())
//     }
// }
