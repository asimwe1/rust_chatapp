# Overview

A quick glance at what makes Rocket special.

# Introduction

This overview is a concise introduction to Rocket. There's also a [full,
detailed guide](guide). If you want to get started immediately, see
[quickstart](guide/quickstart) or the [getting started
guide](guide/getting_started). Otherwise, welcome!

Rocket makes writing web applications easy, fast, and fun. It is Rocket's goal
to have you write as little code as necessary to accomplish your goal. In
practice, this means that your code will be free of boilerplate and that the
common tasks will be handled for you.

# Routes and Handlers

Rocket applications are centered around routes and handlers.

A _handler_ is simply a function that takes an arbitrary number of arguments and
returns a response. A _route_ is a combination of:

  * A set of parameters to match an incoming request against.
  * A handler to process the request and return a response.

The set of parameters to match against includes static paths, dynamic paths,
path segments, forms, query strings, request format specifiers, and body data.
Rocket uses attributes, which look like function decorators in other languages,
to make declaring routes easy. Routes are declared by annotating a function, the
handler, with the set of parameters to match against. A complete route
declaration looks like this:

```rust
#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}
```

This declares the `world` route which matches against the static path
`"/world"` for incoming `GET` requests.

## Mounting

Before Rocket dispatches requests to a route, the route needs to be _mounted_ on
an instance of `Rocket`.

Mounting a route is like namespacing it. Routes are mounted happens via the
`mount` method on a `Rocket` instance. Rocket instances can be created with the
`ignite()` static method.

The `mount` method takes **1)** a path to namespace a list of routes under, and
**2)** a list of route handlers through the `route!` macro. The `route!` macro
ties Rocket's code generation to your application. To mount the `world` route we
declared above, we would use the following code:

```rust
rocket::ignite().mount(“/hello”, routes![world])
```

All together, this creates a new `Rocket` instance via the `ignite` function and
mounts the `world` route to the `"/hello"` path. As a result, requests to the
`"/hello/world"` path will be directed to the `world` function.

## Launching

Now that Rocket knows about the route, you can tell Rocket to start accepting
requests via the `launch` method. The method starts up the server and waits for
incoming requests. When a request arrives, Rocket finds the matching route and
dispatches the request to the route.

We typically call `launch` from the `main` function. Our complete _Hello,
world!_ application thus looks like:

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/hello", routes![world]).launch();
}
```

Note that we've added the `#![feature(plugin)]` and `#![plugin(rocket_codegen)]`
lines to tell Rust that we'll be using Rocket's code generation plugin. We've
also imported the `rocket` crate into our namespace via `extern crate rocket`.
Finally, we call the `launch` method in the `main` function.

If we were to run the application above, our console would show:

```sh
🔧  Configured for development.
    => listening: localhost:8000
    => logging: Normal
    => session key: false
🛰  Mounting '/world':
    => GET /hello/world
🚀  Rocket has launched from localhost:8000...
```

If we now visit `localhost:8000/hello/world`, we would see `Hello, world!`,
exactly as we'd expect.

