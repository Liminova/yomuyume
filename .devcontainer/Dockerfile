FROM node:22.9-bookworm-slim

RUN apt-get update && apt-get install -y curl zsh && apt-get clean

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y