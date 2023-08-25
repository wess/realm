#!/usr/bin/env bash

# Check if yq is installed
if ! command -v yq &> /dev/null; then
    echo "'yq' is not installed. Installing..."
    pip install yq || { echo "Failed to install 'yq'. Please install it manually."; exit 1; }
fi

# Identify the current shell and user details
SHELL_NAME=$(basename $SHELL)

# Extract the name of the current directory
DIR_NAME=$(basename $(pwd))

# List of files and directories related to shell configurations
SHELL_RELATED=("$HOME/.$SHELL_NAME"rc "$HOME/.profile")

# Check for oh-my-zsh and set a flag if found
INSTALL_OH_MY_ZSH=0
if [ -d "$HOME/.oh-my-zsh" ]; then
    SHELL_RELATED+=("$HOME/.oh-my-zsh")
    INSTALL_OH_MY_ZSH=1
fi

# Serialize the current environment variables
env | sed 's/=/="/' | sed 's/$/"/' | grep -vE "^(PATH|HOME)=" > .realm/env.list

# If realm.yaml exists, extract packages from it
if [ -f realm.yaml ]; then
    PACKAGES=($(yq eval '.packages[]' realm.yaml))
else
    PACKAGES=()
fi

{
    echo "FROM alpine:latest"
    while read -r line; do
        echo "ENV $line"
    done < .realm/env.list

    echo "RUN apk update"
    echo "RUN apk add $SHELL_NAME"

    # Add packages from the YAML file
    for pkg in "${PACKAGES[@]}"; do
        echo "RUN apk add $pkg"
    done

    # If oh-my-zsh is detected, install git and zsh
    if [ $INSTALL_OH_MY_ZSH -eq 1 ]; then
        echo "RUN apk add git zsh"
        echo "RUN sh -c \"\$(wget -O- https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" --unattended"
    fi

    echo "RUN mkdir /$DIR_NAME"
    echo "WORKDIR /$DIR_NAME"
    echo "ENTRYPOINT [\"$SHELL_NAME\"]"
} > .realm/Dockerfile
