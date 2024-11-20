#!/usr/bin/env bash
#
# __direnv.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/28/2021
# 
# Copywrite (c) 2021 Wess.io
#

ENV_FILE="$(pwd)/.envrc"

define ENV_CONTENT <<EOF
######
## When values in here are changed
## Rerun 'direnv allow'
######

#### APP SPECIFICS
# ENV Vars or other app specific items.


#### GENERAGED BY REALM
PATH_add $REALM_BIN
EOF

if [[ ! -f "$ENV_FILE" ]]; then
  touch "$ENV_FILE"
fi

echo "$ENV_CONTENT" > "$ENV_FILE"
