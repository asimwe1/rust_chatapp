# Rust Web Framework

I really want a nice, easy to use, safe, and stupid fast web framework for
Rust. I don't want a monolithic Rails like thing. I'd much rather have
something that looks like Bottle. That is, use decorators to declare routes.

Nickel.rs looks kinda nice though (see example 2 below - that looks like, but
isn't Nickel).

Here's what the simplest program might look like with this framework:

```rust
#[route("/")]
fn home() -> Response {
  Response::string("Hello, world!")
}

fn main() {
  RustWebFramework::run("localhost");
}
```

Alternatively...

```rust
fn home() -> Response {
  Response::string("Hello, world!")
}

fn main() {
  route! {
    get '/' => home,
  }

  RustWebFramework::run("localhost");
}
```

## Arguments

Here's what a route that takes arguments might look like:

```rust
#[route("/<page>")]
fn home(page: &str) -> Response {
  Response::string(page)
}
```

The really neat thing here is that the `route` macro will typecheck the
function signature. The signature should also have a return type of `Response`
(or whatever the response type ends up being) and take a number of equivalent
to those present in the route. The type of the arguments can be any `T` that
implements `From<&str>`. The conversion will be done automatically by the route
handler. As such, the following will work as expected:

```rust
#[route("/users/<id>")]
fn home(id: isize) -> Response {
  let response_string = format!("User ID: {}", id);
  Response::string(response_string)
}
```

If the conversion fails, the router should 1) print out a debug error message
and return some user-set no-route-exists things, and 2) allow the programmer to
catch the failure if needed. I'm not quite sure what the best way to allow 2)
is at the moment. Here are a couple of ideas:

  1.  Add an `else` parameter to the `route` macro that will take in the name
      of a function to call with the raw string (and more) if the routing
      fails:

          #[route("/users/<id>", else = home_failed)]
          fn home(id: isize) -> Response { ... }
          fn home_failed(route: &str) -> Response { ... }

  2.  Allow the parameter type to be `Result<T>`. Then the route is always
      called and the user has to check if the conversion was successful or not.

  3.  Pass it off as an error type to another handler.

Open questions here:

    1.  What syntax should be used to match a path component to a regular
        expression? If for some parameter, call it `<name>`, of type `&str`, we
        want to constrain matches to the route to `name`s that match some
        regular expression, say `[a-z]+`, how do we specify that? Bottle does:

            <name:re:[a-z]+>

        We can probably just do:

            <name: [a-z]+>


## Methods

A different HTTP method can be specified with the `method` `route` argument:

```rust
#[route(method = POST, "/users")]
fn add_user(name: &str, age: isize) -> Response { ... }
```

Or, more succinctly:

```rust
#[POST("/users")]
fn add_user(name: &str, age: isize) -> Response { ... }
```

## Route Priority

Do we allow two routes to possibly match a single path? Can we even determine
that no two paths conflict given regular expressions? Answer: Yes
(http://arstechnica.com/civis/viewtopic.php?f=20&t=472178). And if so, which
route gets priority? An idea is to add a `priority` parameter to the `route`
macro:

For example:

```rust
#[GET("/[name: [a-zA-Z]+", priority = 0)]
#[GET("/[name: [a-z]+", priority = 1)]
```

The first route allows lower and uppercase letter, while the second route only
allows lowercase letters. In the case that the entire route has lowercase
letters, the route with the higher priority (1, here) gets called, i.e., the
second one.

## Error Pages

There's a route for error pages, too:

```rust
#[route(method = ERROR, status = 404)]
fn page_not_found(...not sure what goes here yet...) -> Response { .. }
```

Or, more succinctly:

```rust
#[error(404)]
fn page_not_found(...not sure what goes here yet...) -> Response { .. }
```

## Open Questions

1.  How is HTTP data handled? (IE, interpret `Content-Type`s)
    -   Form Data
    -   JSON: Would be nice to automatically convert to structs.

2.  What about Query Params?

3.  How are cookies handled?

    Presumably you would set them via `Response` and get them via...?

4.  Easy support for (but don't bundle it in...) templating would be nice.
    Bottle lets you do:

        #[view("template_name")]
        fn hello(...) -> HashMap { .. }

    and automatically instantiates the template `template_name` with the
    parameters from the HashMap.

5.  Autoreloading. Maybe use the unix-y reloading thing. Maybe not.

6.  Plugins? Would be nice to easily extend routes.

7.  Pre-post hooks/filters?

8.  Caching?

9.  Session support?

    This is basically a server-side local store identified via an ID in a
    cookie.

    http://entrproject.org/

10. Environment support? (debug vs. production vs. test, etc.)

11. Model validation?

12. Internationalization?

13. Be faster than https://github.com/julienschmidt/httprouter.

For 2, 3: the obvious solution is to have a `Request` object with that
information. Do we need that, though? Is there something better?
