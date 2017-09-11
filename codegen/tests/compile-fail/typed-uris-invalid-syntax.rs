#![feature(plugin, decl_macro, custom_derive)]
#![plugin(rocket_codegen)]
#![allow(dead_code, unused_variables)]

extern crate rocket;

#[post("/<id>/<name>")]
fn simple(id: i32, name: String) -> &'static str { "" }

fn main() {
    uri!(simple: id = 100, "Hello"); //~ ERROR cannot be mixed
    uri!(simple: "Hello", id = 100); //~ ERROR cannot be mixed
    uri!(simple,); //~ ERROR expected one of `::`, `:`, or `<eof>`
    uri!(simple:); //~ ERROR expected argument list
    uri!("mount"); //~ ERROR expected `,`, found `<eof>`
    uri!("mount",); //~ ERROR expected identifier
    uri!(); //~ ERROR cannot be empty
    uri!(simple: id = ); //~ ERROR expected argument list
        //~^ ERROR expected expression
}
