ARM_VERSION="0.12.8"
INTEL_VERSION="0.12.9"

function optimize_arm() {
  docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer-arm64:$ARM_VERSION
}

function optimize_intel() {
  docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:$INTEL_VERSION
}

while getopts 'ia' OPTION; do
  case "$OPTION" in
    a)
      echo "Using arm64 optimizer with version $ARM_VERSION"
      optimize_arm
      ;;
    i)
      echo "Using intel optimizer with version $INTEL_VERSION"
      optimize_intel
      ;;
    ?)
      echo "script usage: $(basename \$0) [-i] [-a]" >&2
      exit 1
      ;;
  esac
done
