# Requests

Together, a route's attribute and function signature specify what must be true
about a request in order for the route's handler to be called. You've already
seen an example of this in action:

```rust
#[get("/world")]
fn handler() { .. }
```

This route indicates that it only matches against `GET` requests to the `/world`
route. Rocket ensures that this is the case before `handler` is called. Of
course, you can do much more than specify the method and path of a request.
Among other things, you can ask Rocket to automatically validate:

  * The type of a dynamic path segment.
  * The type of _many_ dynamic path segments.
  * The type of incoming data.
  * The types of query strings, forms, and form values.
  * The expected incoming or outgoing format of a request.
  * Any arbitrary, user-defined security or validation policies.

The route attribute and function signature work in tandem to describe these
validations. Rocket's code generation takes care of actually validating the
proprerties. The remainder of this section describes how to ask Rocket to
validate against all of these properties and more.

## Methods

A Rocket route attribute can be any one of `get`, `put`, `post`, `delete`,
`head`, `patch`, or `options`, each corresponding to the HTTP method to match
against. For example, the following attribute will match against `POST` requests
to the root path:

```rust
#[post("/")]
```

The grammar for these attributes is defined formally in the
[rocket_codegen](https://api.rocket.rs/rocket_codegen/) API docs.

Rocket handles `HEAD` requests automatically when there exists a `GET` route
that would otherwise match. It does this by stripping the body from the
response, if there is one. You can also specialize the handling of a `HEAD`
request by declaring a route for it; Rocket won't interfere with `HEAD` requests
your application handles.

### Reinterpreting Methods

Because browsers only send `GET` and `POST` requests, Rocket _reinterprets_
requests under certain conditions. If a `POST` request contains a body of
`Content-Type: application/x-www-form-urlencoded`, and the form's **first**
field has the name `_method` and a valid HTTP method name as its value (such as
`"PUT"`), that field's value is used as the method for the incoming request.
This allows Rocket applications to submit non-`POST` forms. The [todo
example](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/todo/static/index.html.tera#L47)
makes use of this feature to submit `PUT` and `DELETE` requests from a web form.

## Dynamic Segments

You can declare path segments as dynamic by using angle brackets around variable
names in a route's path. For example, if we wanted to say _Hello!_ to anything,
not just the world, we can declare a route like so:

```rust
#[get("/hello/<name>")]
fn hello(name: &RawStr) -> String {
    format!("Hello, {}!", name.as_str())
}
```

If we were to mount the path at the root (`.mount("/", routes![hello])`), then
any request to a path with two non-empty segments, where the first segment is
`hello`, will be dispatched to the `hello` route. For example, if we were to
visit `/hello/John`, the application would respond with `Hello, John!`.

Any number of dynamic path segments are allowed. A path segment can be of any
type, including your own, as long as the type implements the [`FromParam`].
Rocket implements `FromParam` for many of the standard library types, as well as
a few special Rocket types. For the full list of supplied implementations, see
the [`FromParam` API docs]. Here's a more complete route to illustrate varied
usage:

```rust
#[get("/hello/<name>/<age>/<cool>")]
fn hello(name: String, age: u8, cool: bool) -> String {
    if cool {
        format!("You're a cool {} year old, {}!", age, name)
    } else {
        format!("{}, we need to talk about your coolness.", name)
    }
}
```

[`FromParam`]: https://api.rocket.rs/rocket/request/trait.FromParam.html
[`FromParam` API docs]: https://api.rocket.rs/rocket/request/trait.FromParam.html

### Raw Strings

You may have noticed an unfamiliar [`RawStr`] type in the code example above.
This is a special type, provided by Rocket, that represents an unsanitzed,
unvalidated, and undecoded raw string from an HTTP message. It exists to
separate validated string inputs, represented by types such as `String`, `&str`,
and `Cow<str>` types, from unvalidated inputs, represented by `&RawStr`. It
provides helpful methods to convert the unvalidated string into a validated one.

Because `&RawStr` implements [`FromParam`], it can be used as the type of a
dynamic segment, as in the example above. When used as the type of a dynamic
segment, a `RawStr` points to a potentially undecoded string. By constrast, a
`String` is guaranteed to be decoded. Which you should use depends on whether
you want direct but potentially unsafe access to the string (`&RawStr`), or safe
access to the string at the cost of an allocation (`String`).

[`RawStr`]: https://api.rocket.rs/rocket/http/struct.RawStr.html

## Forwarding

Let's take a closer look at the route attribute and signature pair from the last
example:

```rust
#[get("/hello/<name>/<age>/<cool>")]
fn hello(name: String, age: u8, cool: bool) -> String { ... }
```

What if `cool` isn't a `bool`? Or, what if `age` isn't a `u8`? In this case,
Rocket _forwards_ the request to the next matching route, if there is any. This
continues until a route doesn't forward the request or there are no remaining
routes to try. When there are no remaining routes, a customizable **404 error**
is returned.

Routes are attempted in increasing _rank_ order. Rocket chooses a default
ranking from -4 to -1, detailed in the next section, for all routes, but a
route's rank can also be manually set with the `rank` attribute. To illustrate,
consider the following routes:

```rust
#[get("/user/<id>")]
fn user(id: usize) -> T { ... }

#[get("/user/<id>", rank = 2)]
fn user_int(id: isize) -> T { ... }

#[get("/user/<id>", rank = 3)]
fn user_str(id: &RawStr) -> T { ... }
```

Notice the `rank` parameters in `user_int` and `user_str`. If we run this
application with the routes mounted at the root, requests to `/user/<id>` will
be routed as follows:

  1. The `user` route matches first. If the string at the `<id>` position is an
     unsigned integer, then the `user` handler is called. If it is not, then the
     request is forwarded to the next matching route: `user_int`.

  2. The `user_int` route matches next. If `<id>` is a signed integer,
     `user_int` is called. Otherwise, the request is forwarded.

  3. The `user_str` route matches last. Since `<id>` is a always string, the
     route always matches. The `user_str` handler is called.

Forwards can be _caught_ by using a `Result` or `Option` type. For example, if
the type of `id` in the `user` function was `Result<usize, &RawStr>`, then `user`
would never forward. An `Ok` variant would indicate that `<id>` was a valid
`usize`, while an `Err` would indicate that `<id>` was not a `usize`. The
`Err`'s value would contain the string that failed to parse as a `usize`.

By the way, if you were to omit the `rank` parameter in the `user_str` or
`user_int` routes, Rocket would emit a warning indicating that the routes
_collide_, or can match against similar incoming requests. The `rank` parameter
resolves this collision.

### Default Ranking

If a rank is not explicitly specified, Rocket assigns a default ranking. By
default, routes with static paths and query strings have lower ranks (higher
precedence) while routes with dynamic paths and without query strings have
higher ranks (lower precedence). The table below describes the default ranking
of a route given its properties.

| static path   | query string   | rank   | example             |
| ------------- | -------------- | ------ | ------------------- |
| yes           | yes            | -4     | /hello?world=true   |
| yes           | no             | -3     | /hello              |
| no            | yes            | -2     | /&lt;hi>?world=true |
| no            | no             | -1     | /&lt;hi>            |

## Many Dynamic Segments

You can also match against multiple segments by using `<param..>` in the route
path. The type of such parameters, known as _segments_ parameters, must
implement [`FromSegments`]. Segments parameters must be the final component of a
path: any text after a segments parameter will result in a compile-time error.

As an example, the following route matches against all paths that begin with
`/page/`:

```rust
#[get("/page/<path..>")]
fn get_page(path: PathBuf) -> T { ... }
```

The path after `/page/` will be available in the `path` parameter. The
`FromSegments` implementation for `PathBuf` ensures that `path` cannot lead to
[path traversal attacks](https://www.owasp.org/index.php/Path_Traversal). With
this, a safe and secure static file server can be implemented in 4 lines:

```rust
#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}
```

[`FromSegments`]: https://api.rocket.rs/rocket/request/trait.FromSegments.html

## Format

A route can specify the data format it is willing to accept or respond with
using the `format` route parameter. The value of the parameter is a string
identifying an HTTP media type. For instance, for JSON data, the string
`application/json` can be used.

When a route indicates a payload-supporting method (`PUT`, `POST`, `DELETE`, and
`PATCH`), the `format` route parameter instructs Rocket to check against the
`Content-Type` header of the incoming request. Only requests where the
`Content-Type` header matches the `format` parameter will match to the route.

As an example, consider the following route:

```rust
#[post("/user", format = "application/json", data = "<user>")]
fn new_user(user: JSON<User>) -> T { ... }
```

The `format` parameter in the `post` attribute declares that only incoming
requests with `Content-Type: application/json` will match. (The `data` parameter
is described in the next section.)

When a route indicates a non-payload-supporting method (`GET`, `HEAD`, and
`OPTIONS`), the `format` route parameter instructs Rocket to check against the
`Accept` header of the incoming request. Only requests where the preferred media
type in the `Accept` header matches the `format` parameter will match to the
route.

As an example, consider the following route:

```rust
#[get("/user/<id>", format = "application/json")]
fn user(id: usize) -> JSON<User> { ... }
```

The `format` parameter in the `get` attribute declares that only incoming
requests with `application/json` as the preferred media type in the `Accept`
header will match.

## Request Guards

Request guards are one of Rocket's most powerful instruments. As the name might
imply, a request guard protects a handler from being called erroneously, based
on information contained in an incoming request. More specifically, a request
guard is a type that represents an arbitrary validation policy. The validation
policy is implemented through the [`FromRequest`] trait. Every type that
implements `FromRequest` is a request guard.

Request guards appear as inputs to handlers. An arbitrary number of request
guards can appear as arguments in a route handler. Rocket will automatically
invoke the [`FromRequest`] implementation for request guards before calling the
handler. Rocket only dispatches requests to a handler when all of its guards
pass.

As an example, the following dummy handler makes use of three request guards,
`A`, `B`, and `C`. An input can be identified as a request guard if it is not
named in the route attribute. This is why `param` is not a request guard.

```rust,ignore
#[get("/<param>")]
fn index(param: isize, a: A, b: B, c: C) -> ... { ... }
```

Request guards always fire in left-to-right declaration order. In the example
above, the order will be `A` followed by `B` followed by `C`. Failure is
short-circuiting; if one guard fails, the remaining are not attempted. To learn
more about request guards and implementing them, see the [`FromRequest`]
documentation.

[`FromRequest`]: https://api.rocket.rs/rocket/request/trait.FromRequest.html
[`Cookies`]: https://api.rocket.rs/rocket/http/enum.Cookies.html

### Retrieving Metadata

Sometimes we need data associated with a request that isn't a direct data input.
Rocket makes retrieving and validating such information easy through request
guards. As example, consider the built-in request guard, [`Cookies`]. Since
`Cookies` is a request guard, an argument of that type can simply be added to a
handler:

```rust
use rocket::http::Cookies;

#[get("/")]
fn index(cookies: Cookies) -> Option<String> {
    cookies.get("message")
        .map(|value| format!("Message: {}", value))
}
```

The [cookies example] on GitHub illustrates further use of the `Cookies` type to
get and set cookies, while the [`Cookies`] documentation contains full usage
information.

[cookies example]: https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/cookies

### Custom Guards

You can implement `FromRequest` for your own types. For instance, to protect a
`sensitive` route from running unless an `ApiKey` is present in the request
headers, you might create an `ApiKey` type that implements `FromRequest` and use
it as a request guard:

```rust
#[get("/sensitive")]
fn sensitive(key: ApiKey) -> &'static str { ... }
```

You might also implement `FromRequest` for an `AdminUser` type that validates
that the cookies in the incoming request authenticate an administrator. Then,
any handler with an `AdminUser` or `ApiKey` type in its argument list is assured
to only be invoked if the appropriate conditions are met. Request guards
centralize policies, resulting in a simpler, safer, and more secure
applications.

## Body Data

At some point, your web application will need to process body data. Data
processing, like much of Rocket, is type directed. To indicate that a handler
expects data, annotate it with `data = "<param>"`, where `param` is an argument
in the handler. The argument's type must implement the
[FromData](https://api.rocket.rs/rocket/data/trait.FromData.html) trait. It
looks like this, where `T: FromData`:

```rust
#[post("/", data = "<input>")]
fn new(input: T) -> String { ... }
```

### Forms

Forms are the most common type of data handled in web applications, and Rocket
makes handling them easy. Say your application is processing a form submission
for a new todo `Task`. The form contains two fields: `complete`, a checkbox, and
`description`, a text field. You can easily handle the form request in Rocket
as follows:

```rust
#[derive(FromForm)]
struct Task {
    complete: bool,
    description: String,
}

#[post("/todo", data = "<task>")]
fn new(task: Form<Task>) -> String { ... }
```

The `Form` type implements the `FromData` trait as long as its generic parameter
implements the
[FromForm](https://api.rocket.rs/rocket/request/trait.FromForm.html) trait. In
the example, we've derived the `FromForm` trait automatically for the `Task`
structure. `FromForm` can be derived for any structure whose fields implement
[FromFormValue](https://api.rocket.rs/rocket/request/trait.FromFormValue.html).
If a `POST /todo` request arrives, the form data will automatically be parsed
into the `Task` structure. If the data that arrives isn't of the correct
Content-Type, the request is forwarded. If the data doesn't parse or is simply
invalid, a customizable `400 - Bad Request` or `422 - Unprocessable Entity`
error is returned. As before, a forward or failure can be caught by using the
`Option` and `Result` types:

```rust
#[post("/todo", data = "<task>")]
fn new(task: Option<Form<Task>>) -> String { ... }
```

#### Lenient Parsing

FIXME: Write this.

#### Field Renaming

FIXME: Write this.

#### Field Validation

Fields of forms can be easily validated via implementations of the
`FromFormValue` trait. For example, if you'd like to verify that some user is
over some age in a form, then you might define a new `AdultAge` type, use it as
a field in a form structure, and implement `FromFormValue` so that it only
validates integers over that age:

```rust
struct AdultAge(usize);

impl<'v> FromFormValue<'v> for AdultAge {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<AdultAge, &'v RawStr> {
        match form_value.parse::<usize>() {
            Ok(age) if age >= 21 => Ok(AdultAge(age)),
            _ => Err(form_value),
        }
    }
}

#[derive(FromForm)]
struct Person {
    age: AdultAge
}
```

If a form is submitted with a bad age, Rocket won't call a handler requiring a
valid form for that structure. You can use `Option` or `Result` types for fields
to catch parse failures:

```rust
#[derive(FromForm)]
struct Person {
    age: Option<AdultAge>
}
```

The [forms](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/forms)
and [forms kitchen
sink](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/form_kitchen_sink)
examples on GitHub provide further illustrations.

### JSON

Handling JSON data is no harder: simply use the
[`JSON`](https://api.rocket.rs/rocket_contrib/struct.JSON.html) type:

```rust
#[derive(Deserialize)]
struct Task {
    description: String,
    complete: bool
}

#[post("/todo", data = "<task>")]
fn new(task: JSON<Task>) -> String { ... }
```

The only condition is that the generic type in `JSON` implements the
`Deserialize` trait from [Serde](https://github.com/serde-rs/json). See the
[JSON example] on GitHub for a complete example.

[JSON example]: https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/json

### Streaming

Sometimes you just want to handle the incoming data directly. For example, you
might want to stream the incoming data out to a file. Rocket makes this as
simple as possible via the
[Data](https://api.rocket.rs/rocket/data/struct.Data.html) type:

```rust
#[post("/upload", format = "text/plain", data = "<data>")]
fn upload(data: Data) -> io::Result<String> {
    data.stream_to_file("/tmp/upload.txt").map(|n| n.to_string())
}
```

The route above accepts any `POST` request to the `/upload` path with
`Content-Type` `text/plain`  The incoming data is streamed out to
`tmp/upload.txt` file, and the number of bytes written is returned as a plain
text response if the upload succeeds. If the upload fails, an error response is
returned. The handler above is complete. It really is that simple! See the
[GitHub example
code](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/raw_upload)
for the full crate.

## Query Strings

Query strings are handled just like forms. A query string can be parsed into any
structure that implements the `FromForm` trait. They are matched against by
appending a `?` followed by a dynamic parameter `<param>` to the path.

For instance, say you change your mind and decide to use query strings instead
of `POST` forms for new todo tasks in the previous forms example, reproduced
below:

```rust
#[derive(FromForm)]
struct Task { .. }

#[post("/todo", data = "<task>")]
fn new(task: Form<Task>) -> String { ... }
```

Rocket makes the transition simple: simply declare `<task>` as a query parameter
as follows:

```rust
#[get("/todo?<task>")]
fn new(task: Task) -> String { ... }
```

Rocket will parse the query string into the `Task` structure automatically by
matching the structure field names to the query parameters. If the parse fails,
the request is forwarded to the next matching route. To catch parse failures,
you can use `Option` or `Result` as the type of the field to catch errors for.

See [the GitHub
example](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/query_params)
for a complete illustration.

## Error Catchers

Routing may fail for a variety of reasons. These include:

  * A [request guard](#request-guards) returns `Failure`.
  * A handler returns a [`Responder`](/guide/responses/#responder) that fails.
  * No matching route was found.

If any of these conditions occurs, Rocket returns an error to the client. To do
so, Rocket invokes the _error catcher_ corresponding to the error's status code.
A catcher is like a route, except it only handles errors. Catchers are declared
via the `error` attribute, which takes a single integer corresponding to the
HTTP status code to catch. For instance, to declare a catcher for **404**
errors, you'd write:

```rust
#[error(404)]
fn not_found(req: &Request) -> String { ... }
```

As with routes, Rocket needs to know about a catcher before it is used to handle
errors. The process is similar to mounting: call the `catch` method with a list
of catchers via the `errors!` macro. The invocation to add the **404** catcher
declared above looks like:

```rust
rocket::ignite().catch(errors![not_found])
```

Unlike request handlers, error handlers can only take 0, 1, or 2 parameters of
types [Request](https://api.rocket.rs/rocket/struct.Request.html) and/or
[Error](https://api.rocket.rs/rocket/enum.Error.html). At present, the `Error`
type is not particularly useful, and so it is often omitted. The
[error catcher
example](https://github.com/SergioBenitez/Rocket/tree/v0.2.8/examples/errors) on
GitHub illustrates their use in full.

Rocket has a default catcher for all of the standard HTTP error codes including
**404**, **500**, and more.
