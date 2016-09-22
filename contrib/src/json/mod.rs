extern crate serde;
extern crate serde_json;

use std::ops::{Deref, DerefMut};

use rocket::request::{Request, FromRequest};
use rocket::response::{Responder, Outcome, FreshHyperResponse};
use rocket::response::data;

use self::serde::{Serialize, Deserialize};
use self::serde_json::Error as JSONError;

/// The JSON datatype, which implements both `FromRequest` and `Responder`. This
/// type allows you to trivially consume and respond with JSON in your Rocket
/// application.
///
/// If you're receiving JSON data, simple add a `JSON<T>` type to your function
/// signature where `T` is some type you'd like to parse from JSON. `T` must
/// implement `Deserialize` from [Serde](https://github.com/serde-rs/json). The
/// data is parsed from the HTTP request body.
///
/// ```rust,ignore
/// #[post("/users/", format = "application/json")]
/// fn new_user(user: JSON<User>) {
///     ...
/// }
/// ```
/// You don't _need_ to use `format = "application/json"`, but it _may_ be what
/// you want. Using `format = application/json` means that any request that
/// doesn't specify "application/json" as its first `Accept:` header parameter
/// will not be routed to this handler.
///
/// If you're responding with JSON data, return a `JSON<T>` type, where `T`
/// implements implements `Serialize` from
/// [Serde](https://github.com/serde-rs/json). The content type is set to
/// `application/json` automatically.
///
/// ```rust,ignore
/// #[get("/users/<id>")]
/// fn user(id: usize) -> JSON<User> {
///     let user_from_id = User::from(id);
///     ...
///     JSON(user_from_id)
/// }
/// ```
///
#[derive(Debug)]
pub struct JSON<T>(pub T);

impl<T> JSON<T> {
    /// Consumes the JSON wrapper and returns the wrapped item.
    ///
    /// # Example
    /// ```rust
    /// # use rocket_contrib::JSON;
    /// let string = "Hello".to_string();
    /// let my_json = JSON(string);
    /// assert_eq!(my_json.unwrap(), "Hello".to_string());
    /// ```
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<'r, 'c, T: Deserialize> FromRequest<'r, 'c> for JSON<T> {
    type Error = JSONError;
    fn from_request(request: &'r Request<'c>) -> Result<Self, Self::Error> {
        Ok(JSON(serde_json::from_slice(request.data.as_slice())?))
    }
}

impl<T: Serialize> Responder for JSON<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match serde_json::to_string(&self.0) {
            Ok(json_string) => data::JSON(json_string).respond(res),
            Err(e) => {
                error_!("JSON failed to serialize: {:?}", e);
                Outcome::FailStop
            }
        }
    }
}

impl<T> Deref for JSON<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T> DerefMut for JSON<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.0
    }
}

/// A nice little macro to create simple HashMaps. Really convenient for
/// returning ad-hoc JSON messages.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate rocket_contrib;
/// use std::collections::HashMap;
/// # fn main() {
/// let map: HashMap<&str, usize> = map! {
///     "status" => 0,
///     "count" => 100
/// };
///
/// assert_eq!(map.len(), 2);
/// assert_eq!(map.get("status"), Some(&0));
/// assert_eq!(map.get("count"), Some(&100));
/// # }
/// ```
#[macro_export]
macro_rules! map {
    ($($key:expr => $value:expr),+) => ({
        use std::collections::HashMap;
        let mut map = HashMap::new();
        $(map.insert($key, $value);)+
        map
    });

    ($($key:expr => $value:expr),+,) => {
        map!($($key => $value),+)
    };
}
