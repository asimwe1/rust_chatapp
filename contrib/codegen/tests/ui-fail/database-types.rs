extern crate rocket;
#[macro_use] extern crate rocket_contrib;

struct Unknown;

#[database("foo")]
//~^ ERROR Unknown: rocket_contrib::databases::Poolable
//~^^ ERROR is private
//~^^^ ERROR no method named `get`
struct A(Unknown);
//~^ ERROR Unknown: rocket_contrib::databases::Poolable
//~^^ ERROR Unknown: rocket_contrib::databases::Poolable

#[database("foo")]
//~^ ERROR Vec<i32>: rocket_contrib::databases::Poolable
//~^^ ERROR is private
//~^^^ ERROR no method named `get`
struct B(Vec<i32>);
//~^ ERROR Vec<i32>: rocket_contrib::databases::Poolable
//~^^ ERROR Vec<i32>: rocket_contrib::databases::Poolable

fn main() {  }
