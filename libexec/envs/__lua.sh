#
# __default.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 01/13/2022
# 
# Copywrite (c) 2022 Wess.io
#

#!/usr/bin/env bash
#
# __rust.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#

source "${SCRIPT_ENVS_DIR}/__direnv.sh"

define LUA_IGNORE <<EOF

### Lua ### Generated by Realm
# Compiled Lua sources
luac.out

# luarocks build files
*.src.rock
*.zip
*.tar.gz

# Object files
*.o
*.os
*.ko
*.obj
*.elf

# Precompiled Headers
*.gch
*.pch

# Libraries
*.lib
*.a
*.la
*.lo
*.def
*.exp

# Shared objects (inc. Windows DLLs)
*.dll
*.so
*.so.*
*.dylib

# Executables
*.exe
*.out
*.app
*.i*86
*.x86_64
*.hex
EOF

GIT_IGNORE="$(pwd)/.gitignore"

echo "Setting up for Lua development..." | status

if [ ! -f $GIT_IGNORE ]; then
  source ${SCRIPT_ENVS_DIR}/__gitignore.sh
fi

echo "$LUA_IGNORE" >> .gitignore