# 参与开发

## 环境

Rust 1.97（由 `rust-toolchain.toml` 自动选择）、Node.js ≥ 22.22 与 pnpm 10、uv、Docker（含 Compose）。

~~~sh
make setup   # 安装依赖
make dev     # 本地运行全部角色
make check   # 提交前必须通过：格式、静态检查与全部测试
~~~

## 分支与合并

- `main` 始终可构建、CI 全绿；不直接推送未经检查的改动。
- 功能分支命名 `<类型>/<简述>`，例如 `feat/attempt-contract`、`fix/adapter-shutdown`。
- 通过 Pull Request 合并，CI 通过后合入；倾向 squash 合并，保持 `main` 历史清晰。

## 提交信息

采用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/)，描述用中文：

~~~text
<类型>(<范围>): <简述>

<可选正文：为什么改、影响什么>
~~~

| 类型 | 用途 |
| --- | --- |
| feat | 新功能 |
| fix | 缺陷修复 |
| docs | 文档 |
| refactor | 不改变行为的重构 |
| test | 测试 |
| build / ci | 构建、依赖、CI |
| chore | 其他杂项 |

范围取模块名：`gateway`、`control-api`、`worker`、`runtime`、`domain-billing`、`adapter`、`console`、`deploy`、`docs` 等。

## 代码约定

- 依赖方向：入口层 → `application` → `domain-*`；`storage`、`integrations` 实现应用层接口，由 `runtime` 装配。领域 crate 不依赖 HTTP 框架、SQL 或 SDK。
- 金额使用 `rust_decimal`，不使用浮点数。
- 日志、错误信息和探针响应不得包含正文、密钥或连接串。
- 新增外部依赖在根 `Cargo.toml` 的 `[workspace.dependencies]` 统一声明版本；锁文件随改动一起提交。
- Gateway ↔ Adapter 契约版本变化时同步修改 Rust 与 Python 两侧，并更新 `api/`。
- 设计依据见 [技术架构](docs/technical-architecture.md) 与 [开发计划](docs/development-plan.md)；偏离时在 PR 中说明。
