# 网关能力、技术栈与模型范围

2026-10-29 · 技术调研补充 v0.3 · 文档与源码声明核查，尚未实测

## 1. 当前选型方向

**当前研究收敛到自建 ZenRouter 网关、复用 LiteLLM 核心能力。** Rust 为首选，允许 Python。建议首期以自有 Rust 网关封装 LiteLLM Python Core SDK，后续按已验证的链路引入 Rust 原生组件。固定版本源码、可复用边界与方案比较见[LiteLLM Rust 与复用专项](litellm-reuse-research.html)。

ZenRouter 掌握对外接口、模型路由策略、组织权限、逐级额度和可靠审计；LiteLLM 主要承担适配、协议转换与流式等可复用能力。完整 LiteLLM Proxy 用作验证基线，agentgateway、Bifrost、Higress 等保留为参照；首期默认不直接采用它们的完整网关作为产品核心。

本页保留完整网关产品的能力比较，不能直接作为 Core SDK 或 Rust 组件的能力承诺。LiteLLM Rust 已追加固定发布与 main 源码核查；其他 Rust 候选仍以官方资料、清单和许可核查为主。全部候选尚未部署实测。

## 2. 技术栈与产品侧重点

| 网关 | 核心技术栈 | 能力侧重点 | 对 ZenRouter 的意义 |
| --- | --- | --- | --- |
| LiteLLM | 现有 Proxy 为 Python／FastAPI；管理界面 Next.js／React；另有可选 Rust 核心及独立 Rust 服务的测试路线 | 多供应商协议适配、路由、用量成本、回调与治理接口 | 保留为主候选；不能把可选 Rust 路径的覆盖范围当作完整 Proxy 能力 |
| agentgateway | Rust；仓库依赖 Tokio、Hyper、Axum、Tower；支持独立进程与 Kubernetes 部署 | LLM 与 MCP／A2A 流量、策略、身份、路由和观测 | Rust 方案参照；企业组织、拨款与复盘仍需自行建设 |
| Bifrost | Go 核心与 HTTP 网关；React 管理界面；Go 插件 | 模型转发、回退、基础治理、缓存和可观测 | 功能与实现方式的对照候选，不因 Go 被直接排除 |
| Higress | Envoy／Istio 基础，Wasm 插件可用 Go、Rust、JS | 通用流量治理与 AI 插件结合 | 可用 Rust 写插件，但不等于整个数据面为原生 Rust |
| Portkey Gateway | TypeScript／Hono，支持 Node.js 运行 | 模型代理、路由、回退、缓存和风控集成 | 留作参照；与其托管管理平台分开评估 |
| New API | Go／Gin；React／TypeScript；当前前端使用 Rsbuild | 渠道、模型、用户、额度和费用管理控制台 | 适合参考产品配置体验；AGPL 路线单独评估 |

