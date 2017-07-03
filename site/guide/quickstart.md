# Quickstart

Before you can start writing a Rocket application, you'll need a **nightly**
version of Rust installed. We recommend you use [rustup](https://rustup.rs/) to
install or configure such a version. If you don't have Rust installed and want
extra guidance installing it, the [getting started](/guide/getting-started)
section provides a guide.

## Running Examples

The absolute fastest way to start experimenting with Rocket is to clone the
Rocket repository and run the included examples in the `examples/` directory.
For instance, the following set of commands runs the `hello_world` example:

```sh
git clone https://github.com/SergioBenitez/Rocket
cd Rocket
git checkout v0.2.8
cd examples/hello_world
cargo run
```

There are numerous examples in the `examples/` directory. They can all be run
with `cargo run`.
