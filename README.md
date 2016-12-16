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
  * [API Documentation](https://api.rocket.rs): The "rustdocs" (API documentation).

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

## Performance

Rocket is designed to be performant. At this time, its performance is
[bottlenecked by the Hyper HTTP
library](https://github.com/SergioBenitez/Rocket/issues/17). Even so, Rocket
currently performs _better_ than the latest version of Hyper on a simple "Hello,
world!" benchmark:

**Machine Specs:**

  * **Logical Cores:** 12 (6 cores x 2 threads)
  * **Memory:** 24gb ECC DDR3 @ 1600mhz
  * **Processor:** Intel Xeon X5675 @ 3.07GHz
  * **Operating System:** Mac OS X v10.11.6

**Hyper v0.10.0-a.0** (46 LOC) results (best of 3, +/- 300 req/s, +/- 1us):

	Running 10s test @ http://localhost:3000
	  2 threads and 10 connections
	  Thread Stats   Avg      Stdev     Max   +/- Stdev
		Latency   177.61us   37.04us   1.77ms   78.55%
		Req/Sec    27.56k     1.07k   30.37k    69.31%
	  553567 requests in 10.10s, 77.08MB read
	Requests/sec:  54811.36
    Transfer/sec:      7.63MB

**Rocket v0.0.11** (8 LOC) results (best of 3, +/- 200 req/s, +/- 0.5us):

	Running 10s test @ http://localhost:80
	  2 threads and 10 connections
	  Thread Stats   Avg      Stdev     Max   +/- Stdev
		Latency   170.07us   28.02us 484.00us   72.50%
		Req/Sec    28.55k   830.36    30.43k    69.80%
	  574017 requests in 10.10s, 79.92MB read
	Requests/sec:  56836.22
	Transfer/sec:      7.91MB

**Summary:**

  * Rocket throughput higher by 3.7% (higher is better)
  * Rocket latency lower by 4.0% (lower is better)

## Future Improvements

Rocket is currently built on a synchronous HTTP backend. Once the Rust
aynchronous I/O libraries have stabalized, a migration to a new, more performant
HTTP backend is planned. We expect performance to improve significantly at that
time. The [Stabilize HTTP
Library](https://github.com/SergioBenitez/Rocket/issues/17) issue tracks the
progress on this front.
