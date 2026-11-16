"""ZenRouter 模型适配组件。

职责边界（docs/technical-architecture.md 第 3、6 节）：
- 对 Gateway 选定的端点发起一次调用、转换协议、输出原生数据与规范化计量／终态；
- 不管理组织、虚拟密钥、层级预算，不自主 fallback，SDK 内部重试必须关闭。
"""

CONTRACT_VERSION = "0.1"
"""Gateway ↔ Adapter 内部契约版本。

须与 Rust `zenrouter_contracts::ADAPTER_CONTRACT_VERSION` 一致（由测试校验）。
"""

__version__ = "0.1.0"
