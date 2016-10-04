# Rocket [![Build Status](https://travis-ci.com/SergioBenitez/Rocket.svg?token=CVq3HTkPNimYtLm3RHCn&branch=master)](https://travis-ci.com/SergioBenitez/Rocket)

Rocket is a work-in-progress web framework for Rust (nightly) with a focus on
ease-of-use, expressability, and speed. Here's an example of a complete Rocket
application:

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

fn main() {
    rocket::ignite().mount("/hello", routes![hello]).launch();
}
```

Visiting `localhost:8000/hello/John/58`, for example, will trigger the `hello`
route resulting in the string `Hello, 58 year old named John!` being sent to the
browser. If an `<age>` string was passed in that can't be parsed as a `u8`, the
route won't get called, resulting in a 404 error.

## Documentation

Rocket is extensively documented:

  * [Quickstart](guide/quickstart): How to get started as quickly as possible.
  * [Getting Started](guide/getting_started): How to start your first project.
  * [Overview](overview): A brief introduction.
  * [Guide](guide): A detailed guide and reference to every component.
  * [API Documentation](http://api.rocket.rs/rocket): The "rustdocs" (API documentation).

## Building

### Nightly

Rocket requires a nightly version of Rust as it makes heavy use of syntax
extensions. This means that the first two unwieldly lines in the introductory
example above are required.

### Core, Codegen, and Contrib

All of the Rocket libraries are managed by Cargo. As a result, compiling them is
simple.

  * Core: `cd lib && cargo build`
  * Codegen: `cd codegen && cargo build`
  * Contrib: `cd contrib && cargo build`

### Examples

Rocket ships with an extensive number of examples in the `examples/` directory
which can be compiled and run with Cargo. For instance, the following sequence
of commands builds and runs the `Hello, world!` example:

```
cd examples/hello_world
cargo run
```

You should see `Hello, world!` by visiting `http://localhost:8000`.

## Testing

To test Rocket, simply run `./scripts/test.sh` from the root of the source tree.
This will build and test the `core`, `codegen`, and `contrib` libraries as well
as all of the examples. This is the script that gets run by Travis CI.

### Core

Testing for the core library is done inline in the corresponding module. For
example, the tests for routing can be found at the bottom of the
`lib/src/router/mod.rs` file.

### Codegen

Code generation tests can be found in `codegen/tests`. We use the
[compiletest](https://crates.io/crates/compiletest_rs) library, which was
extracted from `rustc`, for testing. See the [compiler test
documentation](https://github.com/rust-lang/rust/blob/master/COMPILER_TESTS.md)
for information on how to write compiler tests.

## Contributing

Contributions are absolutely, positively welcome and encouraged! Contributions
come in many forms. You could:

  1. Submit a feature request or bug report as an [issue](https://github.com/SergioBenitez/Rocket/issues).
  2. Ask for improved documentation as an [issue](https://github.com/SergioBenitez/Rocket/issues).
  3. Comment on [issues that require
        feedback](https://github.com/SergioBenitez/Rocket/issues?q=is%3Aissue+is%3Aopen+label%3A%22feedback+wanted%22).
  4. Contribute code via [pull requests](https://github.com/SergioBenitez/Rocket/pulls).

We aim to keep Rocket's code quality at the highest level. This means that any
code you contribute must be:

  * **Commented:** Public items _must_ be commented.
  * **Documented:** Exposed items _must_ have rustdoc comments with
    examples, if applicable.
  * **Styled:** Your code should be `rustfmt`'d when possible.
  * **Simple:** Your code should accomplish its task as simply and
     idiomatically as possible.
  * **Tested:** You must add (and pass) convincing tests for any functionality you add.
  * **Focused:** Your code should do what it's supposed to do and nothing more.

All pull requests are code reviewed and tested by the CI.
