# Version 0.0.10 (Oct 03, 2016)

## Breaking

  * Remove `Rocket::new` in favor of `ignite` method.
  * Remove `Rocket::mount_and_launch` in favor of chaining `mount(..).launch()`.
  * `mount` and `catch` take `Rocket` type by value.
  * All types related to HTTP have been moved into `http` module.
  * `Template::render` in `contrib` now takes context by reference.

## Core

  * Rocket now parses option `Rocket.toml` for configuration, defaulting to sane
    values.
  * `ROCKET_ENV` environment variable can be used to specify running environment.

## Docs

  * Document `ContentType`.
  * Document `Request`.
  * Add script that builds docs.

## Testing

  * Scripts can now be run from any directory.
  * Cache Cargo directories in Travis for faster testing.
  * Check that library version numbers match in testing script.

# Version 0.0.9 (Sep 29, 2016)

## Breaking

  * Rename `response::data_type` to `response::data`.

## Core

  * Rocket interprets `_method` field in forms as the incoming request's method.
  * Add `Outcome::Bad` to signify responses that failed internally.
  * Add a `NamedFile` `Responder` type that uses a file's extension for the
    response's content type.
  * Add a `Stream` `Responder` for streaming responses.

## Contrib

  * Introduce the `contrib` crate.
  * Add JSON support via `JSON`, which implements `FromRequest` and `Responder`.
  * Add templating support via `Template` which implements `Responder`.

## Docs

  * Initial guide-like documentation.
  * Add documentation, testing, and contributing sections to README.

## Testing

  * Add a significant number of codegen tests.

