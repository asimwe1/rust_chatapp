use rocket::fairing::AdHoc;
use rocket::local::blocking::{Client, LocalResponse, LocalRequest};
use rocket::http::{Status, ContentType};
use serde::{Serialize, Deserialize};

// Make it easier to work with JSON.
trait LocalResponseExt {
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Option<T>;
}

trait LocalRequestExt {
    fn json<T: serde::Serialize>(self, value: &T) -> Self;
}

impl LocalResponseExt for LocalResponse<'_> {
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Option<T> {
        serde_json::from_reader(self).ok()
    }
}

impl LocalRequestExt for LocalRequest<'_> {
    fn json<T: serde::Serialize>(self, value: &T) -> Self {
        let json = serde_json::to_string(value).expect("JSON serialization");
        self.header(ContentType::JSON).body(json)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Post {
    title: String,
    text: String,
}

fn test(base: &str, stage: AdHoc) {
    // Number of posts we're going to create/read/delete.
    const N: usize = 20;

    // NOTE: If we had more than one test running concurently that dispatches
    // DB-accessing requests, we'd need transactions or to serialize all tests.
    let client = Client::tracked(rocket::ignite().attach(stage)).unwrap();

    // Clear everything from the database.
    assert_eq!(client.delete(base).dispatch().status(), Status::Ok);
    assert_eq!(client.get(base).dispatch().into_json::<Vec<i64>>(), Some(vec![]));

    // Add some random posts, ensure they're listable and readable.
    for i in 1..=N{
        let title = format!("My Post - {}", i);
        let text = format!("Once upon a time, at {}'o clock...", i);
        let post = Post { title: title.clone(), text: text.clone() };

        // Create a new post.
        let response = client.post(base).json(&post).dispatch().into_json::<Post>();
        assert_eq!(response.unwrap(), post);

        // Ensure the index shows one more post.
        let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
        assert_eq!(list.len(), i);

        // The last in the index is the new one; ensure contents match.
        let last = list.last().unwrap();
        let response = client.get(format!("{}/{}", base, last)).dispatch();
        assert_eq!(response.into_json::<Post>().unwrap(), post);
    }

    // Now delete all of the posts.
    for _ in 1..=N {
        // Get a valid ID from the index.
        let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
        let id = list.get(0).expect("have post");

        // Delete that post.
        let response = client.delete(format!("{}/{}", base, id)).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    // Ensure they're all gone.
    let list = client.get(base).dispatch().into_json::<Vec<i64>>().unwrap();
    assert!(list.is_empty());

    // Trying to delete should now 404.
    let response = client.delete(format!("{}/{}", base, 1)).dispatch();
    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_sqlx() {
    test("/sqlx", crate::sqlx::stage())
}

#[test]
fn test_diesel() {
    test("/diesel", crate::diesel_sqlite::stage())
}

#[test]
fn test_rusqlite() {
    test("/rusqlite", crate::rusqlite::stage())
}
