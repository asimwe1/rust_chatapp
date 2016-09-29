# Overview

A quick glance at what makes Rocket special.

# Introduction

This overview is a concise introduction to Rocket. There's also a [full,
detailed guide](guide). If you want to get started immediately, see
[quickstart](guide/quickstart) or the [getting started
guide](guide/getting_started). Otherwise, welcome!

Rocket makes writing web applications easy, fast, and fun. Below is a complete
Rocket application. In fact, it's [one of many](thisexample) complete, runnable
examples in [Rocket's git repository](github). Can you figure out what it does?

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::Rocket;

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    let mut rocket = Rocket::ignite();
    rocket.mount("/hello", routes![hello]);
    rocket.launch()
}
```

If you were to run this application, your console would show:

```sh
🛰  Mounting '/hello':
    => GET /hello/<name>/<age>
🚀  Rocket has launched from localhost:8000...
```

Here's a quick summary of what it does: first, on lines 7 - 10, it declares the
`hello` route to `GET /<name>/<age>`, which returns a `String` formatted with
`name` and `age` from the dynamic path. Then, in the `main` function, it creates
a new `Rocket` instance, mounts the `hello` route at `"/hello"`, and launches
the application.

That's it! Let's break this down.

We'll start with lines 1 and 2. Rocket depends on the latest version Rust
nightly; it makes extensive use of Rust's code generation facilities through
compiler plugins. Plugins are still experimental, so we have to tell Rust that
we're okay with that by writing `#![feature(plugin)]`. We also have to tell the
compiler to use Rocket's code generation crate during compilation with
`#![plugin(rocket_codegen)]`. Lines 4 and 5 bring `rocket::Rocket` into the
namespace.

# Routes

The fun begins on line 7, where the `hello` route and request handler are
declared.

Rocket applications are composed primarily of request handlers and routes. A
_request handler_ is a function that takes an arbitrary number of arguments and
returns a response. A _route_ is a combination of:

  * A set of parameters to match an incoming request against.
  * A request handler to process the request and return a response.

The set of parameters to match against includes static paths, dynamic paths,
path segments, forms, query strings, and request format specifiers. Rocket uses
Rust attributes, which look like function decorators in other languages, to make
declaring routes easy. Routes are declares by annotating a function with the set
of parameters to match against. A complete route declaration looks like:

```rust
#[get("/index")]
fn index() -> &str { "Hello, World!" }
```

You can also use `put`, `post`, `delete`, and `patch` in place of `get`.

## Dynamic Paths

The `hello` route declaration beginning on line 7 of our example applications
tells Rocket that the `hello` function will handle HTTP `GET` requests to the
`<name>/<age>` path. The handler uses `name` and `age` from the path to format
and return a `String` to the user. Here are lines 7 - 10 again:

```rust
#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}
```

The `<name>` and `<age>` parts of the path are _dynamic_: the actual values for
these segments won't be known until someone visits a matching URL. For example,
if someone visit `Mike/21`, `<name>` will be `"Mike"`, and `<age>` will be `21`.
If someone else visits `Bob/91`, `<name>` and `<age>` will be `"Bob"` and
`91`, respectively. Rocket automatically parses dynamic path segments and
passes them to the request handler in variables with matching names. This
means that `name` and `age` can be used immediately in the handler - no
parsing, no checking.

But wait: what happens if someone goes to a URL with an `<age>` that isn't a
valid `u8`? In that case, Rocket doesn't call the handler. Instead, it
_forwards_ the request to the next matching route, if any, and ultimately
returns a `404` if all of them fail. If you want to know if the user passed in a
bad `<age>`, simply use a `Result<u8, &str>` or an `Option<u8>` type for `age`
instead. For more details on routing, route collisions, and much more see the
[routing](guide/routing) chapter of the guide.

Oh, one more thing before we move on! Notice how dynamic path parameters can be
of different types? Actually, path parameters can be of _any_ type, as long as
that type implements Rocket's `FromParam` trait. Rocket uses the `FromParam`
implementation to parse and validate the parameter for you automatically. We've
implemented `FromParam` for plenty of types in the standard library. See the
[FromParam](docs) documentation for more.

## Mounting

Now that we understand the `hello` route, let's move on to lines 13 - 14. Before
Rocket dispatches requests to a route, the route needs to be _mounted_. And
before we can mount a route, we need an instance of `Rocket`.

Mounting a route is like namespacing it. Routes can be mounted any number of
times. Mounting happens with the `mount` method on a `Rocket` instance, which
itself is created with the `ignite()` static method. The `mount` method takes a
list of route handlers given inside of the `route!` macro. The `route!` macro
ties Rocket's code generation to your application. If you'd like to learn more
about the `route!` macro, see the [internals guide](guide/internals).

Let's look at lines 13 - 14 again, which we reproduce below:

```rust
let mut rocket = Rocket::ignite();
rocket.mount(“/hello”, routes![hello]);
```

Line 13 creates the new `Rocket` instance, and line 14 mounts the `hello` route
at the `"/hello"` path. This makes the `hello` handler available at
`/hello/<name>/<age>`. Notice how the mounting path is prepended to the route's
path. There's a ton more information about [mounting in the
guide](/guides/mounting).

## Launching

Now that the route is declared and mounted, the application is ready to launch!
To launch an application and have Rocket start listening for and dispatching
requests, simply call `launch` on the Rocket instance where routes are mounted.
This happens on line 14. Here it is again:

```
rocket.launch()
```

Again, running our full example will show the following in the console:

```sh
🛰  Mounting '/hello':
    => GET /hello/<name>/<age>
🚀  Rocket has launched from localhost:8000...
```

If you visit `http://localhost:8000/hello/Mike/21`, you'll see "Hello, 21 year
old named Mike!". If you have the example running, try visiting other valid and
invalid paths and see what happens! This example's complete crate, ready to
`cargo run`, can be found at
[Github](https://github.com/SergioBenitez/Rocket/tree/master/examples/hello_world).

# Requests

There's a lot more we can do with requests. The [requests](guide/requests)
chapter of the guide talks about requests in details. We'll give you a short
overview of some of the more important and useful features here.

## Forms and Queries

Handling forms and query parameters couldn't be easier: declare a form or query
parameter in the route attribute and handler, then ensure that its type
implements (the automatically derivable) `FromForm`.

Form parameters are declared by adding `form = "<param_name>"` to the route
attribute. Say your application is processing a form submission for a new todo
`Task`. The form contains two fields: `complete`, a checkbox, and `description`,
a text field. You can easily handle the form request in Rocket as follows:

```rust
#[derive(FromForm)]
struct Task {
    description: String,
    complete: bool
}

#[post("/todo", form = "<task>")]
fn new(task: Task) -> String {
    ...
}
```

If you change your mind and want to use query strings for the form instead,
simple declare `<task>` as a query parameter as follows:

```rust
#[get("/todo?<task>")]
fn new(task: Task) -> String {
    ...
}
```

If the form request is invalid according to the form's type, the handler doesn't
get called. Just like in path parameters, you can use `Option` or `Result` in
form structure fields to be notified of parsing errors. You can also easily
define your own types to validate forms and queries against. For more details,
see the [forms](guide/forms) and [queries](guide/queries) chapters of the guide.

## Guards

In addition to `FromParam` types, you can include any number of types that
implement the `FromRequest` trait in handler arguments. For example, to
retrieve cookies from a request, you can use a parameter of `&Cookie` type in a
request handler:

```rust
#[get("/hello")]
fn hello(cookies: &Cookies) -> ..
```

## JSON

# Responses

## Responder

## Templates

## JSON

# What's next?

