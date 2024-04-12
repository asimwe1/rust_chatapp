use rocket::{fairing::AdHoc, *};
use rocket_testbench::client::{self, Error};
use reqwest::tls::TlsInfo;

fn run() -> client::Result<()> {
    let mut client = client::start(|token| {
        #[get("/")]
        fn index() -> &'static str {
            "Hello, world!"
        }

        token.configured_launch(r#"
            [default.tls]
            certs = "{CRATE}/../examples/tls/private/rsa_sha256_cert.pem"
            key = "{CRATE}/../examples/tls/private/rsa_sha256_key.pem"
        "#, rocket::build().mount("/", routes![index]));
    })?;

    let response = client.get("/")?.send()?;
    let tls = response.extensions().get::<TlsInfo>().unwrap();
    assert!(!tls.peer_certificate().unwrap().is_empty());
    assert_eq!(response.text()?, "Hello, world!");

    client.terminate()?;
    let stdout = client.read_stdout()?;
    assert!(stdout.contains("Rocket has launched on https"));
    assert!(stdout.contains("Graceful shutdown completed"));
    assert!(stdout.contains("GET /"));
    Ok(())
}

fn run_fail() -> client::Result<()> {
    let client = client::start(|token| {
        let fail = AdHoc::try_on_ignite("FailNow", |rocket| async { Err(rocket) });
        token.launch(rocket::build().attach(fail));
    });

    if let Err(Error::Liftoff(stdout, _)) = client {
        assert!(stdout.contains("Rocket failed to launch due to failing fairings"));
        assert!(stdout.contains("FailNow"));
    } else {
        panic!("unexpected result: {client:#?}");
    }

    Ok(())
}

fn infinite() -> client::Result<()> {
    use rocket::response::stream::TextStream;

    let mut client = client::start(|token| {
        #[get("/")]
        fn infinite() -> TextStream![&'static str] {
            TextStream! {
                loop {
                    yield rocket::futures::future::pending::<&str>().await;
                }
            }
        }

        token.launch(rocket::build().mount("/", routes![infinite]));
    })?;

    client.get("/")?.send()?;
    client.terminate()?;
    let stdout = client.read_stdout()?;
    assert!(stdout.contains("Rocket has launched on http"));
    assert!(stdout.contains("GET /"));
    assert!(stdout.contains("Graceful shutdown completed"));
    Ok(())
}

fn main() {
    let names = ["run", "run_fail", "infinite"];
    let tests = [run, run_fail, infinite];
    let handles = tests.into_iter()
        .map(|test| std::thread::spawn(test))
        .collect::<Vec<_>>();

    let mut failure = false;
    for (handle, name) in handles.into_iter().zip(names) {
        let result = handle.join();
        failure = failure || matches!(result, Ok(Err(_)) | Err(_));
        match result {
            Ok(Ok(_)) => continue,
            Ok(Err(e)) => eprintln!("{name} failed: {e}"),
            Err(_) => eprintln!("{name} failed (see panic above)"),
        }
    }

    if failure {
        std::process::exit(1);
    }
}
