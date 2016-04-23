#!/bin/bash
set -e

EXAMPLES_DIR="examples/"
LIB_DIR="lib/"
MACROS_DIR="macros/"

function build_and_test() {
  local dir=$1
  if [ -z "${dir}" ] || ! [ -d "${dir}" ]; then
    echo "Tried to build and test inside '${dir}', but it is an invalid path."
    exit 1
  fi

  pushd ${dir}
  echo ":: Building '${PWD}'..."
  cargo clean
  cargo build --verbose

  echo ":: Running unit tests in '${PWD}'..."
  cargo test --verbose
  popd
}

build_and_test $LIB_DIR
build_and_test $MACROS_DIR

for file in ${EXAMPLES_DIR}/*; do
  echo "${file}"
  if [ -d "${file}" ]; then
    build_and_test "${file}"
  fi
done
