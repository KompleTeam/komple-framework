#!/usr/bin/env bash

set -e
set -o pipefail

for c in contracts/*; do
    for sc in $c/*; do
        echo "Generating schema for $sc"
        cd $sc
        cargo schema
        cd -
    done
done
