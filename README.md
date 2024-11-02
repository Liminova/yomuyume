# yomuyume

<div align="center">
  <img src="src/public/favicon/android-chrome-192x192.png" alt="yomuyume logo" width="192" height="192">

  Self-hosted media server for manga and comics, written in Rust.
</div>

## highlights

- ⚡written in Rust for blazingly fast performance,
- 🗃️ using `7zip` under-the-hood, support all the archive formats,
- 📄 manage metadata using (a modified version of) the latest version of [The Anansi Project's `ComicInfo.xml` schema](https://anansi-project.github.io/docs/comicinfo/intro),
- 🌫️ [blurha.sh](https://blurha.sh) as placeholder for loading images,

and many more.

## deploy
WIP

## develop
### 1. environment
highly recommended to use vscode + devcontainer.

### 2. non-rust-or-js dependencies
- [dav1d](https://code.videolan.org/videolan/dav1d) for the [image](https://crates.io/crates/image) crate to decode `avif` images and encode in blurhash.
- [7zz](https://www.7-zip.org/download.html) CLI for interacting with archive files.

automatically handled by the `.devcontainer/postinstall.sh` script.

### 3. architectures

<details>
  <summary>development</summary>

```mermaid
flowchart TD
  caddy["`**caddy server**<br>_apply cert, rewrite headers_`"]
  server["`**rust server**<br>_serve the APIs_`"]
  client["`**nitro server**<br>_serve the nuxt client_`"]
  db["`**postgres database**`"]

  browser <-- **localhost:8080** <br> _APIs w/ HTTPS_ --> caddy
  browser <-- **localhost:3001** <br> _web client w/ HTTPS_ --> caddy

  caddy <-- localhost:8081 --> server
  caddy <-- localhost:3000 --> client
  server <-- localhost:5432 --> db

  subgraph db_container["container"]
    db
  end

  subgraph devcontainer
    server
    client
    db_container
  end

  subgraph host_subgraph["host OS"]
    browser
    caddy
    devcontainer
  end

  style host_subgraph fill:transparent
  style devcontainer fill:transparent
```
- why Caddy in the middle? search the internet for `cookies samesite=none secure`.
- if you're using wsl, run caddy **on windows, not inside wsl** for it to install the cert into the windows certificate store.
- the command: `caddy run --config Caddyfile`
</details>

<details>
  <summary>production</summary>

```mermaid
flowchart LR
  server["`**rust server**<br>_serve both the APIs<br>and the nuxt client_`"]
  db["`**postgres<br>database**`"]
  reverse_proxy["`**reverse proxy<br>_Caddy, NGINX, ..._**`"]

  server <-- postgres:5432 --> db
  server <-- yomuyume:8080 --> reverse_proxy
  reverse_proxy <-- example.com<br>_with TLS_ --> browser
  server <-- server_ip:8080 --> browser

  subgraph client_os["client OS"]
    browser
  end
  subgraph db_container["container"]
    db
  end
  subgraph server_container["container"]
    server
  end
  subgraph rp_container["container"]
    reverse_proxy
  end

  subgraph docker
    server_container
    db_container
    rp_container
  end

  subgraph server_os["server OS"]
    reverse_proxy
    docker
  end

  style server_os fill:transparent
  style docker fill:transparent
  style client_os fill:transparent
```
- freely choose where to put the reverse proxy, I just prefer it to be in a container rather than the host OS of the server.
- skip the reverse proxy entirely and use `your_server_hostname:8080` or `your_server_ip:8080` directly if that's your thing.
</details>

<details>
  <summary>production w/ cloudflare tunnel</summary>

```mermaid
  flowchart LR
    server["`**rust server**<br>_serve both the APIs<br>and the nuxt client_`"]
    db["`**postgres<br>database**`"]
    cloudflared["`**cloudflared**`"]

    server <-- postgres:5432 --> db
    server <-- yomuyume:8080 --> cloudflared
    cloudflared <-- secure tunnel --> cloudflare_cdn["`cloudflare<br>global CDN`"]
    cloudflare_cdn <-- example.com<br>_with TLS_ --> browser

    subgraph client_os["client OS"]
      browser
    end
    subgraph db_container["container"]
      db
    end
    subgraph server_container["container"]
      server
    end
    subgraph cloudflare_container["container"]
      cloudflared
    end

    subgraph docker
      server_container
      db_container
      cloudflare_container
    end

    subgraph server_os["server OS"]
      cloudflared
      docker
    end

    style server_os fill:transparent
    style docker fill:transparent
    style client_os fill:transparent
```
- one advantage of this is if you use cloudflare access to protect applications running on your server, yomuyume (in the future) can read the authentication headers from cloudflare and access the app directly without re-login.
</details>

### 4. repo structure
- `.devcontainer/`: everything needed for the development environment (`mold` linker, `dav1d`, `7zz`, etc.)
- `benches/`: some micro benchmarks
- `database/`: schemas, migrations `.sql` files
- `src/`: the nuxt spa web client
- `src-rust/`: the rust server

### 5. interact with the database
using `sqlx::query!()` to achieve compile-time syntactic and semantic checks, but it can only interact with one database inside a postgres server at a time (there's only one `DATABASE_URL` env var).

as a result, we share the same database for the backend itself and the benchmarks, so try to keep eveerything in `database/` neat and tidy. sqltools allows you to `Run Selected Query` in the context menu, use that.

## license

licensed under either of

-   Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## contribution

unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
