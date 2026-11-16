"""启动 Adapter：`zenrouter-adapter` 或 `python -m zenrouter_adapter`。

配置来自环境变量：
- ZENROUTER_ADAPTER_HOST（默认 127.0.0.1，只应监听 loopback 或受控内网）
- ZENROUTER_ADAPTER_PORT（默认 8090）
- ZENROUTER_ADAPTER_LOG_LEVEL（默认 info）
"""

import os

import uvicorn

from .app import create_app


def main() -> None:
    host = os.environ.get("ZENROUTER_ADAPTER_HOST", "127.0.0.1")
    port = int(os.environ.get("ZENROUTER_ADAPTER_PORT", "8090"))
    log_level = os.environ.get("ZENROUTER_ADAPTER_LOG_LEVEL", "info")
    uvicorn.run(
        create_app(),
        host=host,
        port=port,
        log_level=log_level,
        # 正文可能出现在访问日志的查询串中，默认关闭访问日志。
        access_log=False,
        # 收到 SIGTERM 后等待存量流结束的上限，由 Rust 监督进程的停止超时兜底。
        timeout_graceful_shutdown=10,
    )


if __name__ == "__main__":
    main()
