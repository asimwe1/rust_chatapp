use rocket::local::blocking::Client;

#[test]
fn hello_world() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/").dispatch();
    assert_eq!(response.into_string(), Some("Hello, world!".into()));

    let response = client.get("/hello/world").dispatch();
    assert_eq!(response.into_string(), Some("Hello, world!".into()));
}

#[test]
fn hello_mir() {
    let client = Client::tracked(super::rocket()).unwrap();
    let response = client.get("/hello/%D0%BC%D0%B8%D1%80").dispatch();
    assert_eq!(response.into_string(), Some("Привет, мир!".into()));
}
