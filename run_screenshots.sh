#!/usr/bin/env bash
# Capture screenshots for all examples.
# Delegates to run_host.sh -s (headless, no SDL window).
set -e
exec ./run_host.sh -s
