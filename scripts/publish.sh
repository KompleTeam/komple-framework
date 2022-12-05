#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

PUBLISH_COMMAND="cargo publish"

PACKAGES="types utils"
MODULES="fee hub metadata permission whitelist token mint marketplace merge"
PERMISSIONS="link ownership attribute"

SLEEP_TIME=30

echo "Starting packages publishing..."

for pack in $PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    eval "$PUBLISH_COMMAND"
    sleep $SLEEP_TIME
  )
done

echo "Starting modules publishing..."

for cont in $MODULES; do
  (
    cd "contracts/modules/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
    sleep $SLEEP_TIME
  )
done

echo "Starting modules publishing..."

for cont in $PERMISSIONS; do
  (
    cd "contracts/permissions/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
    sleep $SLEEP_TIME
  )
done

echo "Everything is published!"
