# Rocket

Rocket is a work-in-progress web framework for Rust (nightly) with a focus on
ease-of-use, expressability, and speed. It currently does not work. But, when it
does, the following will be the canonical "Hello, world!" example:

```rust
#![feature(plugin)]
#![plugin(rocket_macros)]

extern crate rocket;
use rocket::{Rocket, Request, Response, Method, Route};

#[route(GET, path = "/hello")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    let mut rocket = Rocket::new("localhost", 8000);
    rocket.mount_and_launch("/", routes![hello]);
}
```

Rocket requires a nightly version of Rust as it makes heavy use of syntax
extensions. This also means that the first two unwieldly lines in the Rust file
above are required.

## Building

Try running the examples in the `examples/` folder. For instance, the following
sequence of commands builds the `Hello, world!` example:

```
cd examples/hello
cargo build
cargo run
```

### OS X

Apple has stopped shipping `openssl` with OS X.11. As such, if your build fails
compile, you'll need to install `openssl`, `cargo clean`, and then `cargo build`
again. Here are some lightweight instructions:

```
brew install openssl
brew link --force openssl
export OPENSSL_INCLUDE_DIR=`brew --prefix openssl`/include
export OPENSSL_LIB_DIR=`brew --prefix openssl`/lib
```

