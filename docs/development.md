# Development

## Recommendation

-   [VSCode](https://code.visualstudio.com/)
-   [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

## Reverse proxy / CORS

Since we rely on `HttpOnly` cookies for auth, the backend APIs and frontend SPA must be served under the same origin (domain + port) and over HTTPS on the browser.

In production, this is a non-issue since the backend serves both. In development they're 2 separate servers, thus relying on a reverse proxy to re-route them to be under the same origin.

Install or get the [latest caddy](https://github.com/caddyserver/caddy/releases/latest) binary and run `caddy run -c Caddyfile` **outside** the devcontainer for Caddy to install its certificates.

## Database

It's highly recommended to use `sqlx::query!` and `sqlx::query_as!` macros exclusively to interact with the database since it

-   uses prepared statements under the hood, avoids SQL injections,
-   verifies SQL syntactic and semantics at compilation, reducing runtime errors (though not all).
