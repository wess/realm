#!/usr/bin/env bash
#
# __envrc.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#

define ENV_RC <<EOF
######
## When values in here are changed
## Rerun 'direnv allow'
######
#### GENERAGED BY REALM
PATH_add $SCRIPT_ROOT
PROJECT_LAYOUT
#### APP SPECIFICS
EOF

OUT_DIR="$(pwd)/.envrc"

if [ ! -f $OUT_DIR ]; then
  mkdir -p "$(dirname "$OUT_DIR")" && touch "$OUT_DIR"
  
  echo "$ENV_RC" >> $OUT_DIR
fi


unset ENV_RC
unset OUT_DIR