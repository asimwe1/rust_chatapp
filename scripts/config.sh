# Simply sets up a few useful variables.

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

function relative() {
  local full_path="${SCRIPT_DIR}/../${1}"

  if [ -d "${full_path}" ]; then
    # Try to use readlink as a fallback to readpath for cross-platform compat.
    if command -v realpath >/dev/null 2>&1; then
      echo $(realpath "${full_path}")
    elif ! (readlink -f 2>&1 | grep illegal > /dev/null); then
      echo $(readlink -f "${full_path}")
    else
      echo "Rocket's scripts require 'realpath' or 'readlink -f' support." >&2
      echo "Install realpath or GNU readlink via your package manager." >&2
      echo "Aborting." >&2
      exit 1
    fi
  else
    # when the directory doesn't exist, fallback to this.
    echo "${full_path}"
  fi
}

EXAMPLES_DIR=$(relative "examples") || exit $?
LIB_DIR=$(relative "lib") || exit $?
CODEGEN_DIR=$(relative "codegen") || exit $?
CONTRIB_DIR=$(relative "contrib") || exit $?
DOC_DIR=$(relative "target/doc") || exit $?

if [ "${1}" = "-p" ]; then
  echo $SCRIPT_DIR
  echo $EXAMPLES_DIR
  echo $LIB_DIR
  echo $CODEGEN_DIR
  echo $CONTRIB_DIR
  echo $DOC_DIR
fi
