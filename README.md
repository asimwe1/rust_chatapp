# Rocket [![Build Status](https://travis-ci.com/SergioBenitez/Rocket.svg?token=CVq3HTkPNimYtLm3RHCn&branch=master)](https://travis-ci.com/SergioBenitez/Rocket)

Rocket is a work-in-progress web framework for Rust (nightly) with a focus on
ease-of-use, expressability, and speed. Here's an example of a complete Rocket
application:

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
    Rocket::new("localhost", 8000).mount_and_launch("/hello", routes![hello]);
}
```

Visiting `localhost:8000/hello/John/58`, for example, will trigger the `hello`
route resulting in the string `Hello, 58 year old named John!` being sent to the
browser. If an `<age>` string was passed in that can't be parsed as a `u8`, the
route won't get called, resulting in a 404 error.

## Overview

Rocket employs code generation to remove the need for boilerplate involved with
parsing requests and request parameters, prevent invalid and/or incorrect
requests from invoking a user's request handler, and  allow the user to define
what is valid and/or correct.

Rocket uses type _guards_, or _constraints_, on request handlers to
accomplishing it's safety and correctness goals. In Rocket, handlers are _only_
invoked if all types that appear in the request handler's argument list can be
derived from the incoming request.

In their simplest incarnation, guards can be types expected of parameters in a
matched path. This is illustrated in the previous example where the `hello`
request handler is only be invoked if the dynamic path parameter `<age>` parses
as a `u8`. Guards can also be derived directly from a request. For instance, you
can define an `AdminUser` type that can be derived only if the proper cookies
were sent along with the request. Then, by simply including the type in a
handler's argument list as follows:

    #[get("/admin/post/<id>")]
    fn admin(user: AdminUser, id: isize) ->  { .. }

you can be assured that the handler will be invoked _only_ if an administrative
user is logged in. Any number of such guards can be included. For example, to
retrieve the request's cookies along with the admin user, simply include the
`&Cookies` type in the argument list:

    #[get("/admin/post/<id>")]
    fn admin(user: AdminUser, cookies: &Cookies, id: isize) ->  { .. }

Full documentation about built-in request guards is coming soon.

## Building

### Nightly

Rocket requires a nightly version of Rust as it makes heavy use of syntax
extensions. This also means that the first two unwieldly lines in the Rust file
above are required.

### Examples

Try running the examples in the `examples/` folder. For instance, the following
sequence of commands builds and runs the `Hello, world!` example:

```
cd examples/hello_world
cargo run
```

Then visit `localhost:8000`. You should see `Hello, world!`.

### OS X

Apple has stopped shipping `openssl` with OS X.11. As such, if your build fails
to compile with some `openssl` related errors, you'll need to install `openssl`,
`cargo clean`, and then `cargo build` again. Here are some lightweight
instructions:

```
brew install openssl
brew link --force openssl
export OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include
export OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
```

