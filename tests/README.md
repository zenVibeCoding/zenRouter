# tests

跨组件的契约与集成验证放在这里。单元测试与各组件放在一起（Rust `#[cfg(test)]`、`adapters/litellm/tests`）。

计划内容（对应 [技术架构第 12 节](../docs/technical-architecture.md) 的验证门槛）：

| 目录（规划） | 门槛 | 内容 |
| --- | --- | --- |
| `contract/` | A | Gateway ↔ Adapter 契约、三类客户端入口与模拟上游 |
| `ledger/` | B | 两副本并发消耗同一部门额度、重复事件、未知用量 |
| `content/` | C / C2 | 文件留痕崩溃恢复；不留正文／脱敏／完整三种模式的违规副本扫描 |
| `deploy/` | E | 四类部署形态的初始化、升级、停止与恢复 |

骨架阶段为空。
