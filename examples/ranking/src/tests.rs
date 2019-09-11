use rocket::local::Client;

async fn test(uri: String, expected: String) {
    let rocket = rocket::ignite().mount("/", routes![super::hello, super::hi]);
    let client = Client::new(rocket).unwrap();
    let mut response = client.get(uri).dispatch().await;
    assert_eq!(response.body_string().await, Some(expected));
}

#[rocket::async_test]
async fn test_hello() {
    for &(name, age) in &[("Mike", 22), ("Michael", 80), ("A", 0), ("a", 127)] {
        let uri = format!("/hello/{}/{}", name, age);
        let expected = format!("Hello, {} year old named {}!", age, name);
        test(uri, expected).await;
    }
}

#[rocket::async_test]
async fn test_failing_hello_hi() {
    // Invalid integers.
    for &(name, age) in &[("Mike", 1000), ("Michael", 128), ("A", -800), ("a", -200)] {
        let uri = format!("/hello/{}/{}", name, age);
        let expected = format!("Hi {}! Your age ({}) is kind of funky.", name, age);
        test(uri, expected).await;
    }

    // Non-integers.
    for &(name, age) in &[("Mike", "!"), ("Michael", "hi"), ("A", "blah"), ("a", "0-1")] {
        let uri = format!("/hello/{}/{}", name, age);
        let expected = format!("Hi {}! Your age ({}) is kind of funky.", name, age);
        test(uri, expected).await;
    }
}
