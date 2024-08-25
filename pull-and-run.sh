#!/bin/bash
source config.env

cat token.txt | docker login ghcr.io -u Rationelis --password-stdin

docker pull ghcr.io/rationelis/media-vault:latest

docker run \
    -v ${MEDIA_VAULT_PATH}/media-vault-in:/app/media-vault-in \
    -v ${MEDIA_VAULT_PATH}/media-vault-out:/app/media-vault-out \
    -v ${MEDIA_VAULT_PATH}/config:/app/config \
    ghcr.io/rationelis/media-vault:latest
