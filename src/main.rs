#![feature(plugin)]
#![plugin(rocket_macros)]

#[route(POST, path = "/")]
fn function(_x: usize, _y: isize) {

}

#[route(GET, path = "/")]
fn main() {
    println!("Hello, world!");
    function(1, 2);
}
