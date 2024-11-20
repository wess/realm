#!/usr/bin/env bash
#
# __config.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#

CUR_DIR="$(pwd)"

DEFAULT_EMAIL=$(cat ${HOME}/.gitconfig | grep -i -o '[a-zA-Z0-9._%+-]\+@[a-zA-Z0-9.-]\+\.[a-zA-Z]\{2,4\}')
DEFAULT_NAME="$USER"
DEFAULT_COMPANY="$USER Incorporated"

if [[ -z "${REALM_AUTHOR}" ]]; then
  read -p "Enter your name [${DEFAULT_NAME}]: " USER_NAME

  if [ -z "$USER_NAME" ]; then
    USER_NAME=${DEFAULT_NAME}
  fi

  export REALM_AUTHOR="$USER_NAME"
fi

if [[ -z "${REALM_AUTHOR_EMAIL}" ]]; then
  read -p "Enter your email [${DEFAULT_EMAIL}]: " USER_EMAIL

  if [ -z "$USER_EMAIL" ]; then
    USER_EMAIL=${DEFAULT_EMAIL}
  fi

  export REALM_AUTHOR_EMAIL="$USER_EMAIL"
fi

if [[ -z "${REALM_COMPANY}" ]]; then
  read -p "Enter your company [${DEFAULT_COMPANY}]: " USER_COMPANY

  if [ -z "$USER_COMPANY" ]; then
    USER_COMPANY=${DEFAULT_COMPANY}
  fi

  export REALM_COMPANY="$USER_COMPANY"
fi

define REALM_VARS <<EOF
export REALM_AUTHOR="$USER_NAME"
export REALM_NAME="$USER"
export REALM_EMAIL="$USER_EMAIL"
export REALM_COMPANY="$USER_COMPANY"
export REALM_EDITOR="vim"
EOF

if [[ ! -f "$REALM_CONFIG" ]]; then
  mkdir -p "$(dirname "$REALM_CONFIG")" && touch "$REALM_CONFIG"
  echo "$REALM_VARS" > $REALM_CONFIG
fi



### CLEAN UP
unset CUR_DIR
unset DEFAULT_EMAIL
unset DEFAULT_NAME
unset DEFAULT_COMPANY
unset USER_NAME
unset USER_EMAIL
unset USER_COMPANY