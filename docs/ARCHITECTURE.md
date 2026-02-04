# DevMan 架构详解

> AI 认知工作管理系统 - 架构设计文档

## 目录

- [概述](#概述)
- [整体架构](#整体架构)
- [五层模型](#五层模型)
- [crate 结构](#crate-结构)
- [数据流](#数据流)
- [关键设计决策](#关键设计决策)
- [扩展点](#扩展点)
- [性能考虑](#性能考虑)

---

## 概述

DevMan 是一个面向 AI 的认知工作管理系统，旨在将 AI 的产出与决策结构化、可复用并且可质检。

### 设计目标

1. **认知存储与复用** - 减少 AI 的重复思考
2. **进度可视化** - 工作透明化
3. **质量保证** - 自动化 + 人工质检
4. **Token 优化** - 工具化稳定操作
5. **可追溯性** - 完整工作日志

### 核心价值

```
不是：AI 执行任务的平台
而是：AI 的外部认知和工程管理基础设施
```

---

## 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DevMan 系统架构                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐              │
│   │   CLI       │   │   MCP       │   │   AI        │              │
│   │   Frontend  │   │   Server    │   │   Interface │              │
│   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘              │
│          │                 │                 │                      │
│          └────────────────┼─────────────────┘                      │
│                           ▼                                        │
│              ┌─────────────────────────┐                           │
│              │      Work Manager       │                           │
│              │   (任务执行与编排)       │                           │
│              └────────────┬────────────┘                           │
│                           │                                        │
│     ┌─────────────────────┼─────────────────────┐                  │
│     ▼                     ▼                     ▼                  │
│  ┌──────────┐       ┌──────────┐       ┌──────────┐               │
│  │ Quality  │       │Progress  │       │Knowledge │               │
│  │ Engine   │       │ Tracker  │       │ Service  │               │
│  └────┬─────┘       └────┬─────┘       └────┬─────┘               │
│       │                  │                  │                      │
│       └──────────────────┼──────────────────┘                      │
│                          ▼                                         │
│              ┌─────────────────────────┐                           │
│              │      Tools              │                           │
│              │   (cargo, git, npm...)  │                           │
│              └────────────┬────────────┘                           │
│                           │                                        │
│                           ▼                                        │
│              ┌─────────────────────────┐                           │
│              │      Storage            │                           │
│              │   (JsonStorage/         │                           │
│              │    SqliteStorage)       │                           │
│              └─────────────────────────┘                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 五层模型

DevMan 采用五层架构设计，每层有明确的职责：

```
Layer 5: Knowledge Service    (知识检索与复用)
    ↑
    │  提供上下文推荐、模板系统、相似度匹配
    │
Layer 4: Quality Assurance     (质量检验)
    ↑
    │  自动化检查、人机协作、质量门
    │
Layer 3: Progress Tracking     (进度管理)
    ↑
    │  目标进度、阻塞检测、时间预估
    │
Layer 2: Work Management       (工作执行)
    ↑
    │  任务管理、上下文管理、事件记录
    │
Layer 1: Storage & State       (存储与状态)
    ↑
    │  数据持久化、版本管理、查询接口
    │
```

### Layer 1: 存储与状态 (Storage)

**职责**: 数据持久化和版本管理

**组件**:
- `Storage` trait - 存储抽象接口
- `JsonStorage` - 文件式 JSON 存储实现（默认）
- `SqliteStorage` - SQLite 存储实现（推荐生产使用）

**存储后端对比**:

| 特性 | JsonStorage | SqliteStorage |
|------|-------------|---------------|
| 部署方式 | 文件夹 | 单文件 |
| 查询能力 | O(n) 遍历 | SQL 查询 |
| 性能 | 一般 | 优秀 |
| 依赖 | 无 | sqlx |
| 推荐场景 | 开发/小型项目 | 生产/大数据量 |

**切换存储后端**:
```rust
// JSON 存储（默认）
let storage = JsonStorage::new(".devman").await?;

// SQLite 存储（推荐生产使用）
let storage = SqliteStorage::new(".devman/devman.db").await?;
```

**目录结构** (JsonStorage):
```
.devman/
├── goals/           # 目标数据
├── projects/        # 项目数据
├── phases/          # 阶段数据
├── tasks/           # 任务数据
├── events/          # 事件数据
├── knowledge/       # 知识数据
├── quality/         # 质检数据
├── work_records/    # 工作记录
├── embeddings/      # 向量索引（如果启用向量搜索）
└── meta/            # 元数据版本标记
    ├── goals/
    ├── projects/
    └── ...
```

**关键特性**:
- 特征门控: `json` (默认) 或 `sqlite` feature
- 元数据版本标记（仅保存版本号和时间戳）
- 完整的快照由项目 Git 仓库管理

### Layer 2: 工作管理 (Work)

**职责**: 任务执行、上下文管理、事件记录

**组件**:
- `WorkManager` - 工作管理器
- `WorkExecutor` - 工作执行器
- `WorkContext` - 工作上下文

**任务状态机**:
```
┌─────────┐
│ Pending │ ←──────────────────────────────┐
└────┬────┘                                │
     │ 执行                                │ 重新激活
     ▼                                     │
┌─────────┐    放弃                    ┌─────────┐
│ InProgress ───────────────────────► │ Abandoned│
└────┬────┘                           └─────────┘
     │ 完成
     ▼
┌─────────┐
│ Completed│
└─────────┘
```

**关键概念**:
- `TaskIntent` - 任务意图（Implementation, Investigation, Documentation 等）
- `ExecutionStep` - 执行步骤（工具调用、工作流）
- `StateTransition` - 状态转换验证

### Layer 3: 进度追踪 (Progress)

**职责**: 目标进度计算、阻塞检测、时间预估

**组件**:
- `ProgressTracker` - 进度追踪器
- `BlockerDetector` - 阻塞检测器
- `TimeEstimator` - 时间预估器

**阻塞检测**:
```
检测类型:
├── 依赖关系阻塞 (Task A 等待 Task B)
├── 循环依赖检测 (A → B → C → A)
├── 资源阻塞 (缺少依赖项)
└── 决策阻塞 (等待人工决策)
```

**时间预估**:
```rust
struct TimeEstimate {
    estimated_minutes: u32,    // 预估时间（分钟）
    confidence: f32,           // 置信度 (0-1)
    complexity: ComplexityLevel, // 复杂度分级
    factors: Vec<String>,      // 影响因素
    historical_base: Option<u32>, // 历史基准
}
```

### Layer 4: 质量保证 (Quality)

**职责**: 自动化检查、人机协作、质量门

**组件**:
- `QualityEngine` - 质量引擎
- `QualityCheckRegistry` - 检查器注册表
- `HumanReviewService` - 人工审核服务

**架构**:
```
┌─────────────────────────────────────────┐
│           QualityProfile                │  ← 质检配置集合
├─────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │ Phase   │  │ Phase   │  │ Phase   │ │  ← 阶段质量门
│  │ Gate 1  │  │ Gate 2  │  │ Gate 3  │ │
│  └────┬────┘  └────┬────┘  └────┬────┘ │
│       │            │            │       │
│       ▼            ▼            ▼       │
│  ┌─────────────────────────────────┐    │
│  │      QualityCheck               │    │  ← 质量检查
│  │  ├─ Generic (内置)              │    │
│  │  └─ Custom (自定义)             │    │
│  └─────────────────────────────────┘    │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐    │
│  │      OutputParser               │    │  ← 输出解析
│  │  ├─ Regex                      │    │
│  │  ├─ JsonPath                   │    │
│  │  └─ LineContains               │    │
│  └─────────────────────────────────┘    │
├─────────────────────────────────────────┤
│  ┌─────────────────────────────────┐    │
│  │      HumanReviewService         │    │  ← 人机协作
│  │  ├─ Slack/Email/Webhook         │    │
│  │  └─ Review Form                 │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

**质量门策略**:
- `AllMustPass` - 所有检查必须通过
- `WarningsAllowed { max_warnings }` - 允许警告
- `ManualDecision` - 需要人工决策

### Layer 5: 知识服务 (Knowledge)

**职责**: 知识存储、检索、模板管理

**组件**:
- `KnowledgeService` - 知识服务
- `TemplateRegistry` - 模板注册表
- `KnowledgeClassifier` - 知识分类器
- `VectorKnowledgeService` - 向量检索服务（可选）
- `RerankerService` - 重排序服务（可选）

**知识类型**:
```
KnowledgeType:
├── LessonLearned    # 经验教训
├── BestPractice     # 最佳实践
├── CodePattern      # 代码模式
├── Solution         # 解决方案
├── Template         # 模板
└── Decision         # 决策
```

**检索架构**:
```
Query → 向量检索 (Top 50) → Reranker 重排序 → Top 10
```

**检索能力**:
- 关键词搜索（TF-IDF 简化版）
- 标签检索（OR/AND 逻辑）
- 上下文推荐
- 相似度匹配
- 按类型筛选
- **向量检索**（可选）：语义搜索，支持 Ollama 本地模型
- **Reranker 重排序**（可选）：两阶段检索，提升相关性

**向量搜索配置**:
```rust
let config = VectorSearchConfig {
    enabled: true,
    model: EmbeddingModel::Qwen3Embedding0_6B,  // 或 OpenAI
    ollama_url: "http://localhost:11434",
    dimension: 1024,
    threshold: 0.75,  // 相似度阈值
};

let vector_service = VectorKnowledgeServiceImpl::new(storage.clone(), config);
vector_service.initialize().await?;
```

**Reranker 配置**:
```rust
let reranker_config = RerankerConfig {
    enabled: true,
    model: RerankerModel::Qwen3Reranker0_6B,  // Ollama 本地模型
    ollama_url: "http://localhost:11434",
    max_candidates: 50,   // 向量检索候选数
    final_top_k: 10,      // 最终返回数
};
```

---

## crate 结构

```
DevMan/
├── Cargo.toml              # 工作空间配置
│
├── crates/
│   ├── core/              # 核心数据模型
│   │   ├── goal.rs        # Goal 数据模型
│   │   ├── project.rs     # Project 数据模型
│   │   ├── phase.rs       # Phase 数据模型
│   │   ├── task.rs        # Task 数据模型
│   │   ├── work_record.rs # WorkRecord 数据模型
│   │   ├── event.rs       # Event 数据模型
│   │   ├── knowledge.rs   # Knowledge 数据模型
│   │   ├── quality.rs     # Quality 数据模型
│   │   ├── id.rs          # ID 类型定义
│   │   └── lib.rs         # 模块入口
│   │
│   ├── storage/           # 存储层
│   │   ├── trait_.rs      # Storage trait
│   │   ├── json_storage.rs # JsonStorage 实现
│   │   ├── sqlite_storage.rs # SqliteStorage 实现 (可选)
│   │   └── lib.rs
│   │
│   ├── work/              # 工作管理 (Layer 2)
│   │   ├── manager.rs     # WorkManager
│   │   ├── executor.rs    # WorkExecutor
│   │   ├── context.rs     # WorkContext
│   │   └── lib.rs
│   │
│   ├── progress/          # 进度追踪 (Layer 3)
│   │   ├── tracker.rs     # ProgressTracker
│   │   ├── estimator.rs   # TimeEstimator
│   │   ├── blocker.rs     # BlockerDetector
│   │   └── lib.rs
│   │
│   ├── quality/           # 质量保证 (Layer 4)
│   │   ├── engine.rs      # QualityEngine
│   │   ├── checks.rs      # GenericCheckType
│   │   ├── custom.rs      # CustomCheckBuilder
│   │   ├── registry.rs    # QualityCheckRegistry
│   │   ├── gate.rs        # QualityGate
│   │   ├── human.rs       # HumanReviewService
│   │   ├── parser.rs      # OutputParser
│   │   └── lib.rs
│   │
│   ├── knowledge/         # 知识服务 (Layer 5)
│   │   ├── service.rs     # KnowledgeService
│   │   ├── template.rs    # TemplateRegistry
│   │   ├── search.rs      # KnowledgeSearch
│   │   ├── classification.rs # KnowledgeClassifier
│   │   ├── vector.rs      # VectorKnowledgeService (可选)
│   │   ├── reranker.rs    # RerankerService (可选)
│   │   └── lib.rs
│   │
│   ├── tools/             # 工具集成
│   │   ├── trait.rs       # Tool trait
│   │   ├── builtin.rs     # BuiltinToolExecutor
│   │   ├── workflow.rs    # Workflow
│   │   └── lib.rs
│   │
│   ├── ai/                # AI 接口
│   │   ├── interface.rs   # AIInterface trait
│   │   ├── interactive.rs # InteractiveAI trait
│   │   ├── mcp_server.rs  # MCP Server 实现
│   │   └── lib.rs
│   │
│   └── cli/               # 命令行工具
│       └── main.rs
│
└── docs/
    ├── DESIGN.md          # 设计方案
    ├── API.md             # API 参考
    ├── TODO.md            # 开发路线图
    ├── QUALITY_GUIDE.md   # 质检扩展指南
    ├── KNOWLEDGE.md       # 知识管理指南
    ├── ARCHITECTURE.md    # 本文档
    └── CONTRIBUTING.md    # 贡献指南
```

---

## 数据流

### 典型任务执行流程

```
1. AI 创建任务
   ┌─────────────────────────────────────┐
   │ WorkManager.create_task()          │
   │ - 生成 TaskId                      │
   │ - 初始化 Task 状态                 │
   │ - 创建关联的 QualityGate           │
   └─────────────────────────────────────┘
                     │
                     ▼
2. 知识检索（获取上下文）
   ┌─────────────────────────────────────┐
   │ KnowledgeService.search()          │
   │ - 搜索相关最佳实践                 │
   │ - 获取相似解决方案                 │
   │ - 返回推荐知识列表                 │
   └─────────────────────────────────────┘
                     │
                     ▼
3. 执行任务步骤
   ┌─────────────────────────────────────┐
   │ WorkExecutor.execute_steps()       │
   │ - 调用 ToolExecutor                │
   │ - 捕获输出和指标                   │
   │ - 记录 WorkRecord                  │
   └─────────────────────────────────────┘
                     │
                     ▼
4. 运行质量检查
   ┌─────────────────────────────────────┐
   │ QualityEngine.run_gate()           │
   │ - 依次执行 QualityCheck            │
   │ - 解析输出，评估条件               │
   │ - 生成 Finding 和 Metric           │
   └─────────────────────────────────────┘
                     │
                     ▼
5. 更新进度
   ┌─────────────────────────────────────┐
   │ ProgressTracker.update()           │
   │ - 计算完成百分比                   │
   │ - 检测阻塞情况                     │
   │ - 更新 TimeEstimate                │
   └─────────────────────────────────────┘
                     │
                     ▼
6. 记录知识（如果需要）
   ┌─────────────────────────────────────┐
   │ KnowledgeService.create()          │
   │ - 提取工作记录中的知识             │
   │ - 创建 Knowledge 条目              │
   │ - 建立关联关系                     │
   └─────────────────────────────────────┘
```

### AI 交互流程

```
┌──────────┐     ┌──────────────┐     ┌────────────────┐
│   AI     │────►│   MCP Server │────►│  WorkManager   │
│          │◄�────│              │◄────│                │
└──────────┘     └──────────────┘     └────────────────┘
     │                                       │
     │               ┌───────────────────────┘
     │               │
     │               ▼
     │     ┌────────────────────┐
     │     │ KnowledgeService  │  ◄── 上下文推荐
     │     └────────────────────┘
     │
     │               ┌───────────────────────┐
     └──────────────►│  QualityEngine        │  ◄── 质量检查
                    └───────────────────────┘
```

---

## 关键设计决策

### 1. 存储设计：文件式 JSON

**决策**: 使用纯文件式 JSON 存储，不包含自动 Git 集成

**原因**:
- 轻量级，无外部依赖
- 版本管理由项目自身 Git 负责
- 避免存储层复杂性

**权衡**:
- 优点: 简单、可移植、无依赖
- 缺点: 不支持复杂查询

### 2. 标识符：ULID

**决策**: 所有实体使用 ULID 作为唯一标识符

**原因**:
- 可排序（按创建时间）
- 全球唯一
- 字符串表示人类可读

**示例**:
```
01HX3V2F5F8Q8Y5Z5X5V5F5F8Q8
```

### 3. 异步优先

**决策**: 所有 I/O 操作使用 async/await

**原因**:
- 高并发支持
- 更好的资源利用
- 与 Tokio 生态集成

**实现**:
```rust
#[async_trait]
trait Storage {
    async fn load_goal(&self, id: GoalId) -> Result<Option<Goal>>;
    async fn save_goal(&self, goal: &Goal) -> Result<()>;
}
```

### 4. 质量检查可扩展

**决策**: 支持内置 + 自定义双模式检查

**原因**:
- 内置检查覆盖常见场景
- 自定义检查支持业务特定需求
- 输出解析器支持多种格式

**实现**:
```rust
enum QualityCheckType {
    Generic(GenericCheckType),  // 内置
    Custom(CustomCheckSpec),    // 自定义
}
```

### 5. 人机协作

**决策**: 关键检查支持人工审核

**原因**:
- 自动化无法解决所有问题
- 业务规则需要人工判断
- 渐进式自动化策略

**流程**:
```
自动检查 → 需要人工? → 发送通知 → 人工审核 → 记录结果
```

---

## 扩展点

### 1. 新增存储后端

实现 `Storage` trait：

```rust
#[async_trait]
trait Storage: Send + Sync {
    async fn load_goal(&self, id: GoalId) -> Result<Option<Goal>>;
    async fn save_goal(&self, goal: &Goal) -> Result<()>;
    // ... 其他方法
}
```

### 2. 新增工具

实现 `Tool` trait：

```rust
#[async_trait]
trait Tool: Send + Sync {
    fn name(&self) -> &str;
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput>;
}
```

### 3. 新增检查器

使用 `CustomCheckBuilder`：

```rust
let check = CustomCheckBuilder::new("my-check")
    .command("my-tool")
    .arg("--analyze")
    .output_parser(OutputParser::Regex { pattern: "...".to_string() })
    .pass_condition("score >= 80")
    .build();
```

### 4. 新增通知渠道

扩展 `NotificationChannel` 枚举：

```rust
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook: String },
    Webhook { url: String },
    DingTalk { webhook: String },  // 新增
    // ...
}
```

### 5. 新增知识类型

扩展 `KnowledgeType` 枚举：

```rust
pub enum KnowledgeType {
    LessonLearned { lesson: String, context: String },
    BestPractice { practice: String, rationale: String },
    CodePattern { pattern: CodeSnippet, usage: String },
    Solution { problem: String, solution: String, verified: bool },
    Template { template: TemplateContent, 适用场景: Vec<String> },
    Decision { decision: String, alternatives: Vec<String>, reasoning: String },
    StandardProcess { name: String, steps: Vec<String> },  // 新增
}
```

---

## 性能考虑

### 1. 异步并发

```rust
// 并行执行多个检查
let results = futures::future::join_all(
    checks.iter().map(|check| engine.run_check(check, &context))
).await;
```

### 2. 知识检索优化

- 标签索引
- 关键词缓存
- 相似度计算的早期终止
- 向量索引缓存（避免重复计算 embedding）
- Reranker 批处理优化

### 3. 向量检索优化

```rust
// 批量生成 embedding
let embeddings = model.encode_batch(&texts).await?;

// 本地向量索引（余弦相似度）
let index = LocalVectorIndex::new(dimension);
index.add_all(embeddings)?;

// 搜索
let results = index.search(&query_embedding, top_k)?;
```

### 3. 存储批量操作

```rust
// 批量保存
async fn save_all(&self, goals: &[Goal]) -> Result<()> {
    for goal in goals {
        self.save_goal(goal).await?;
    }
    Ok(())
}
```

### 4. 超时控制

```rust
// 检查器执行超时
timeout(Duration::from_secs(300), engine.run_check(check, &context))
```

---

## 依赖关系图

```
devman-cli
    │
    └── devman-core
    │       │
    │       └── devman-storage (依赖)
    │       └── devman-quality (依赖)
    │       └── devman-knowledge (依赖)
    │
    └── devman-storage
    │
    └── devman-quality
            │
            └── devman-core (依赖)
            └── devman-tools (依赖)
            └── devman-storage (依赖)
```

---

## 部署架构

### 开发环境

```
┌─────────────────┐
│   cargo run     │  ← 直接运行 CLI
│   devman-cli    │
└─────────────────┘
```

### 生产环境

```
┌─────────────────────────────────────┐
│         二进制部署                   │
│  ┌─────────┐  ┌─────────────────┐  │
│  │ devman  │  │ devman-ai       │  │  ← MCP Server
│  │  cli    │  │  (stdio/socket) │  │
│  └─────────┘  └─────────────────┘  │
│         │              │            │
│         ▼              ▼            │
│  ┌─────────────────────────────────┤
│  │         .devman/               │  ← 本地存储
│  └─────────────────────────────────┤
└─────────────────────────────────────┘
```

### MCP Server 部署

```bash
# 启动 MCP Server
devman-ai --transport stdio    # stdio 传输
devman-ai --transport socket --port 3000  # socket 传输
```

---

*最后更新: 2026-02-04*
