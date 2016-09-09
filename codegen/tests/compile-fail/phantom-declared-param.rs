#![feature(plugin)]
#![plugin(rocket_codegen)]

#[get("/<param>")] //~ ERROR declared
fn get() {  } //~ ERROR isn't in the function signature

#[get("/<a>")] //~ ERROR declared
fn get2() {  } //~ ERROR isn't in the function signature

#[get("/a/b/c/<a>/<b>")]
    //~^ ERROR 'a' is declared
    //~^^ ERROR 'b' is declared
fn get32() {  }
    //~^ ERROR isn't in the function signature
    //~^^ ERROR isn't in the function signature

fn main() {  }
