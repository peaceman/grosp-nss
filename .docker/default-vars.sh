#!/usr/bin/env bash

DOCKER_REPO="${DOCKER_REPO:-peaceman/grosp-nss}"

source "${BASH_SOURCE%/*}/determine-target-tag.sh"
