echo "Loading paths..."

## Homebrew
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/opt/homebrew/sbin:$PATH"

## Rust
export PATH="${HOME}/.cargo/env:$PATH"
#export DYLD_LIBRARY_PATH=${HOME}/.rustup/toolchains/stable-x86_64-apple-darwin/lib

## Flutter
export PATH="$HOME/Development/flutter/bin:$PATH"

## Libs
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"

## Local dev
export PATH="$HOME/Development/bin:$PATH"

## Bun/ElectronBun
export PATH="$HOME/.bun/bin:$PATH"