By the way, this example's complete crate, ready to `cargo run`, can be found on
[Github](https://github.com/SergioBenitez/Rocket/tree/master/examples/hello_world).
You can find dozens of other complete examples, spanning all of Rocket's
features, in the [Github examples
directory](https://github.com/SergioBenitez/Rocket/tree/master/examples/).

# Requests

If all we could do was match against static paths like `"/world"`, Rocket
wouldn't be much fun. Of course, Rocket allows you to match against just about
any information in an incoming request.

## Dynamic Paths

You can declare path segments as dynamic by using angle brackets around variable
names in a route's path. For example, if we wanted to say _Hello!_ to anything,
not just the world, we could declare a route and handler like so:

```rust
#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

If we were to mount the path at the root (`.mount("/", routes![hello])`), then
any request to a path with two non-empty segments, where the first segment is
`hello`, will be dispatched to the `hello` route. For example, if we were to
visit `/hello/John`, the application would respond with `Hello, John!`.

You can have any number of dynamic path segments, and the type of the path
segment can be any type that implements the [FromParam
trait](https://api.rocket.rs/rocket/request/trait.FromParam.html), including
your own! Here's a somewhat complicated route to illustrate:

```rust
#[get("/hello/<name>/<age>/<cool>")]
fn hello(name: &str, age: u8, cool: bool) -> String {
    if cool {
      format!("You're a cool {} year old, {}!", age, name)
    } else {
      format!("{}, we need to talk about your coolness.", name)
    }
}
```

## Forwarding

What if `cool` ain't a `bool`? Or, what if `age` isn't a `u8`? In this case, the
request is _forwarded_ to the next matching route, if there is any. This
continues until a route doesn't forward the request or there are no more routes
to try. When there are no remaining matching routes, a 404 error, which is
customizable, is returned.

Routes are tried in increasing _rank_ order. By default, routes with static
paths have a rank of 0 and routes with dynamic paths have a rank of 1. Ranks can
be manually set with the `rank` route parameter.

To illustrate, consider the following two routes:

```rust
#[get("/user/<id>")]
fn user(id: usize) -> T { ... }
```

```rust
#[get("/user/<id>", rank = 2)]
fn user_str(id: &str) -> T { ... }
```

Notice the `rank` parameter in the second route, which sets the rank of the
`user_str` route to 2. If we run this application with both routes mounted at
the root (`.mount("/", routes![user, user_str])`), requests to any route where
the `<id>` path segment is an unsigned integer will be handled by the `user`
route. If the `<id>` path segment is not an unsigned integer, the `user` route
will forward the request. Rocket will then dispatch the request to the next
matching route, `user_str`.

Forwards can be _caught_ by using a `Result` or `Option` type. For example, if
the type of `id` in the `user` function was `Result<usize, &str>`, an `Ok`
variant would indicate that `<id>` was a valid `usize`, while an `Err` would
indicate that `<id>` was not a `usize`. The `Err`'s value would contain the
string that failed to parse as a `usize`.

By the way, if you were to omit the `rank` parameter in the `user_str` route,
Rocket would emit a warning indicating that the `user` and `user_str` routes
_collide_, or can both match against an incoming request. The `rank` parameter
resolves this collision.

## Request Guards

Sometimes we need data associated with a request that isn't a direct input.
Headers and cookies are a good example of this: they simply tag along for the
ride.

Rocket makes retrieving such information easy: simply add any number of
parameters to the request handler with types that implement the `FromRequest`
trait. If the data can be retrieved from the incoming request, the handler is
called. If it cannot, the handler isn't called, and the request is forwarded on.
In this way, these parameters also act as _guards_: they protect the request
handler from being called erroneously.

For example, to retrieve cookies and the Content-Type header from a request, we
can declare a route as follows:

```rust
#[get("/")]
fn index(cookies: &Cookies, content: ContentType) -> String { ... }
```

You can implement `FromRequest` for your own types as well. For example, you
might implement `FromRequest` for an `AdminUser` type that validates that the
cookies in the incoming request authenticate an administrator. Then, any handler
with the `AdminUser` type in its argument list is assured that it will only be
invoked if an administrative user is logged in. This centralizes policies,
resulting in a simpler, safer, and more secure application.

## Data

At some point, your web application will need to process data, and Rocket makes
it as simple as possible. Data processing, like much of Rocket, is type
directed. To indicate that a handler expects data, annotate a route with a `data
= "<param>"` parameter, where `param` is an argument in the handler of a type
that implement the `FromData` trait.

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
implements the `FromForm` trait. In the example, we've derived the `FromForm`
trait automatically for the `Task` structure. If a `POST /todo` request arrives,
the form data will automatically be parsed into the `Task` structure. If the
data that arrives isn't of the correct content-type, the request is forwarded.
If the data is simply invalid, a customizable `400 Bad Request` error is
returned. As before, a forward or failure can be caught by using the `Option`
and `Result` types.

### Query Strings

If you change your mind and decide to use query strings instead of `POST` forms
for the todo task, Rocket makes the transition simple: simply declare `<task>`
as a query parameter as follows:

```rust
#[get("/todo?<task>")]
fn new(task: Task) -> String { ... }
```

This works because Rocket uses the `FromForm` trait to parse structures from
query parameters as well.

### JSON

Handling JSON data is no harder: simply use the `JSON` type:

```rust
#[derive(Deserialize)]
struct Task {
    description: String,
    complete: bool
}

#[post("/todo", data = "<task>")]
fn new(task: JSON<Task>) -> String { ... }
```

The only condition is that the generic type to `JSON` implements the
`Deserialize` trait.

### Streaming Data

Sometimes you just want to handle the incoming data directly. For example, you
might want to stream the incoming data out to a file. Rocket makes this as
simple as possible:

```rust
#[post("/upload", format = "text/plain", data = "<data>")]
fn upload(data: Data) -> io::Result<Plain<String>> {
    data.stream_to_file("/tmp/upload.txt").map(|n| Plain(n.to_string()))
}
```

The route above accepts any `POST` request to the `/upload` path with
`Content-Type` `text/plain`  The incoming data is streamed out to
`tmp/upload.txt` file, and the number of bytes written is returned as a plain
text response if the upload succeeds. If the upload fails, an error response is
returned. The handler above is complete. It really is that simple! See the
[Github example
code](https://github.com/SergioBenitez/Rocket/blob/master/examples/raw_upload/src/main.rs)
for the full crate.

# Responses

Up until the last example, we've been returning the type of `String` from
request handlers. In fact, any type that implements the `Responder` trait can be
returned, including your own!

## Result

One of the most common types to return is `Result`. Returning a `Result` means
one of two things: If the error type tself implements `Responder`, the response
will come from either the `Ok` or `Err` value, whichever the variant is. If the
error type does _not_ implement `Responder`, a customizable internal server
error will be returned.

## JSON

Responding with JSON data is just as simple: simply return a JSON type. For
example, to respond with the JSON value of the `Task` structure from previous
examples, we would write:

```rust
#[derive(Serialize)]
struct Task { ... }

#[get("/todo")]
fn todo() -> JSON<Task> { ... }
```

Note that the generic type for the JSON response type must implement
`Serialize`.

## Templates

Rocket has built-in support for templating. To respond with a rendered template,
simply return a `Template` type:

```rust
#[get("/")]
fn index() -> Template {
  let context = ...;
  Template::render("index", &context)
}
```

The `render` static method takes in the name of a template (here, `"index"`) and
a value to use as the _context_ for the template's rendering. The context must
contain all of the parameters expected by the template.

Templating support in Rocket is engine agnostic. The engine used to render a
template depends on the template file's extension. For example, if a file ends
with `.hbs`, Handlebars is used, while if a file ends with `.tera`, Tera is
used.

## Streaming

When a large amount of data is to be returned, it is often better to stream the
data to the client so as to avoid consuming large amounts of memory. Rocket
provides the `Stream` type to accomplish this. The `Stream` type can be created
from any `Read` type. For example, to stream from a local Unix stream, we might
write:

```rust
#[get("/stream")]
fn stream() -> io::Result<Stream<UnixStream>> {
    let mut unix = UnixStream::connect("/path/to/my/socket")?;
    Stream::from(unix)
}

```

Rocket takes care of the rest.

# What's next?

That was just a taste of what Rocket has to offer! There's so much more:

  * [Quickstart](guide/quickstart): How to get started as quickly as possible.
  * [Getting Started](guide/getting_started): How to start your first project.
  * [Overview](overview): A brief introduction.
  * [Guide](guide): A detailed guide and reference to every component.
  * [API Documentation](https://api.rocket.rs): The "rustdocs" (API documentation).
