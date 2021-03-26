#![feature(custom_test_frameworks)]
#![test_runner(criterion::runner)]

#[cfg_attr(test, macro_use)]
extern crate criterion_macro;

#[cfg(test)] mod routing;

pub fn main() {
    eprintln!("help: cargo bench");
}
