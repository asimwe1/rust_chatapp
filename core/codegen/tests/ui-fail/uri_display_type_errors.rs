#[macro_use] extern crate rocket;

struct BadType;

#[derive(UriDisplay)]
struct Bar1(BadType);
//~^ ERROR UriDisplay

#[derive(UriDisplay)]
struct Bar2 {
    field: BadType,
    //~^ ERROR UriDisplay
}

#[derive(UriDisplay)]
struct Bar3 {
    field: String,
    bad: BadType,
    //~^ ERROR UriDisplay
}

#[derive(UriDisplay)]
enum Bar4 {
    Inner(BadType),
    //~^ ERROR UriDisplay
}

#[derive(UriDisplay)]
enum Bar5 {
    Inner {
        field: BadType,
        //~^ ERROR UriDisplay
    },
}

#[derive(UriDisplay)]
enum Bar6 {
    Inner {
        field: String,
        other: BadType,
        //~^ ERROR UriDisplay
    },
}

fn main() {  }