技术栈依据来自官方仓库 README 与依赖文件：[LiteLLM Python 依赖](https://github.com/BerriAI/litellm/blob/b25ea48c4b194dbae4d858839619c426f1258ac6/pyproject.toml)、[LiteLLM UI](https://github.com/BerriAI/litellm/blob/b25ea48c4b194dbae4d858839619c426f1258ac6/ui/litellm-dashboard/package.json)、[agentgateway 依赖](https://github.com/agentgateway/agentgateway/blob/main/Cargo.toml)、[Bifrost](https://github.com/maximhq/bifrost)、[Higress](https://github.com/higress-group/higress)、[Portkey 依赖](https://github.com/Portkey-AI/gateway/blob/669825cbe89ee51569918b8f78a9db486fd69dd4/package.json)、[New API](https://github.com/QuantumNous/new-api)。框架名称说明实现方式，不构成性能排名。

### LiteLLM 的 Rust 路线怎样理解

官方 Rust 文档目前标为 Beta：一种方式是在 Python Proxy 内为部分模型启用 Rust 请求转换和网络调用，鉴权、配置、路由、回调及费用等仍由 Python 管理；另一种是独立 Axum 服务，功能覆盖更少，文档尚未提供预构建镜像。不同端点的流式、工具和参数支持也不同。因此本轮比较 LiteLLM 时，以现有 Proxy 的完整能力为基准，Rust 路径另做实验。来源：[LiteLLM Rust Gateway](https://docs.litellm.ai/docs/proxy/rust_gateway)。

补充源码结论：v1.106.6 已含模块化 Rust workspace，main 中还有更多供应商与内部接口变化；顶层 litellm crate 仍无实际 SDK API。Anthropic 原生 Messages 流式与 Anthropic → OpenAI Chat 流式是不同链路，后者在所查 Rust 实现中尚不支持。详情及固定提交证据见[复用专项的协议矩阵](litellm-reuse-research.html#section-4)。

## 3. 网关能力比较

“有”表示官方资料描述了能力或接口，尚不代表所有供应商、模型和部署方式都通过验证。“待核”表示本轮证据不足，不直接判定不支持。下表面向开源网关，不把商业控制台能力合并进来。

| 能力 | LiteLLM | agentgateway | Bifrost | Higress |
| --- | --- | --- | --- | --- |
| 统一调用与协议转换 | 多类端点与供应商适配 | OpenAI、Messages、Responses 等，按供应商有原生／转换差异 | 统一接口与供应商兼容入口，矩阵按操作细分 | AI Proxy 提供 OpenAI／Claude 协议与供应商转换 |
| 文本流式、工具、多模态 | 有，按模型与转换路径验证 | 有，按模型与转换路径验证 | 有，按供应商操作矩阵验证 | 有流式与多模态示例；工具字段保真需专测 |
| 同模型多渠道、重试回退 | 模型组部署与回退配置 | 虚拟模型、负载分配、fallback | 多 Key／供应商分配与回退 | 路由和插件组合；具体失败语义需核对 |
| 缓存 | 网关响应缓存；上游缓存观测另看用量 | 上游缓存计量需核；本轮未确认通用响应缓存方案 | 语义缓存等；上游缓存是独立口径 | AI Cache 插件 |
| 安全扩展 | 自定义 guardrail 与回调 | 请求／响应 guardrail、自定义 webhook、外部鉴权／处理 | Go 插件；部分安全产品能力在企业版 | 脱敏、安全与 Wasm 插件 |
| 额度和费用基础 | Key／用户／团队预算，含有条件预占 | Key 的金额或 Token 预算；不能替代分级拨款 | Key／团队等基础预算 | Token 限流与额度插件，业务账本另建 |
| 观测与日志 | 成本、日志回调、观测集成 | OTel、访问日志、数据库日志、费用展示 | 日志、指标与追踪 | 指标、访问日志与 AI 统计 |

主要依据：[LiteLLM 端点目录](https://docs.litellm.ai/docs/supported_endpoints)、[LiteLLM 架构](https://docs.litellm.ai/docs/proxy/architecture)、[agentgateway 功能与协议矩阵](https://agentgateway.dev/docs/standalone/latest/)、[agentgateway 虚拟模型](https://agentgateway.dev/docs/standalone/latest/documentation/llm/virtual-models/)、[Bifrost 操作矩阵](https://docs.getbifrost.ai/providers/supported-providers/overview)、[Higress AI Proxy](https://higress.io/en/docs/latest/plugins/ai/api-provider/ai-proxy/)、[Higress 插件](https://higress.ai/en/plugins/)。

两项与我们需求直接相关的限制：agentgateway 文档示例是请求完成后计入预算、超额后拒绝后续请求，而且未配置价格的模型不会产生金额扣计；这不是已经证明的并发预算预占。其 webhook 支持放行、脱敏、拒绝和失败策略，但工具字段与流式检查窗口仍需实测。来源：[Key 预算行为](https://agentgateway.dev/docs/standalone/latest/documentation/llm/cost-controls/budget-limits/per-key/)、[Webhook 行为](https://agentgateway.dev/docs/standalone/latest/documentation/llm/prompt-guards/webhooks/)。

## 4. 模型支持范围

网关通常不负责运行模型权重，而是接入供应商或企业自己的推理服务。模型范围应按“模型家族 → 供应商渠道 → API 类型 → 具体功能”核对。模型在目录里、存在兼容入口、生产可用，是三个不同状态。

下表列代表性范围，不是完整型号清单，不承诺某一具体新版本已通过验证。**适配**指官方目录有专门条目；**兼容**指通过 OpenAI 等兼容接口接入；**待核**指本轮未确认专用适配。

| 模型／接入范围 | LiteLLM | agentgateway | Bifrost | Higress |
| --- | --- | --- | --- | --- |
| OpenAI 模型 | 适配 OpenAI／Azure | 适配 OpenAI／Azure | 适配 OpenAI／Azure | 适配 OpenAI／Azure |
| Claude | 适配 Anthropic、Bedrock 等 | 适配 Anthropic、Bedrock、Vertex 等 | 适配 Anthropic、Bedrock、Vertex 等 | AI Proxy 列出 Claude |
| Gemini | Google AI Studio／Vertex | Gemini／Vertex | Gemini／Vertex | AI Proxy 列出 Gemini |
| DeepSeek | 专用条目 | 专用条目 | 专用条目 | 专用条目 |
| 通义千问 Qwen | DashScope 等专用条目 | 官方模型手册列为 OpenAI 兼容接入 | 专用条目待核；可评估自定义兼容供应商 | Qwen 专用配置 |
| Kimi／Moonshot | 专用条目 | 官方模型手册列为 OpenAI 兼容接入 | 专用条目待核；可评估自定义兼容供应商 | Moonshot 专用配置 |
| 智谱 GLM、MiniMax、豆包 | Z.AI、MiniMax、Volcengine 条目 | 手册列出 Zhipu 兼容接入；其他渠道需逐项核查 | 专用条目待核；兼容能力需逐项核查 | 智谱、MiniMax 有条目；豆包本轮待核 |
| 企业自部署 Llama／Qwen 等 | Ollama、vLLM 等入口 | Ollama 及 vLLM、LM Studio 等兼容入口 | Ollama、vLLM、SGLang 等入口 | Ollama 入口；其他服务按兼容协议验证 |

来源：[LiteLLM 供应商目录](https://docs.litellm.ai/docs/providers)、[agentgateway 接入目录](https://agentgateway.dev/docs/standalone/latest/integrations/llm/providers/)、[agentgateway 模型手册](https://agentgateway.dev/models/)、[Bifrost 供应商矩阵](https://docs.getbifrost.ai/providers/supported-providers/overview)、[Bifrost 自定义供应商](https://docs.getbifrost.ai/providers/custom-providers)、[Higress 供应商配置](https://higress.io/en/docs/latest/plugins/ai/api-provider/ai-proxy/)。

Portkey 的公开仓库列出 OpenAI、Azure、Gemini、Anthropic、Mistral、Cohere、Ollama 等；New API 列出 OpenAI、Anthropic、Gemini、Azure、Bedrock、Vertex、DeepSeek、Qwen 等。两者也支持更广范围的渠道，但这轮不把宣传模型数量作为验收范围。来源：[Portkey](https://github.com/Portkey-AI/gateway#supported-providers)、[New API](https://github.com/QuantumNous/new-api)。

### 同一个模型还要看哪些功能

| 功能 | 要核对的内容 |
| --- | --- |
| 文本与流式 | 普通回复、逐段输出、取消、断流、用量结束事件 |
| 工具调用 | 参数、并行调用、工具结果回传、供应商专有工具 |
| 结构化输出与推理 | JSON Schema、推理参数及返回字段是否保留 |
| 图片与音视频 | 图片输入不等于图片生成；语音转写不等于实时语音；视频通常有异步任务协议 |
| 向量与重排 | Embeddings 和 Rerank 独立验证，不由聊天支持推导 |
| 缓存与计量 | 缓存读取／写入 Token、价格、缺失字段、失败尝试费用 |
| 渠道切换 | 替补渠道是否真有同版本、同上下文与同功能；不能只匹配模型显示名 |

例如 agentgateway 的协议矩阵明确区分原生支持、协议转换、估算、透传和未实现；Bifrost 则按供应商分别列聊天、Responses、图像、语音、向量等操作。我们应沿用这种可核对的矩阵，而不是写一句“支持所有主流模型”。

## 5. 其他 Rust 候选的筛选结果

| 候选 | 已查到的事实 | 本轮处理 |
| --- | --- | --- |
| TensorZero | Rust 网关，兼具结构化推理、观测、评测与实验；GitHub 显示仓库在 2026 年 6 月 12 日归档 | 保留架构参考；不作为默认可持续跟随上游升级的主候选 |
| Helicone 独立 ai-gateway | Rust／Tokio／Axum；模型代理、路由、限流和缓存；当前 Cargo 标记测试版本；README 与 Cargo 声明 Apache-2.0，根 LICENSE 内容为 GPL-3.0 | 许可证声明冲突未解决前暂不作为首选；不与 Helicone 其他仓库或托管服务混为一谈 |
| LiteLLM Rust 路线 | 官方明确是可选 Beta，且独立服务覆盖少于 Python Proxy | 作为 LiteLLM 的单独实验项，不提前承诺完整替代 |

来源：[TensorZero 仓库归档状态](https://github.com/tensorzero/tensorzero)、[TensorZero 网关](https://www.tensorzero.com/docs/gateway)、[Helicone README](https://github.com/Helicone/ai-gateway)、[Helicone Cargo](https://github.com/Helicone/ai-gateway/blob/main/Cargo.toml)、[Helicone LICENSE](https://github.com/Helicone/ai-gateway/blob/main/LICENSE)、[LiteLLM Rust 状态](https://docs.litellm.ai/docs/proxy/rust_gateway)。这是对实际证据的筛选，不是对 Rust 生态整体成熟度的结论。

## 6. 对 ZenRouter 的技术建议

阶段安排已收敛：当前保留以下网关路线与验证事项，专项实验后置；下一轮先补齐其他技术方向的基线，不继续在网关侧展开。

Rust 优先用于自有运行时治理和业务服务，候选组合为 Tokio／Axum／Tower，PostgreSQL 保存组织、配置与权威额度账目，正文使用对象存储接口，控制台仍采用 TypeScript。语言偏好不要求所有组件都由 Rust 实现。

后续网关专项优先验证 LiteLLM Core SDK 的独立封装和限定链路的 Rust 组件嵌入。主流模型直连、同模型双渠道、国内供应商、自部署推理入口、流式、工具调用、缓存、预算、可靠日志与风控仍使用同一验收标准；完整 Proxy 作为对照。若某个型号缺少 SDK 路径，再评估补丁、兼容接入和自写适配的代价。

当前没有性能证据证明混合语言或原生 Rust 哪种更快。自有网关与治理架构是项目方向，模型适配优先复用；本轮仍停留在研究与方案验证准备，没有开始实现生产网关。

早期其他 Rust 候选的证据见[Rust 来源清单](research/rust-source-manifest.json)，其中部分只记录默认分支及校验值。此次 LiteLLM 深入研究另保存了[固定发布与 main 证据](research/litellm-rust-manifest.json)，不应混淆两批证据的版本精度。

## 7. 多实例负载均衡、横向扩容与高可用

已纳入核心要求：多个 ZenRouter 网关实例共同承接业务流量，通过统一入口分发请求，并支持扩缩容、故障摘除和滚动升级。该能力属于“模型与路由”及横向的“平台管理与运维”，不新增一级模块。

这里有两层负载均衡：入口将请求分给不同网关实例，解决我们自身的承载能力；每个网关再按统一策略选择上游模型渠道，解决供应商容量与可用性。扩容网关不会自动增加上游供应商的配额。

| 范围 | 需要具备的能力 |
| --- | --- |
| 网关承载 | 多实例部署、入口健康检查、故障摘除、按需扩容；入口自身也需避免单点 |
| 全局治理 | 同一部门、员工和供应商的额度与限流跨实例有效；配置和撤权有明确生效时效；各次尝试和日志可关联 |
| 流式与升级 | 单次流式连接由承接实例持续处理；计划缩容或升级先停止接收新流量，再在约定时间内等待存量请求结束；不能承诺实例崩溃后把已输出流无缝迁移 |
| 整体容量 | 联合评估活跃流式连接、请求时长、排队、CPU、网络和存储压力；入口、共享存储、风控和日志链路同时纳入容量与恢复验证 |

### 完整网关部署的参照依据

LiteLLM 官方生产指南已有多 Pod、横向扩容、共享 Redis／数据库与后台任务分工的方案；同时指出运行时配置通过定期轮询收敛，数据库连接数会随实例和 worker 增加。这说明有可复用的部署路线，但不能把它视为我们整套治理已经具备一致性保证。来源：[LiteLLM 生产部署建议](https://docs.litellm.ai/docs/proxy/prod)。

agentgateway 的 Kubernetes 配置文档提供代理 Deployment 的 HPA、Pod 中断预算和优雅退出配置。需要区分代理扩容与控制面扩容，也要区分部署模式：其内置每 Key 金额预算文档目前限定 standalone 模式；本地限流计数按实例独立，共享限流需外部服务；金额预算不能由 Token 限流直接替代。来源：[代理扩容与退出](https://agentgateway.dev/docs/kubernetes/latest/documentation/setup/customize/configs/)、[预算模式边界](https://agentgateway.dev/docs/standalone/latest/documentation/llm/cost-controls/budget-limits/)、[全局与本地限流](https://github.com/agentgateway/website/blob/main/assets/agw-docs/pages/agentgateway/llm/rate-limit.md)。

### 补充验证范围

同一套业务流量经统一入口分到至少两个网关实例，分别验证增加实例、停止实例、滚动升级，以及共享预算、配置更新和日志恢复。压测必须包含长时间流式请求，记录吞吐、错误率、首 Token 延迟、尾延迟和共享依赖负载；不以“两个容器都能启动”作为完成标准。真实规模尚未确定，本轮不承诺单实例容量、实例数量或无损故障切换。

## 8. 下游自主选择协议：同协议代理与跨协议转换

已确认的产品要求：ZenRouter 同时提供 OpenAI、Anthropic 和 Gemini 三类客户端兼容入口，由下游用户通过 SDK／调用端点自行选择；同一上游模型可面向多个已支持的客户端协议。OpenAI 是必备入口，Chat Completions 与 Responses 分别记录兼容性，不将两者混为一个检查项。接口兼容覆盖请求、非流式响应、流式事件和错误格式，而不只是最终的文本内容。

例如同一个 Anthropic 上游：使用 OpenAI Chat Completions 的客户端发来请求，网关将请求转换为 Anthropic Messages，并将上游返回转换回 OpenAI 格式；使用 Anthropic Messages 的客户端发来请求，则按 Anthropic 协议代理并返回。两种方式同时提供，由调用方选择，管理员配置模型权限和可用渠道。Gemini 原生入口也按相同原则纳入范围。

### 候选网关的客户端接口

“文档有”表示官方资料明确列出该客户端入口，不等于全部字段、模型和跨协议组合已经实测通过。“待核”表示本轮证据不足，不代表产品不支持。此处讨论客户端到网关的 API，区别于网关能调用的上游模型。

| 候选 | OpenAI Chat Completions | OpenAI Responses | Anthropic Messages | Gemini 原生接口 | 主要边界 |
| --- | --- | --- | --- | --- | --- |
| LiteLLM 完整 Proxy | 文档有 | 文档有 | 文档有 | 文档有 generateContent／streamGenerateContent | 按实际转换链及端点验证；不将完整 Python Proxy 的能力套用到 Rust Beta |
| agentgateway | 文档有 | 文档有，但取决于上游 | 文档有 | 文档有 generateContent／streamGenerateContent | Gemini 原生请求仅进入 Gemini API、Vertex Gemini 或声明对应格式的自定义后端；不能推断任意互转 |
| Bifrost | 文档有 | 文档有 | 文档有 Anthropic SDK 入口 | 文档有 Google GenAI SDK 入口 | 原生客户端入口与内部转换并存；具体流式、工具和扩展字段仍需验证 |
| Higress AI Proxy | 文档有 | 本轮待核 | 文档有 | original 原协议模式已列出，Gemini 端点及治理覆盖需专项验证 | 文档明确自动识别 OpenAI Chat 与 Claude Messages；不能用上游 Gemini 适配证明完整原生客户端兼容 |
| New API | 文档有 | 文档有 | 文档有 | 文档有 generateContent／streamGenerateContent | README 描述当前源码；需固定实际部署版本核对，字段映射仍受渠道与上游能力限制 |
| Portkey 开源 Gateway | 文档有 | 本轮待核 | 本轮原生客户端入口待核 | 本轮原生客户端入口待核 | 已确认 OpenAI 兼容客户端；支持 Anthropic／Gemini 供应商不自动等于原生入口与治理完整可用 |

LiteLLM 依据：[端点目录](https://docs.litellm.ai/docs/supported_endpoints)、[Anthropic Messages](https://docs.litellm.ai/docs/anthropic_unified/)、[Gemini 原生与流式](https://docs.litellm.ai/docs/generateContent)、[Responses](https://docs.litellm.ai/docs/response_api)。

agentgateway 依据：[客户端／供应商矩阵](https://agentgateway.dev/docs/standalone/latest/)、[Gemini 原生路由边界](https://agentgateway.dev/docs/standalone/latest/documentation/llm/api-types/gemini/)、[Messages 转换](https://agentgateway.dev/docs/standalone/latest/documentation/llm/api-types/messages/)。

Bifrost 依据：[OpenAI SDK](https://docs.getbifrost.ai/integrations/openai-sdk/overview)、[Anthropic SDK](https://docs.getbifrost.ai/integrations/anthropic-sdk/overview)、[Google GenAI SDK](https://docs.getbifrost.ai/integrations/genai-sdk/overview)。其余依据：[Higress AI Proxy](https://higress.io/en/docs/latest/plugins/ai/api-provider/ai-proxy/)、[New API 接口清单](https://github.com/QuantumNous/new-api#protocols-and-endpoints)、[Portkey 开源入口示例](https://github.com/Portkey-AI/gateway#quickstart)。

### 以用户提出的 Anthropic 上游核对转换链

下表箭头表示请求方向：下游客户端 → 网关 → 上游；响应沿反方向转换回客户端使用的协议。

| 代表性链路 | LiteLLM | agentgateway | Bifrost |
| --- | --- | --- | --- |
| OpenAI Chat 客户端 → Anthropic 上游 | 文档有转换 | 矩阵标为转换支持 | 文档有转换 |
| Anthropic Messages 客户端 → Anthropic 上游 | 文档有同协议接入 | 矩阵标为原生支持 | 文档有同协议接入 |
| OpenAI Responses 客户端 → Anthropic 上游 | 文档有桥接及流式示例 | 当前矩阵标为未实现 | 文档有转换及流式支持 |

链路依据：[LiteLLM Anthropic 适配](https://docs.litellm.ai/docs/providers/anthropic)、[LiteLLM Responses 桥接](https://docs.litellm.ai/docs/response_api)、[LiteLLM Messages](https://docs.litellm.ai/docs/anthropic_unified/)、[agentgateway 能力矩阵](https://agentgateway.dev/docs/standalone/latest/)、[Bifrost Anthropic 操作与转换](https://docs.getbifrost.ai/providers/supported-providers/anthropic)、[Bifrost Anthropic 客户端](https://docs.getbifrost.ai/integrations/anthropic-sdk/overview)。

### 选型与产品边界

本轮判断：多协议入口支持继续研究 LiteLLM 核心复用；agentgateway 保留为 Rust 参照，但 Anthropic 上游的 Responses 转换缺口必须明确评估。不能仅以“支持 OpenAI”认定这条链路满足要求，也不要求用户在同协议与转换模式之间做全局取舍。SDK 封装后的能力须按[复用专项](litellm-reuse-research.html)重新验证。

同协议代理以保留原生能力为目标，不要求内部绝不使用统一表示，也不承诺字节级透明转发；鉴权、模型映射和经授权的风控处置仍可改变请求。无论选择哪种入口，都必须执行统一权限、额度、安全、路由和审计规则。

跨协议转换不代表全部功能等价。工具调用、结构化输出、思考签名、缓存用量、错误及流式终态要按链路验证；不支持的组合或关键字段应给出明确错误或声明限制，不能静默丢弃后返回成功。Realtime、WebSocket、文件、批处理等列为独立兼容项，三类核心接口的支持不自动覆盖这些能力。

下一轮验证采用三家官方客户端／SDK，对同一上游分别发起同协议和跨协议请求，检查返回格式与治理行为；先覆盖上述 Anthropic 两条核心链路，并将 OpenAI Responses → Anthropic 作为单独门槛。当前仍为文档核查，尚未执行实际调用。

### LiteLLM 专项结论：多协议同时开放，包含 Anthropic 流式

结论：以完整 LiteLLM Python Proxy 为基准，官方资料已经覆盖用户要求的多协议并存、Anthropic 同协议流式，以及 Anthropic 上游转换成 OpenAI 客户端格式的能力，可作为首轮验证路线。此结论基于文档核查，不是 ZenRouter 已部署实测的结论，也不适用于尚未核对的 Rust Beta 功能集合。

| 用户希望的能力 | 已确认依据 | 结论边界 |
| --- | --- | --- |
| 同一网关开放 OpenAI、Anthropic、Gemini 接口 | 官方分别提供完整 Proxy 的 Chat／Responses、Messages、generateContent 端点及客户端示例 | 可由下游选用端点；实际模型及转换组合分别配置和验收 |
| Anthropic 客户端调用 Anthropic 上游，并流式返回 | Messages 文档明确列出 Streaming，并提供 Anthropic 流式示例 | 支持能力已有；工具增量、思考签名、缓存计量和错误终态仍需测试 |
| OpenAI 客户端调用 Anthropic 上游，并流式返回 | Anthropic 适配及 Responses 文档有转换与流式示例 | 分别返回 OpenAI Chat 或 Responses 格式，不保证与原生 Anthropic 的所有功能一一对应 |
| Anthropic 原生透传 | 官方另有 Anthropic Passthrough，列出原生格式、流式、日志和成本追踪 | 与统一 Messages 路由区分评估；不自动假定全部路由、风控和预算能力完全相同 |

来源：[LiteLLM 端点目录](https://docs.litellm.ai/docs/supported_endpoints)、[Anthropic Messages 与流式](https://docs.litellm.ai/docs/anthropic_unified/)、[Anthropic 模型适配](https://docs.litellm.ai/docs/providers/anthropic)、[Responses 的 Anthropic 流式示例](https://docs.litellm.ai/docs/response_api)、[Gemini Proxy](https://docs.litellm.ai/docs/generateContent)、[Anthropic 原生透传](https://docs.litellm.ai/docs/pass_through/anthropic_completion)。

与本项目实时风控相关的已知限制：Messages 文档说明流式输出可被 guardrail 阻断，但脱敏等文本改写仅适用于非流式输出；不能将“支持流式”解释为“已经支持边流式边改写所有内容”。我们的测试应分别覆盖流式拦截和脱敏策略。来源：[Messages 风控边界](https://docs.litellm.ai/docs/anthropic_unified/)。

当前无需再次选择单一协议，也无需为了 Anthropic 流式自行从零编写代理。后续用固定版本的 LiteLLM 与三家官方 SDK 验证同一部署下的协议并存，再核对流式事件保真、取消／断流和治理链路。
