# Simply sets up a few useful variables.

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

function relative() {
  local full_path="${SCRIPT_DIR}/../${1}"

  # Use readlink as a fallback to readpath for cross-platform compat.
  if ! command -v realpath >/dev/null 2>&1; then
    echo $(readlink -f "${full_path}")
  else
    echo $(realpath "${full_path}")
  fi
}

EXAMPLES_DIR=$(relative "examples")
LIB_DIR=$(relative "lib")
CODEGEN_DIR=$(relative "codegen")
CONTRIB_DIR=$(relative "contrib")
DOC_DIR=$(relative "target/doc")

if [ "${1}" = "-p" ]; then
  echo $SCRIPT_DIR
  echo $EXAMPLES_DIR
  echo $LIB_DIR
  echo $CODEGEN_DIR
  echo $CONTRIB_DIR
  echo $DOC_DIR
fi
