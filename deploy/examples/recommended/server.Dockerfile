FROM rust:alpine AS install

# Add tools for building
RUN apk add --no-cache git openssl-dev g++
ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include

# Clone
RUN git clone --depth 1 https://github.com/Liminova/yomuyume.git /yomuyume
WORKDIR /yomuyume

# Build, add permissions
RUN cargo build --package yomuyume-server --release
RUN chmod +x ./target/release/yomuyume-server

# New stage for smaller image
FROM alpine:3.19
COPY --from=install /yomuyume/target/release/yomuyume-server /yomuyume-server
EXPOSE 3000/tcp

ENTRYPOINT [ "/yomuyume-server" ]