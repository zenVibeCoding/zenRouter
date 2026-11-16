# syntax=docker/dockerfile:1
#
# ZenRouter 统一应用镜像：Rust 组合角色 + Python Adapter + 控制台静态资源。
# 同一镜像通过 `serve --roles ...` 选择运行角色；数据库迁移使用 `migrate`。
# 构建：docker build -f deploy/docker/Dockerfile -t zenrouter:dev .

ARG RUST_IMAGE=rust:1.97.0-slim-trixie
ARG NODE_IMAGE=node:22.23-trixie-slim
ARG PYTHON_IMAGE=python:3.13.13-slim-trixie

# ---- 控制台 ----
FROM ${NODE_IMAGE} AS console
WORKDIR /src
RUN corepack enable
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/console/package.json apps/console/
RUN pnpm install --frozen-lockfile
COPY apps/console apps/console
RUN pnpm --filter @zenrouter/console build

# ---- Rust ----
FROM ${RUST_IMAGE} AS rust
WORKDIR /src
COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY crates crates
COPY migrations migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p zenrouter-runtime \
    && cp target/release/zenrouter /usr/local/bin/zenrouter

# ---- Python Adapter（锁定依赖，运行时不下载包）----
FROM ${PYTHON_IMAGE} AS adapter
COPY --from=ghcr.io/astral-sh/uv:0.11.8 /uv /usr/local/bin/uv
ENV UV_COMPILE_BYTECODE=1 UV_LINK_MODE=copy UV_PYTHON_DOWNLOADS=never
WORKDIR /opt/zenrouter/adapter
COPY adapters/litellm/pyproject.toml adapters/litellm/uv.lock adapters/litellm/README.md ./
RUN uv sync --frozen --no-dev --no-install-project
COPY adapters/litellm/src src
RUN uv sync --frozen --no-dev

# ---- 运行镜像 ----
FROM ${PYTHON_IMAGE} AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 zenrouter \
    && useradd --system --uid 10001 --gid zenrouter --home-dir /var/lib/zenrouter zenrouter \
    && mkdir -p /var/lib/zenrouter/content \
    && chown -R zenrouter:zenrouter /var/lib/zenrouter

COPY --from=rust /usr/local/bin/zenrouter /usr/local/bin/zenrouter
COPY --from=adapter /opt/zenrouter/adapter /opt/zenrouter/adapter
COPY --from=console /src/apps/console/dist /opt/zenrouter/console
COPY --chmod=0644 deploy/docker/zenrouter.toml /etc/zenrouter/zenrouter.toml

ENV ZENROUTER_CONFIG=/etc/zenrouter/zenrouter.toml \
    PATH=/opt/zenrouter/adapter/.venv/bin:$PATH
USER zenrouter
WORKDIR /var/lib/zenrouter
EXPOSE 8080 8081

# tini 负责信号转发与僵尸进程回收；业务就绪与排空由 zenrouter 自身处理。
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/zenrouter"]
CMD ["serve", "--roles", "all"]
