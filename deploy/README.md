# Yomuyume deployment guide

# 1. Pre-requisites

- [Docker Engine](https://docs.docker.com/engine/install/), remember to follow the [post-installation steps](https://docs.docker.com/engine/install/linux-postinstall/).

- [100+ Docker Concepts you Need to Know](https://youtu.be/rIrNIzy6U_g) if you don't know Docker.

# 2. Preparing the library

- The library is a directory with the following structure:
    ```plaintext
    library
    ├── <category>
    │   ├── <title>.zip
    │   ├── <title>.toml
    │   └── ...
    ├── <category>.toml
    └── ...
    ```

- `<category>.toml` matches the category name, and contains the following:
    ```toml
    name = "Category name"
    description = "Category description"
    id = "<uuid>" # safely ignore this field
    ```

- `<title>.toml` matches the title name, and contains the following:
    ```toml
    title = "Title name"
    description = "Title description"
    release = "YYYY-MM-DD"
    author = "Title author"
    tags = ["Tag 1", "Tag 2"]
    cover = "cover.jpg"

    [descriptions]
    "<page>" = "Description for page <page>"
    ```

# 3. Building & Deploying

## 2.1. Client

### 2.1.1. Building the static site

> Bring your own web server

```sh
curl -sL https://github.com/Liminova/yomuyume/raw/main/deploy/generate-client.sh

sh generate-client.sh
```

- Read what the script does, don't blindly execute scripts from strangers on the internet.

- The script calls itself inside Docker containers to generate the static site, so you will need to actually download the script and execute it locally, no one-liner here.

- Make sure there is no `yomuyume` and `yomuyume-client` directory where you are running the script.

- After finishing,
    - you will have a `yomuyume-client` directory with the static site
    - `.pnpm-store` is the cache directory for the `pnpm` package manager, keep for faster future builds or `rm -rf`

- Use Caddy to serve, push to GitHub Pages, Vercel, Netlify, etc.

#### Docker compose example with Caddy

- Download [Caddy](https://caddyserver.com/download) and place in the same directory as the `Caddyfile` and `yomuyume-client` directory.

- See [examples/client-caddy](./examples/client-caddy/) for a Docker Compose example with Caddy.

- `docker-compose up`, add `-d` to run in the background.

### 2.1.2. With a web server

- See [examples/client-caddy](./examples/client-caddy/).

## 2.2. Server

- See [examples/](./examples/recommended/).

| Environment variable | Description                                            | Default                      | Required |
|----------------------|--------------------------------------------------------|------------------------------|----------|
| `APP_NAME`           | Name of the app                                        | `yomuyume`                   |          |
| `LIBRARY_PATH`       | Path to the library                                    | `/library`                   | ⭕        |
|                      |                                                        |                              |          |
| `SERVER_ADDRESS`     | Address for the server to listen, better not change it | `0.0.0.0`                    |          |
| `SERVER_PORT`        | Port for the server to bind                            | `3000`                       |          |
| `DATABASE_URL`       | Database URL                                           | `sqlite:/sqlite.db?mode=rwc` | ⭕        |
|                      |                                                        |                              |          |
| `JWT_SECRET`         | JWT secret                                             |                              | ⭕        |
| `JWT_MAXAGE_DAY`     | JWT max age in days                                    | `30`                         |          |
|                      |                                                        |                              |          |
| `SMTP_HOST`          | SMTP host                                              |                              | ⚠️        |
| `SMTP_PORT`          | SMTP port                                              |                              | ⚠️        |
| `SMTP_USERNAME`      | SMTP username                                          |                              | ⚠️        |
| `SMTP_PASSWORD`      | SMTP token/password                                    |                              | ⚠️        |
| `SMTP_FROM_EMAIL`    | SMTP from email                                        |                              | ⚠️        |
| `SMTP_FROM_NAME`     | SMTP from name                                         |                              | ⚠️        |
|                      |                                                        |                              |          |
| `FFMPEG_PATH`        | Path to ffmpeg to transcode                            |                              | ⚠️        |
| `DJXL_PATH`          | Path to djxl                                           |                              | ⚠️        |
| `TEMP_DIR`           | Path to temporary directory                            | `/tmp`                       |          |

- ⭕ Required

- ⚠️ Optional, but required for some features:
    - SMTP for email verification/password reset/account recovery.
    - DJXL and FFMPEG for decoding `JPEG XL` and `AVIF` pages respectively. `PNG`, `WEBP`, `JPEG` and `GIF` are natively supported.

- `docker-compose up`, add `-d` to run in the background.