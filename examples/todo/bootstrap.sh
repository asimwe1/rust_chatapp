#! /usr/bin/env bash

SCRIPT_PATH=$(cd "$(dirname "$0")" ; pwd -P)
DATABASE_URL=${SCRIPT_PATH}/db/db.sql

pushd $SCRIPT_PATH
  # install the diesel CLI tools
  cargo install diesel_cli

  # create db/db.sql
  diesel migration --database-url=$DATABASE_URL run
popd $SCRIPT_PATH
