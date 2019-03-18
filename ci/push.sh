#!/usr/bin/env bash

set -e

docker build --file ci/Dockerfile --tag miindy/irro-ci:latest .
docker push miindy/irro-ci:latest
