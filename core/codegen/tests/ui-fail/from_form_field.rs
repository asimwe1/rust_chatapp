#[macro_use] extern crate rocket;

#[derive(FromFormField)]
struct Foo1;

#[derive(FromFormField)]
struct Foo2(usize);

#[derive(FromFormField)]
struct Foo3 {
    foo: usize,
}

#[derive(FromFormField)]
enum Foo4 {
    A(usize),
}

#[derive(FromFormField)]
enum Foo5 { }

#[derive(FromFormField)]
enum Foo6<T> {
    A(T),
}

#[derive(FromFormField)]
enum Bar1 {
    #[field(value = 123)]
    A,
}

#[derive(FromFormField)]
enum Bar2 {
    #[field(value)]
    A,
}

fn main() { }
