# Rocket Todo Example

This example makes use of a SQLite database via `diesel` to store todo tasks. As
a result, you'll need to have `sqlite3` and its headers installed:

  * **OS X:** `brew install sqlite`
  * **Debian/Ubuntu:** `apt-get install libsqlite3-dev`
  * **Arch:** `pacman -S sqlite`

## Running

**Before running, building, or testing this example, you'll need to ensure that
a SQLite database file with the proper schema is present.**

On a Unix machine or with bash installed, you can simply run the `boostrap.sh`
script to create the database. The script installs the `diesel_cli` tools if
they're not already installed and runs the migrations.

You can also install the Diesel CLI and run the migrations manually with the
following commands:

```sh
# install Diesel CLI tools
cargo install diesel_cli --no-default-features --features sqlite

# create db/db.sql
diesel migration run --database-url="db/db.sql"
```
