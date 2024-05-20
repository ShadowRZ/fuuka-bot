FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye
RUN sed -i 's|deb.debian.org|mirrors.bfsu.edu.cn|g' /etc/apt/sources.list
# Add OpenSSL / SQLite development packages
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev
