echo "Loading hooks..."

# Homebrew
eval "$(/opt/homebrew/bin/brew shellenv)"

# DIRENV
eval "$(direnv hook zsh)"

# RUST
source ${HOME}/.cargo/env 