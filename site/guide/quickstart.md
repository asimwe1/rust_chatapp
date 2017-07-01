# Quickstart

Rocket requires a recent nightly version of Rust. We recommend you use
[rustup](https://rustup.rs/) to install or configure such a version. If you
don't have Rust installed, the [getting started](/guide/getting-started) section
guides you through installing Rust.

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
