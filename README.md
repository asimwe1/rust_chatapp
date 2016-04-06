# Rocket

[![Build Status](https://travis-ci.com/SergioBenitez/rocket.svg?token=CVq3HTkPNimYtLm3RHCn&branch=master)](https://travis-ci.com/SergioBenitez/rocket)

Rocket is a work-in-progress web framework for Rust (nightly) with a focus on
ease-of-use, expressability, and speed. Here's an example of a complete Rocket
application:

```rust
#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::Rocket;

#[route(GET, path = "/<name>/<age>")]
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

Rocket requires a nightly version of Rust as it makes heavy use of syntax
extensions. This also means that the first two unwieldly lines in the Rust file
above are required.

## Building

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

