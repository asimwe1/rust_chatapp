#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::http::RawStr;
use rocket::http::uri::UriDisplay;

macro_rules! assert_uri_display {
    ($v:expr, $s:expr) => (
        let uri_string = format!("{}", &$v as &UriDisplay);
        assert_eq!(uri_string, $s);
    )
}

#[derive(UriDisplay, Clone)]
enum Foo<'r> {
    First(&'r RawStr),
    Second {
        inner: &'r RawStr,
        other: usize,
    },
    Third {
        #[form(field = "type")]
        kind: String,
    },
}

#[test]
fn uri_display_foo() {
    let foo = Foo::First("hello".into());
    assert_uri_display!(foo, "hello");

    let foo = Foo::First("hello there".into());
    assert_uri_display!(foo, "hello%20there");

    let foo = Foo::Second { inner: "hi".into(), other: 123 };
    assert_uri_display!(foo, "inner=hi&other=123");

    let foo = Foo::Second { inner: "hi bo".into(), other: 321 };
    assert_uri_display!(foo, "inner=hi%20bo&other=321");

    let foo = Foo::Third { kind: "hello".into() };
    assert_uri_display!(foo, "type=hello");

    let foo = Foo::Third { kind: "hello there".into() };
    assert_uri_display!(foo, "type=hello%20there");
}

#[derive(UriDisplay)]
struct Bar<'a> {
    foo: Foo<'a>,
    baz: String,
}

#[test]
fn uri_display_bar() {
    let foo = Foo::First("hello".into());
    let bar = Bar { foo, baz: "well, hi!".into() };
    assert_uri_display!(bar, "foo=hello&baz=well,%20hi!");

    let foo = Foo::Second { inner: "hi".into(), other: 123 };
    let bar = Bar { foo, baz: "done".into() };
    assert_uri_display!(bar, "foo.inner=hi&foo.other=123&baz=done");

    let foo = Foo::Third { kind: "hello".into() };
    let bar = Bar { foo, baz: "turkey day".into() };
    assert_uri_display!(bar, "foo.type=hello&baz=turkey%20day");
}

#[derive(UriDisplay)]
struct Baz<'a> {
    foo: Foo<'a>,
    bar: Bar<'a>,
    last: String
}

#[test]
fn uri_display_baz() {
    let foo1 = Foo::Second { inner: "hi".into(), other: 123 };
    let foo2 = Foo::Second { inner: "bye".into(), other: 321 };
    let bar = Bar { foo: foo2, baz: "done".into() };
    let baz = Baz { foo: foo1, bar, last: "ok".into() };
    assert_uri_display!(baz, "foo.inner=hi&foo.other=123&\
                              bar.foo.inner=bye&bar.foo.other=321&bar.baz=done&\
                              last=ok");

    let foo1 = Foo::Third { kind: "hello".into() };
    let foo2 = Foo::First("bye".into());
    let bar = Bar { foo: foo1, baz: "end".into() };
    let baz = Baz { foo: foo2, bar, last: "done".into() };
    assert_uri_display!(baz, "foo=bye&\
                              bar.foo.type=hello&bar.baz=end&\
                              last=done");
}
