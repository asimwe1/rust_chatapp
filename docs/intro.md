Welcome to Rocket! This is the official guide. It is designed to serve as a
starting point as a well as reference. This guide is conversational. For concise
and purely technical documentation, see the [API documentation](/API).

-------------------------------------------------------------------------------

# IDEA

HAVE THIS GUIDE SERVE AS A REFERENCE AS WELL BY MARKING PIECES OF THIS GUIDE AS
'GUIDE ONLY' AND OMITTING THOSE SECTIONS WHEN SOMEONE WANTS A REFERENCE ONLY. IT
WOULD BE NEAT IF THERE WAS A LITTLE JAVASCRIPT BUTTON THAT JUST HID THE GUIDE
PARTS.

# Guide

Hello! By now you've gleaned that Rocket is a web framework for Rust. You also
know that it aims to be fast, easy, and flexible. It also aims to be _fun_, and
it accomplishes this by ensuring that you write as little code as needed to
accomplish your task. This guide is meant to introduce you to the core,
intermediate, and advanced concepts of Rocket. After reading this guide, you
should find yourself being _very_ productive with Rocket.

## Audience

Readers of this guide are assumed to have a good grasp of the Rust programming
language. Readers new to Rust are encourage to read through the [Rust
Book](https://doc.rust-lang.org/book/). This guide also assumes a basic
understanding of web application fundamentals and HTTP.

# Foreword

Rocket's philosophy is that a function declaration should contain all of the
necessary information to process a request. This immediately prohibits APIs
where request state is retrieved from a global context. As a result of the
locality of information, request handling is _self contained_ in Rocket:
handlers are regular functions that can be called by other code.

Rocket also believes that all request handling information should be _typed_.
Because the web and HTTP are themselves untyped (or _stringly_ typed, as some
call it), this means that something or someone has to convert strings to native
types. Rocket does this for you with zero programming overhead.

These two core philosophies dictate Rocket's interface, and you will find the
ideas embedded in Rocket's core features. But, enough with the boring diatribe.
Let's get to know Rocket.

# Quick Start

The absolute fastest way to start experimenting with Rocket is to clone the
Rocket repository and run the included examples. For instance, the following set
of commands runs the `hello_world` example:

```sh
git clone https://github.com/SergioBenitez/rocket
cd rocket/examples/hello_world
cargo run
```

There are numerous examples in `rocket/examples`, all of which can be run with
Cargo by using `cargo run`. Note that Rocket requires the latest Rust nightly.

# Getting Started

Let's create and run our first Rocket application. We'll ensure we have a
compatible version of Rust, create a new Cargo project that uses Rocket, and
then run the project.

## Rust

Rocket makes heavy use of Rust's syntax extensions. Because syntax extension
don't yet have a stable compiler API, we'll need to use a nightly version of
Rust with Rocket. If you already have a working installation of the latest Rust
nightly, feel free to skip this section.

To install a nightly version of Rust, we recommend using `rustup`. Install
`rustup` by following the instructions on [their website](https://rustup.rs/).
Once `rustup` is installed, configure Rust nightly as your default toolchain by
running the command:

```sh
rustup default nightly
```

If you prefer, once we setup a project directory in the following section, you
can use per-directory defaults to use the nightly version _only_ for your Rocket
project by running the following command in the directory:

```sh
rustup override set nightly
```

Rocket requires the latest version of Rust nightly. If your Rocket application
suddently stops building, ensure you're using the latest version of Rust by
updating:

```sh
rustup update
```

## Creating a Rocket Project

Start by creating a new binary-based Cargo project and changing into the new
directory:

```sh
cargo new hello-rocket --bin
cd hello-rocket
```

Now, add Rocket and its code generation facilities to your project by ensuring
your `Cargo.toml` file contains the following dependencies:

```
[dependencies]
rocket = "*"
rocket_codegen = "*"
```

Build the project now to ensure your Rust version is compatible with the latest
Rocket version:

```sh
cargo build
```

Modify `src/main.rs` so that it contains the code for the Rocket `Hello, world!`
program, which we reproduce below:

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/hello", routes![hello]).launch();
}
```

Run the program by using `cargo run`. You should see the following in your
terminal:

```sh
🔧  Configured for development.
    => listening: localhost:8000
    => logging: Normal
    => session key: false
🛰  Mounting '/hello':
    => GET /hello/
🚀  Rocket has launched from localhost:8000...
```

Finally, visit `http://localhost:8000` to see your first Rocket application in
action.

# Introduction

Rocket provides a focused number of core primitives to build web servers and
applications with Rust: the rest is up to you. In short, Rocket provides
routing, pre-processing of requests, and post-processing of responses. Your
application code fills the gap between pre-processing and post-processing.

Rocket _does not_ force decisions on you. Templates, serialization, sessions,
and just about everything else are all pluggable, optional components. While
Rocket has official support and libraries for each of these, they are completely
optional to use, and writing your own versions of these is not only possible,
but straightforward. These components feel like first-class citizens.

If you'd like, you can think of Rocket as being a more flexible, friendly medley
of [Rails](rubyonrails.org), [Flask](http://flask.pocoo.org/),
[Bottle](http://bottlepy.org/docs/dev/index.html), and
[Yesod](http://www.yesodweb.com/), except without all of the bloat. We prefer to
think of Rocket as something new.

# From Request to Response

This section of the guide provides a grand overview of Rocket by examining a
simple application. Let's begin.

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("hi/<name>/<age>")]
fn hello(name: &str, age: i8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("bye/<name>")]
fn goodbye(name: &str) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[error(404)]
fn not_found() -> &'static str {
    "Sorry, I don't know what you're looking for."
}

fn main() {
    rocket::ignite()
        .mount("/", routes![hello, goodbye])
        .catch(errors![not_found])
        .launch();
}
```

# Routing

The task of a web application is to handle incoming requests by returning an
appropriate response. The code that handles the request is called a _request
handler_. Requests are made to specific paths, URIs, and with a specific intent,
declared via an HTTP method, in mind. The code that determines which request
handler should be invoked for a given request is called a _request router_, or
just _router_.

In Rocket, request handlers are regular functions, and you tell Rocket's router
which requests are intended for a given handler through an annotation, or
attribute, on that function. We call the combination of a handler and its
attribute a _route_.

The code below is one of the simplest routes we can write in Rocket:

```rust
#[get("/hello")]
fn hello() -> &'static str {
    "Hello, world!"
}
```

As you might expect, the code declares that the `hello` function is the handler
for any `GET` requests to the `/hello` path, and that the response to the
request is of type `&'static str` whose value is `Hello, world!`.

Rocket route attributes have the following grammar:

```ebnf
route := METHOD '(' path, kv_param* ')'

path := PATH
      | 'path' = PATH

kv_param := 'rank' = INTEGER
          | 'form' = STRING
          | 'format' = STRING
```
