Rocket Todo Example
===================

Before running this example, you'll need to ensure there's a database file
present. You can do this with Diesel.

Running migration with Diesel
-----------------------------

Just run the following commands in your shell:

```
cargo install diesel_cli # installs the diesel CLI tools
DATABASE_URL=db/db.sql diesel migration run # create db/db.sql
```

