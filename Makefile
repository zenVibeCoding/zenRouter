# ZenRouter 开发入口。运行 `make help` 查看全部目标。

SHELL := /bin/bash
.DEFAULT_GOAL := help

DEV_PG_PORT ?= 55442
DEV_DATABASE_URL ?= postgres://zenrouter:zenrouter-dev@127.0.0.1:$(DEV_PG_PORT)/zenrouter
# 优先使用 `docker compose` 插件，缺失时回退到独立的 docker-compose。
COMPOSE ?= $(shell docker compose version >/dev/null 2>&1 && echo 'docker compose' || echo docker-compose)
DEPS_COMPOSE := $(COMPOSE) -f deploy/compose/compose.deps.yaml
ADAPTER := adapters/litellm
IMAGE ?= zenrouter:dev

export ZENROUTER_DEV_PG_PORT := $(DEV_PG_PORT)

.PHONY: help
help: ## 显示帮助
	@awk 'BEGIN{FS=":.*## "} /^[a-zA-Z_-]+:.*## /{printf "  \033[32m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

## ---- 安装 ----
.PHONY: setup
setup: ## 安装前端与 Python 依赖，预取 Rust 依赖
	pnpm install --frozen-lockfile
	uv sync --project $(ADAPTER) --frozen
	cargo fetch --locked

## ---- 本地开发 ----
.PHONY: deps-up deps-down migrate dev console-dev adapter-dev
deps-up: ## 启动开发依赖（PostgreSQL），等待健康
	$(DEPS_COMPOSE) up -d --wait

deps-down: ## 停止开发依赖（保留数据卷）
	$(DEPS_COMPOSE) down

migrate: ## 对开发数据库执行迁移
	ZENROUTER__DATABASE__URL=$(DEV_DATABASE_URL) cargo run -q -- --config config/dev.toml migrate

dev: deps-up migrate ## 组合运行 gateway+control+worker，并监督 Python Adapter
	@mkdir -p .data/content
	ZENROUTER__DATABASE__URL=$(DEV_DATABASE_URL) cargo run -- --config config/dev.toml serve --roles all

console-dev: ## 启动控制台开发服务器（热更新，代理到 127.0.0.1:8081）
	pnpm console:dev

adapter-dev: ## 单独启动 Python Adapter
	uv run --project $(ADAPTER) zenrouter-adapter

## ---- 检查 ----
.PHONY: check fmt lint test test-rust test-python test-console
check: lint test ## 全部格式、静态检查与测试

fmt: ## 格式化 Rust 与 Python 代码
	cargo fmt --all
	uv run --project $(ADAPTER) ruff format $(ADAPTER)

lint: ## 格式检查与静态检查
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cd $(ADAPTER) && uv run ruff check . && uv run ruff format --check .
	pnpm console:typecheck

test: test-rust test-python test-console ## 运行全部测试

test-rust:
	cargo test --workspace --locked

test-python:
	cd $(ADAPTER) && uv run pytest -q

test-console:
	pnpm console:build

## ---- 构建与部署 ----
.PHONY: build image compose-up compose-down
build: ## 构建控制台与 release 二进制
	pnpm console:build
	cargo build --release --locked

image: ## 构建统一应用镜像
	docker build -f deploy/docker/Dockerfile -t $(IMAGE) .

compose-up: ## 以 Compose 默认安装方式启动（需 deploy/compose/.env）
	$(COMPOSE) -f deploy/compose/compose.yaml up -d --build

compose-down: ## 停止 Compose 安装
	$(COMPOSE) -f deploy/compose/compose.yaml down
