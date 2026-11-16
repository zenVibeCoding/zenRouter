# ZenRouter

**开源企业级 AI 路由与治理平台 · 前期调研与设计文档。**

让每一次 AI 调用可控、可见、可追溯。

ZenRouter 面向企业多团队的 AI 使用场景，统一接入模型服务，提供路由、成本与权限管理、内容安全分析、调用审计与企业可配置的正文留存能力。

本仓库只保留前期阶段的需求、调研、原型与架构文档，不包含实现代码。所有文档均为设计输入和建议，不代表功能已经实现或通过验证。

## 阅读顺序

1. [产品全景与范围基线（HTML）](docs/product-overview.html)：产品定位、四大模块、核心方向、建设边界与推进路线。
2. [产品需求基线](docs/product-requirements.md)：已确认的需求方向、功能范围、建议分期、关键流程、验收场景和待决策事项。
3. [业务架构总览（HTML）](docs/business-architecture.html) · [Markdown](docs/business-architecture.md)：业务闭环、关键对象与三条业务链路。
4. [业务模块拆分（HTML）](docs/business-modules.html)：组织与权限、安全与审计、模型与路由、额度与成本四个一级模块，以及二级能力、公共能力和责任边界。
5. [初版交互原型（HTML）](docs/prototype.html) · [功能地图](docs/prototype.html#map) · [功能模块与原型说明](docs/prototype-plan.md)：18 个页面入口，演示应用接入、逐级额度、调用追溯与风险复核。
6. [组织与权限需求原型（HTML）](docs/identity-prototype.html)：成员、部门、角色授权、项目应用、凭证、企业身份、账户会话与管理记录的专项原型。页面中的“已实现／待开发”等标记为 2026-11-05 的历史状态。
7. [技术选型与总体架构（HTML）](docs/technical-architecture.html) · [Markdown](docs/technical-architecture.md) · [架构图](docs/technical-architecture.svg) · [部署方式图](docs/deployment-modes.svg)：默认技术组合、模块与运行边界、完整调用链、层级预算、可靠留痕、四类部署规划和验证门槛。

## 调研资料

- [技术可行性调研（HTML）](docs/technical-research.html) · [Markdown](docs/technical-research.md)：五个开源候选、许可与商业边界、复用方案、技术栈建议及待执行的验证门槛。
- [网关专项比较（HTML）](docs/gateway-research.html) · [Markdown](docs/gateway-research.md)：网关能力、技术栈、模型范围比较。
- [LiteLLM Rust 与复用方案（HTML）](docs/litellm-reuse-research.html) · [Markdown](docs/litellm-reuse-research.md)：固定版本的 Rust 覆盖、Python Core SDK、独立网关架构及后续验证门槛。
- [三个治理模块技术调研（HTML）](docs/governance-research.html) · [Markdown](docs/governance-research.md)：组织与权限、安全与审计、额度与成本的推荐路线、替代方案、复用边界与许可。
- [调研来源清单](docs/research/)：官方资料、源码快照与竞品 README 的核查记录。

## 维护

修改五份调研／架构 Markdown 后，执行以下命令重新生成对应 HTML：

~~~sh
python3 scripts/render-research.py
~~~

原型可直接用浏览器打开，无需安装依赖。[原型检查脚本](scripts/check-prototype.cjs)需要 Node.js、Playwright 和 Chromium。

项目工作名为 **ZenRouter**；正式对外使用前仍需核查名称、域名与商标。
