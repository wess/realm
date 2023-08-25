#!/usr/bin/env bash

 # Check for Dockerfile in .realm
if [ ! -f .realm/Dockerfile ]; then
    echo "Dockerfile not found in .realm. Run init first."
    exit 1
fi

# Check if the Docker image already exists
IMAGE_NAME="myalpine"
if docker image inspect $IMAGE_NAME > /dev/null 2>&1; then
    echo "Image $IMAGE_NAME already exists. Deleting..."
    docker image rm $IMAGE_NAME
fi

# Build the Docker image
docker build -t $IMAGE_NAME -f .realm/Dockerfile .

# Extract the name of the current directory
DIR_NAME=$(basename $(pwd))

# Run the Docker container interactively with proper volume mapping
docker run -it --rm -v $(pwd):/$DIR_NAME -w /$DIR_NAME $IMAGE_NAME