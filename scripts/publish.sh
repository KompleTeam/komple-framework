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

PUBLISH_COMMAND="cargo publish --dry-run"

STANDALONE_PACKAGES="types"
PACKAGES="utils"

STANDALONE_MODULES="fee hub metadata permission whitelist"
MODULES="token mint marketplace merge"

STANDALONE_PERMISSIONS="link ownership"
PERMISSIONS="attribute"

SLEEP_TIME=30

for pack in $STANDALONE_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Waiting for publishing standalone packages"
sleep $SLEEP_TIME

for pack in $PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Waiting for publishing packages"
sleep $SLEEP_TIME

for cont in $STANDALONE_MODULES; do
  (
    cd "contracts/modules/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Waiting for publishing standalone modules"
sleep $SLEEP_TIME

for cont in $MODULES; do
  (
    cd "contracts/modules/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Waiting for publishing modules"
sleep $SLEEP_TIME

for cont in $STANDALONE_PERMISSIONS; do
  (
    cd "contracts/permissions/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Waiting for publishing standalone permissions"
sleep $SLEEP_TIME

for cont in $STANDALONE_PERMISSIONS; do
  (
    cd "contracts/permissions/$cont"
    echo "Publishing $cont"
    eval "$PUBLISH_COMMAND"
  )
done

echo "Everything is published!"