#!/bin/bash
set -e

if [ ! -f runtime.env ]; then
    echo "runtime.env file not found!"
    exit 1
fi
source runtime.env

if [ -z "$MEDIA_VAULT_PATH" ]; then
    echo "MEDIA_VAULT_PATH is not set. Please set it in the runtime.env file."
    exit 1
fi

if ! cat token.txt | docker login ghcr.io -u Rationelis --password-stdin; then
    echo "Docker login failed!"
    exit 1
fi

if ! docker pull ghcr.io/rationelis/media-vault:latest; then
    echo "Failed to pull the Docker image!"
    exit 1
fi

CONTAINER_NAME="media-vault"
if [ $(docker ps -q -f name=$CONTAINER_NAME) ]; then
    echo "Stopping and removing the existing container..."
    docker stop $CONTAINER_NAME
    docker rm $CONTAINER_NAME
fi

if ! docker run -d \
    --name $CONTAINER_NAME \
    -v ${MEDIA_VAULT_PATH}/media-vault-in:/app/media-vault-in \
    -v ${MEDIA_VAULT_PATH}/media-vault-out:/app/media-vault-out \
    -v ${MEDIA_VAULT_PATH}/config:/app/config \
    ghcr.io/rationelis/media-vault:latest; then
    echo "Failed to start the Docker container!"
    exit 1
fi

echo "Container started successfully!"
