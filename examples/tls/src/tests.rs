use std::fs::{self, File};

use rocket::local::blocking::Client;
use rocket::fs::relative;

#[test]
fn hello_mutual() {
    let client = Client::tracked(super::rocket()).unwrap();
    let cert_paths = fs::read_dir(relative!("private")).unwrap()
        .map(|entry| entry.unwrap().path().to_string_lossy().into_owned())
        .filter(|path| path.ends_with("_cert.pem") && !path.ends_with("ca_cert.pem"));

    for path in cert_paths {
        let response = client.get("/")
            .identity(File::open(&path).unwrap())
            .dispatch();

        let response = response.into_string().unwrap();
        let subject = response.split(']').nth(1).unwrap().trim();
        assert_eq!(subject, "C=US, ST=CA, O=Rocket, CN=localhost");
    }
}

#[test]
fn hello_world() {
    let profiles = [
        "rsa_sha256",
        "ecdsa_nistp256_sha256_pkcs8",
        "ecdsa_nistp384_sha384_pkcs8",
        "ecdsa_nistp256_sha256_sec1",
        "ecdsa_nistp384_sha384_sec1",
        "ed25519",
    ];

    // TODO: Testing doesn't actually read keys since we don't do TLS locally.
    for profile in profiles {
        let config = rocket::Config::figment().select(profile);
        let client = Client::tracked(super::rocket().configure(config)).unwrap();
        let response = client.get("/").dispatch();
        assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }
}
