#!/usr/bin/env bash
#
# __go.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#

source "${SCRIPT_ENVS_DIR}/__direnv.sh"

define GO_IGNORE <<EOF

### GO ###
# If you prefer the allow list template instead of the deny list, see community template:
# https://github.com/github/gitignore/blob/main/community/Golang/Go.AllowList.gitignore
#
# Binaries for programs and plugins
*.exe
*.exe~
*.dll
*.so
*.dylib

# Test binary, built with `go test -c`
*.test

# Output of the go coverage tool, specifically when used with LiteIDE
*.out

# Dependency directories (remove the comment below to include it)
# vendor/

# Go workspace file
go.work

### Go Patch ###
/vendor/
/Godeps/

# End of https://www.toptal.com/developers/gitignore/api/go

EOF

GIT_IGNORE="$(pwd)/.gitignore"

echo "Setting up for Go development..." | status

if [ ! -f $GIT_IGNORE ]; then
  source ${SCRIPT_ENVS_DIR}/__gitignore.sh
fi

echo "$GO_IGNORE" >> .gitignore

PROJECT_NAME=${PWD##*/}
MAIN=main.go

echo "Creating new Go project..." | status

go mod init "$PROJECT_NAME"

echo 'export PATH=$(PWD)/bin:$PATH' >> .envrc
echo 'package main' >> $MAIN 
echo 'import "fmt"' >> $MAIN
echo 'func main() {' >> $MAIN
echo '    fmt.Println("hello world")' >> $MAIN 
echo '}' >> $MAIN
