# zenrouter-adapter

ZenRouter 的 Python 模型适配组件，封装 LiteLLM Core SDK，独立进程运行。

- 只执行 Gateway 已选定的一次上游尝试，不管理组织、密钥、预算，不自主 fallback。
- 只安装 `litellm-core`，不与常规 `litellm` 分发包并存。
- 契约版本见 `zenrouter_adapter.CONTRACT_VERSION`，与 Rust 侧 `ADAPTER_CONTRACT_VERSION` 由测试保持一致。

~~~sh
uv sync                      # 安装依赖（含 dev 组）
uv run zenrouter-adapter     # 启动，默认 127.0.0.1:8090
uv run pytest                # 测试
uv run ruff check . && uv run ruff format --check .
~~~
