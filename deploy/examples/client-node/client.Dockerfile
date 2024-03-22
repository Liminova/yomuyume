FROM node:21-alpine AS install

# Prepare
RUN apk add --no-cache git
RUN git clone --depth 1 https://github.com/Liminova/yomuyume-client.git /temp/yomuyume-client

# Build
WORKDIR /temp/yomuyume-client
RUN npm install -g pnpm
RUN pnpm install
RUN pnpm run build

# New stage for smaller image
FROM node:21-alpine as production
WORKDIR /usr/src/app
COPY --from=install /temp/yomuyume-client/.output /usr/src/app/public
EXPOSE 3000/tcp
ENTRYPOINT [ "node", "public/server/index.mjs" ]