#! /usr/bin/env bash

SCRIPTPATH=$(cd "$(dirname "$0")" ; pwd -P)
DATABASE_URL=${SCRIPTPATH}/db/db.sql 

pushd $SCRIPTPATH
  # install the diesel CLI tools
  cargo install diesel_cli

  # create db/db.sql
  diesel migration --database-url=$DATABASE_URL run
popd $SCRIPTPATH
