#!/usr/bin/env bash
set -e

# Brings in _ROOT, _DIR, _DIRS globals.
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
source "${SCRIPT_DIR}/config.sh"

# Add Cargo to PATH.
export PATH=${HOME}/.cargo/bin:${PATH}
export CARGO_INCREMENTAL=0
CARGO="cargo"

# Checks that the versions for Cargo projects $@ all match
function check_versions_match() {
  local last_version=""
  for dir in "${@}"; do
    local cargo_toml="${dir}/Cargo.toml"
    if ! [ -f "${cargo_toml}" ]; then
      echo "Cargo configuration file '${cargo_toml}' does not exist."
      exit 1
    fi

    local version=$(grep version "${cargo_toml}" | head -n 1 | cut -d' ' -f3)
    if [ -z "${last_version}" ]; then
      last_version="${version}"
    elif ! [ "${version}" = "${last_version}" ]; then
      echo "Versions differ in '${cargo_toml}'. ${version} != ${last_version}"
      exit 1
    fi
  done
}

# Ensures there are no tabs in any file.
function ensure_tab_free() {
  local tab=$(printf '\t')
  local matches=$(git grep -E -I "${tab}" "${PROJECT_ROOT}" | grep -v 'LICENSE')
  if ! [ -z "${matches}" ]; then
    echo "Tab characters were found in the following:"
    echo "${matches}"
    exit 1
  fi
}

# Ensures there are no files with trailing whitespace.
function ensure_trailing_whitespace_free() {
  local matches=$(git grep -E -I "\s+$" "${PROJECT_ROOT}" | grep -v -F '.stderr:')
  if ! [ -z "${matches}" ]; then
    echo "Trailing whitespace was found in the following:"
    echo "${matches}"
    exit 1
  fi
}

function test_contrib() {
  FEATURES=(
    json
    msgpack
    tera_templates
    handlebars_templates
    serve
    helmet
    diesel_postgres_pool
    diesel_sqlite_pool
    diesel_mysql_pool
    postgres_pool
    mysql_pool
    sqlite_pool
    memcache_pool
    brotli_compression
    gzip_compression
  )

  echo ":: Building and testing contrib [default]..."

  pushd "${CONTRIB_LIB_ROOT}" > /dev/null 2>&1

    $CARGO test $@

    for feature in "${FEATURES[@]}"; do
      echo ":: Building and testing contrib [${feature}]..."
      $CARGO test --no-default-features --features "${feature}" $@
    done

  popd > /dev/null 2>&1
}

function test_core() {
  FEATURES=(
    secrets
    tls
  )

  pushd "${CORE_LIB_ROOT}" > /dev/null 2>&1

    echo ":: Building and testing core [no features]..."
    $CARGO test --no-default-features $@

    for feature in "${FEATURES[@]}"; do
      echo ":: Building and testing core [${feature}]..."
      $CARGO test --no-default-features --features "${feature}" $@
    done

  popd > /dev/null 2>&1
}

function test_examples() {
  for dir in $(find "${EXAMPLES_DIR}" -maxdepth 1 -mindepth 1 -type d); do
    echo ":: Building and testing example [${dir#"${EXAMPLES_DIR}/"}]..."

    pushd "${dir}" > /dev/null 2>&1
      $CARGO test $@
    popd > /dev/null 2>&1
  done
}

function test_guide() {
  echo ":: Building and testing guide..."

  pushd "${GUIDE_TESTS_ROOT}" > /dev/null 2>&1
    $CARGO test $@
  popd > /dev/null 2>&1
}

function test_default() {
  for project in "${ALL_PROJECT_DIRS[@]}"; do
    echo ":: Building and testing ${project#"${PROJECT_ROOT}/"}..."

    pushd "${project}" > /dev/null 2>&1
      $CARGO test --all-features $@
    popd > /dev/null 2>&1
  done
}

if [[ $1 == +* ]]; then
    CARGO="$CARGO $1"
    shift
fi

# The kind of test we'll be running.
TEST_KIND="default"
KINDS=("contrib" "core" "examples" "guide" "all")

if [[ " ${KINDS[@]} " =~ " ${1#"--"} " ]]; then
    TEST_KIND=${1#"--"}
    shift
fi

echo ":: Preparing. Environment is..."
print_environment
echo "  CARGO: $CARGO"
echo "  EXTRA FLAGS: $@"

echo ":: Ensuring all crate versions match..."
check_versions_match "${ALL_PROJECT_DIRS[@]}"

echo ":: Checking for tabs..."
ensure_tab_free

echo ":: Checking for trailing whitespace..."
ensure_trailing_whitespace_free

echo ":: Updating dependencies..."
if ! $CARGO update ; then
  echo "   WARNING: Update failed! Proceeding with possibly outdated deps..."
fi

case $TEST_KIND in
  contrib) test_contrib $@ ;;
  core) test_core $@ ;;
  examples) test_examples $@ ;;
  guide) test_guide $@ ;;
  default)
    test_examples $@ & examples=$!
    test_default $@ & default=$!
    test_guide $@ & guide=$!

    wait $examples && wait $default && wait $guide
    ;;
  all)
    test_core $@ & core=$!
    test_contrib $@ & contrib=$!
    test_examples $@ & examples=$!
    test_default $@ & default=$!
    test_guide $@ & guide=$!

    wait $core && wait $contrib && wait $examples && wait $default && wait $guide
    ;;
esac
