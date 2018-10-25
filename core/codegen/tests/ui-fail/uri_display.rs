#[macro_use] extern crate rocket;

#[derive(UriDisplay)]
struct Foo1;
//~^ ERROR not supported

#[derive(UriDisplay)]
struct Foo2();
//~^ ERROR not supported

#[derive(UriDisplay)]
enum Foo3 { }
//~^ ERROR not supported

#[derive(UriDisplay)]
enum Foo4 {
    Variant,
    //~^ ERROR not supported
}

#[derive(UriDisplay)]
struct Foo5(String, String);
//~^ ERROR exactly one

#[derive(UriDisplay)]
struct Foo6 {
    #[form(field = 123)]
    //~^ ERROR invalid value: expected string
    field: String,
}
