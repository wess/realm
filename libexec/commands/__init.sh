#!/usr/bin/env bash
#
# __init.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#



arg=$1
shift

case $arg in
  "flutter")
    source "${SCRIPT_ENVS_DIR}/__flutter.sh"
    ;;
  "node")
    source "${SCRIPT_ENVS_DIR}/__node.sh"
    ;;
  "rust")
    source "${SCRIPT_ENVS_DIR}/__rust.sh"
    ;;
  "lua")
    source "${SCRIPT_ENVS_DIR}/__lua.sh"
    ;;
  "go")
    source "${SCRIPT_ENVS_DIR}/__go.sh"
    ;;
  "db")
    source "${SCRIPT_ENVS_DIR}/__db.sh"
    ;;
  *)
    echo "Invalid init command." | error
    ;;
esac

