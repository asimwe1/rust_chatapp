#!/bin/bash
set -e

#
# Builds the rustdocs for all of the libraries.
#

# Brings in: PROJECT_ROOT, EXAMPLES_DIR, LIB_DIR, CODEGEN_DIR, CONTRIB_DIR, DOC_DIR
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "${SCRIPT_DIR}/config.sh"

# We need to clean-up beforehand so we don't get all of the dependencies.
echo ":::: Cleaning up before documenting..."
cargo clean
cargo update

# Generate the rustdocs for all of the crates.
echo ":::: Generating the docs..."
pushd "${PROJECT_ROOT}" > /dev/null 2>&1
RUSTDOCFLAGS="-Z unstable-options --crate-version ${ROCKET_VERSION}" \
  cargo doc -p rocket -p rocket_contrib -p rocket_codegen_next --no-deps --all-features
popd > /dev/null 2>&1

# Blank index, for redirection.
touch "${DOC_DIR}/index.html"
