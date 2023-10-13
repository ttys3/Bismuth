#!/usr/bin/env bash

set -eou pipefail

systemd-run --user --on-active="10s" --unit="bismuth.service"


