#!/bin/bash
set -e

#
# Builds the rustdocs for all of the libraries.
#

# Brings in: PROJECT_ROOT, EXAMPLES_DIR, LIB_DIR, CODEGEN_DIR, CONTRIB_DIR, DOC_DIR
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "${SCRIPT_DIR}/config.sh"

function mk_doc() {
  local dir=$1
  pushd "${dir}" > /dev/null 2>&1
    echo ":: Documenting '${dir}'..."
    cargo doc --no-deps --all-features
  popd > /dev/null 2>&1
}

# We need to clean-up beforehand so we don't get all of the dependencies.
echo ":::: Cleaning up before documenting..."
cargo clean
cargo update

# Generate the rustdocs for all of the crates.
for dir in "${ALL_PROJECT_DIRS[@]}"; do
  mk_doc "${dir}"
done

# Blank index, for redirection.
touch "${DOC_DIR}/index.html"
