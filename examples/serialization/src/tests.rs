use rocket::local::blocking::Client;
use rocket::http::{Status, ContentType, Accept};

#[test]
fn json_bad_get_put() {
    let client = Client::tracked(super::rocket()).unwrap();

    // Try to get a message with an ID that doesn't exist.
    let res = client.get("/json/99").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), Status::NotFound);

    let body = res.into_string().unwrap();
    assert!(body.contains("error"));
    assert!(body.contains("Resource was not found."));

    // Try to get a message with an invalid ID.
    let res = client.get("/json/hi").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), Status::NotFound);
    assert!(res.into_string().unwrap().contains("error"));

    // Try to put a message without a proper body.
    let res = client.put("/json/80").header(ContentType::JSON).dispatch();
    assert_eq!(res.status(), Status::BadRequest);

    // Try to put a message with a semantically invalid body.
    let res = client.put("/json/0")
        .header(ContentType::JSON)
        .body(r#"{ "dogs?": "love'em!" }"#)
        .dispatch();

    assert_eq!(res.status(), Status::UnprocessableEntity);

    // Try to put a message for an ID that doesn't exist.
    let res = client.put("/json/80")
        .header(ContentType::JSON)
        .body(r#"{ "message": "Bye bye, world!" }"#)
        .dispatch();

    assert_eq!(res.status(), Status::NotFound);
}

#[test]
fn json_post_get_put_get() {
    let client = Client::tracked(super::rocket()).unwrap();

    // Create/read/update/read a few messages.
    for id in 0..10 {
        let uri = format!("/json/{}", id);
        let message = format!("Hello, JSON {}!", id);

        // Check that a message with doesn't exist.
        let res = client.get(&uri).header(ContentType::JSON).dispatch();
        assert_eq!(res.status(), Status::NotFound);

        // Add a new message. This should be ID 0.
        let res = client.post("/json")
            .header(ContentType::JSON)
            .body(format!(r#"{{ "message": "{}" }}"#, message))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        // Check that the message exists with the correct contents.
        let res = client.get(&uri).header(Accept::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_string().unwrap();
        assert!(body.contains(&message));

        // Change the message contents.
        let res = client.put(&uri)
            .header(ContentType::JSON)
            .body(r#"{ "message": "Bye bye, world!" }"#)
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        // Check that the message exists with the updated contents.
        let res = client.get(&uri).header(Accept::JSON).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body = res.into_string().unwrap();
        assert!(!body.contains(&message));
        assert!(body.contains("Bye bye, world!"));
    }
}

#[test]
fn msgpack_get() {
    let client = Client::tracked(super::rocket()).unwrap();
    let res = client.get("/msgpack/1").header(ContentType::MsgPack).dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::MsgPack));

    // Check that the message is `[1, "Hello, world!"]`
    assert_eq!(&res.into_bytes().unwrap(), &[146, 1, 173, 72, 101, 108, 108,
        111, 44, 32, 119, 111, 114, 108, 100, 33]);
}

#[test]
fn msgpack_post() {
    // Dispatch request with a message of `[2, "Goodbye, world!"]`.
    let client = Client::tracked(super::rocket()).unwrap();
    let res = client.post("/msgpack")
        .header(ContentType::MsgPack)
        .body(&[146, 2, 175, 71, 111, 111, 100, 98, 121, 101, 44, 32, 119, 111,
            114, 108, 100, 33])
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.into_string(), Some("Goodbye, world!".into()));
}
