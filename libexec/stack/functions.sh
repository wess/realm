#
# functions.sh
# realm
#
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
#
# Copywrite (c) 2021 Wess.io
#

echo "Loading functions..."

mkcd() {
  mkdir -p $1 && cd $1
}

mkth() {
      for p in "$@"; do
        _dir="$(dirname -- "$p")"
        [ -d "$_dir" ] || mkdir -p -- "$_dir"
    touch -- "$p"
    done
}

mkcode() {
  mkdir -p $1 && code $1
}

updateall() {
  brew update && brew upgrade && flutter upgrade
}


die() {
  target=$1
  shift

  case $target in
  "docker")
    echo "Killing all docker containers..."
    docker stop $(docker ps -q)
  ;;
  *)
    echo "Invalid target: $target"
  ;;
  esac
}

ghc() {
  repo=$1
  shift

  local url="git@github.com:$repo.git"

  IFS='/'
  local addr=($(echo $repo | tr -s "/"))
  local proj="${addr[@]: -1}"
  local cwd="$(pwd)/$proj"

  echo "Cloning $url to $cwd..."

  git clone $url $cwd
}

# Custom 'use' command for direnv
use_asdf() {
  local tool_version
  for tool_version in $(cat .tool-versions); do
    local tool_name
    tool_name=$(echo $tool_version | awk '{print $1}')
    local version
    version=$(echo $tool_version | awk '{print $2}')
    asdf local $tool_name $version
  done
}

db_env() {
    local db_name="${1:-example_db}"

    cat <<EOL > .env
SERVER_PORT=3000

DB_HOST=localhost
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
DB_NAME=$db_name

DATABASE_URL="postgresql://\${DB_USER}:\${DB_PASSWORD}@\${DB_HOST}:\${DB_PORT}/\${DB_NAME}?schema=public"
EOL
    echo ".env file created successfully with DB_NAME=${db_name}."

    cat "dotenv_if_exists .env" > .envrc

    echo ".envrc file created successfully."
}

#!/bin/bash

# Function to create directories and files
mkall() {
  local base_path=""
  local first_file="$1"
  shift # Remove the first argument for further processing

  # Expand ~ in the first file path
  first_file=$(eval echo "$first_file")

  # Determine the base path from the first file
  if [[ "$first_file" == */* ]]; then
    base_path="${first_file%/*}" # Extract everything before the last slash
  else
    base_path="." # Use the current directory if no path is provided
  fi

  # Ensure the base directory exists
  mkdir -p "$base_path"

  # Create the first file
  touch "$first_file"
  echo "Created: $first_file"

  # Create the remaining files
  for file in "$@"; do
    touch "$base_path/$file"
    echo "Created: $base_path/$file"
  done
}
