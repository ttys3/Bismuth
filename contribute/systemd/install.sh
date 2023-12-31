#!/usr/bin/env bash

set -eou pipefail

cp -v ./bismuth.* ~/.config/systemd/user/

systemctl --user daemon-reload

systemctl --user stop bismuth.timer

systemctl --user enable --now bismuth.timer

systemctl --user start bismuth.timer

systemctl --user status bismuth.timer --no-pager

systemctl --user list-timers --no-pager
