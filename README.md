# DevMan ⚙️

**AI 的认知工作管理系统 — 外部大脑 + 项目经理 + 质检员**

简要说明、状态与快速上手指南。

---

## 项目概览 💡

DevMan 是一个面向 AI 的工作管理与质量保证基础设施，目标是把 AI 的产出与决策结构化、可复用并且可质检。

核心特性
- 结构化的 **Goal / Project / Phase / Task / WorkRecord** 数据模型
- Git + JSON 的轻量存储后端 (`GitJsonStorage`)
- 可扩展的 **QualityEngine**（支持通用检查和自定义检查）
- 基本的 **KnowledgeService** 和 **ProgressTracker** 实现
- 内置工具执行（`cargo`、`npm`、`git`、`fs`）和 CLI 用法
- MCP Server 框架（目前为占位实现）

---

## 当前状态 ✅

（详情请参阅 `docs/TODO.md`）

- 核心数据模型：已实现
- 存储（Git+JSON）：已实现（支持保存/加载、commit/rollback）
- 质量引擎：已实现基础能力（编译/测试/格式/lint/文档 + 自定义命令）
- 知识服务：实现了基础检索与推荐
- 进度追踪 / 工作管理 / 工具集成 / CLI：已实现基础功能
- AI 接口：`AIInterface` 已有 trait，MCP Server 为基本占位（需完善协议）

---

## 快速上手 🚀

### 构建

在仓库根目录运行：

```bash
# 构建所有 crate
cargo build --workspace

# 或运行 CLI（示例）
cargo run -p devman-cli -- CreateGoal "My goal" "描述"
cargo run -p devman-cli -- ListGoals
cargo run -p devman-cli -- ShowGoal <goal-id>
```

### 使用 `devman-cli` 示例

- 创建目标
  - `cargo run -p devman-cli -- CreateGoal "Title" "Description"`
- 列出目标
  - `cargo run -p devman-cli -- ListGoals`
- 查看目标详情
  - `cargo run -p devman-cli -- ShowGoal <goal-id>`

> 注：CLI 使用 `GitJsonStorage`（默认目录 `.devman/`）保存数据。

### 运行 MCP Server（占位）

MCP Server 当前为基础实现并监听退出信号：

```bash
# 运行 MCP Server（占位）
cargo run -p devman-ai
```

（MCP 协议层与 stdio/mcp transport 尚未完全实现，欢迎贡献）

---

## 开发与测试 🔧

- 单元测试：`cargo test --all`
- 代码审查与质量：已有质量检查模型与执行框架，可扩展更多检查器
- 本地存储目录：`.devman/`（Git 仓库）

---

## 代码结构（概览） 📁

- `crates/core` - 核心数据模型（Goal/Project/Phase/Task/Quality/...）
- `crates/storage` - 存储后端（`GitJsonStorage`）
- `crates/quality` - 质量引擎与检查
- `crates/knowledge` - 知识服务
- `crates/progress` - 进度追踪
- `crates/work` - 工作管理/执行
- `crates/tools` - 工具抽象与内置具体实现
- `crates/ai` - AI 接口（trait + MCP server 占位）
- `crates/cli` - 命令行工具

---

## 如何贡献 ✨

1. 新增或修改功能前先在 `docs/TODO.md` 提交需求或计划
2. 创建 feature branch 并提交 PR
3. 保持跨 crate 的接口兼容（增加 trait、实现等）

---

## 反馈与下一步 💬

目前最需要的工作：
- 完善 MCP / AI 协议实现
- 完善输出解析与自定义质检验证流程
- 增强知识检索（语义/相似度）与模板系统
- 补充更多测试与文档（`API.md`, `QUALITY_GUIDE.md`）

欢迎提交 issue 或 PR！

---

最后更新：2026-01-29
