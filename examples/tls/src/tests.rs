use rocket::local::blocking::Client;

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
        assert_eq!(response.into_string(), Some("Hello, world!".into()));
    }
}
