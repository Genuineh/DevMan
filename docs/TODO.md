# DevMan 开发规划

> AI 的执行认知框架 - 长期计划系统

## 项目定位

DevMan 不是任务管理器，而是 **AI 的执行认知框架**。

```
Goal → Plan → Action → Observation → Reflection → Goal Update
```

闭环，支持：
- 长期目标追踪
- 假设驱动的探索
- 自动反思与学习
- 自进化能力

---

## 核心架构（四层模型）

```
Layer 4: Evolution Layer     ← 自我改进
Layer 3: Reflection Layer    ← 复盘与学习
Layer 2: Execution Layer     ← 执行与调度
Layer 1: Memory Layer        ← 长期记忆
```

---

## 开发路线图

### 阶段 0：项目初始化
- [ ] 初始化 Rust 项目 (`cargo init`)
- [ ] 创建 workspace 结构
- [ ] 配置基础依赖 (tokio, serde, clap)
- [ ] 设置 pre-commit hooks

### 阶段 1：核心数据模型 (Layer 1)
- [ ] 定义核心数据结构
  - [ ] `Task` - 任务实体
  - [ ] `Event` - 时间线原子
  - [ ] `KnowledgeNode` - 认知图谱节点
  - [ ] `TaskLink` - 任务关联关系
- [ ] 实现状态机
  - [ ] IDEA → QUEUED → ACTIVE → REVIEW → DONE
  - [ ] ACTIVE → BLOCKED → ABANDONED 分支
- [ ] 序列化/反序列化 (serde)

### 阶段 2：存储抽象层
- [ ] 定义 `Storage` trait
  ```rust
  trait Storage {
      fn save_task(&mut self, task: &Task) -> Result<()>;
      fn load_task(&self, id: TaskId) -> Result<Option<Task>>;
      fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>>;
      fn save_event(&mut self, event: &Event) -> Result<()>;
      // ...
  }
  ```
- [ ] 实现 `GitJsonStorage`
  - [ ] JSON 文件存储
  - [ ] Git commit 每次变更
  - [ ] 自动生成时间线
- [ ] 为未来 SQLite/GraphDB 预留接口

### 阶段 3：执行循环 (Layer 2)
- [ ] 实现 Task Selector
  - [ ] 可插拔选择策略
  - [ ] 默认：优先级 + 依赖解析
- [ ] 实现 Dependency Resolver
  - [ ] 依赖图构建
  - [ ] 循环依赖检测
  - [ ] 探索模式（允许绕过依赖）
- [ ] 实现 Resource Scheduler
  - [ ] 时间/计算预算控制
  - [ ] 并发任务管理
- [ ] 核心执行循环
  ```rust
  loop {
      task = select_task();
      result = execute(task);
      reflection = analyze(result);
      update_memory(reflection);
  }
  ```

### 阶段 4：反思系统 (Layer 3)
- [ ] 实现 Reflection Engine
  - [ ] Expected vs Actual 对比
  - [ ] 假设验证逻辑
  - [ ] 错误分类
- [ ] 知识更新机制
  - [ ] 置信度调整
  - [ ] 新知识节点生成
- [ ] 自增长任务树
  - [ ] 根据反思自动生成新任务
  - [ ] 任务拆分与合并

### 阶段 5：对外接口
- [ ] CLI 实现
  - [ ] `devman add` - 添加任务
  - [ ] `devman list` - 列出任务
  - [ ] `devman run` - 执行一步
  - [ ] `devman reflect` - 触发反思
  - [ ] `devman status` - 查看状态
- [ ] MCP Server 实现
  - [ ] MCP 协议适配
  - [ ] 工具注册 (add_task, execute, reflect, etc.)
  - [ ] Claude Code 集成测试

### 阶段 6：进化层 (Layer 4)
- [ ] 元数据收集
  - [ ] 任务失败率统计
  - [ ] 策略效率追踪
- [ ] 自适应调优
  - [ ] 优先级权重学习
  - [ ] 探索比例调整
- [ ] 目标进化机制
  - [ ] 目标升级/分裂/放弃

### 阶段 7：高级特性
- [ ] 多 Agent 协同
  - [ ] Planner / Executor / Critic / Archivist 角色
- [ ] 认知图可视化
- [ ] 时间线回溯
- [ ] 实验管理

---

## Crate 结构

```
devman/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── core/                     # 核心数据模型
│   │   ├── src/
│   │   │   ├── task.rs
│   │   │   ├── event.rs
│   │   │   ├── knowledge.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── storage/                  # 存储抽象与实现
│   │   ├── src/
│   │   │   ├── trait.rs
│   │   │   ├── git_json.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── execution/                # 执行调度
│   │   ├── src/
│   │   │   ├── selector.rs
│   │   │   ├── dependency.rs
│   │   │   ├── scheduler.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── reflection/               # 反思系统
│   │   ├── src/
│   │   │   ├── engine.rs
│   │   │   ├── analyzer.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── evolution/                # 进化层
│   │   ├── src/
│   │   │   ├── optimizer.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── cli/                      # 命令行
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── mcp/                      # MCP Server
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
└── docs/
    └── TODO.md                   # 本文档
```

---

## 核心数据模型

### Task
```rust
struct Task {
    id: TaskId,
    intent: String,              // 为什么存在
    hypothesis: String,          // 预期改变
    status: TaskStatus,
    confidence: f32,             // 0-1
    priority: u8,                // 0-255
    links: Vec<TaskLink>,        // 依赖/阻塞/相关
    logs: Vec<EventId>,          // 关联事件
    created_at: Time,
    updated_at: Time,
}

enum TaskStatus {
    Idea,
    Queued,
    Active,
    Blocked,
    Review,
    Done,
    Abandoned,
}

struct TaskLink {
    to: TaskId,
    kind: LinkKind,  // DependsOn, Blocks, RelatedTo, DerivesFrom
}
```

### Event
```rust
struct Event {
    id: EventId,
    timestamp: Time,
    actor: AgentId,
    action: String,
    result: String,
    delta_knowledge: Vec<KnowledgeUpdate>,
    related_tasks: Vec<TaskId>,
}
```

### KnowledgeNode
```rust
struct KnowledgeNode {
    id: NodeId,
    claim: String,
    confidence: f32,
    derived_from: Vec<NodeId>,
    evidence: Vec<EventId>,
    created_at: Time,
}
```

### ReflectionReport
```rust
struct ReflectionReport {
    task_id: TaskId,
    success: bool,
    insight: String,
    confidence_delta: f32,
    derived_tasks: Vec<Task>,
    knowledge_updates: Vec<KnowledgeNode>,
    generated_at: Time,
}
```

---

## 依赖选择

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4", features = ["derive"] }
git2 = "0.18"
chrono = { version = "0.4", features = ["serde"] }
ulid = "1"
```

---

## 开发优先级

### P0 - 第一版可运行 (MVP)
1. 核心数据模型
2. Git+JSON 存储
3. 基础执行循环
4. CLI 基本命令

### P1 - 完整功能
1. 反思系统
2. MCP Server
3. 依赖解析
4. 任务选择策略

### P2 - 高级特性
1. 进化层
2. 多 Agent
3. 可视化

---

## 设计原则

1. **存储抽象** - 所有存储操作通过 trait，可替换
2. **策略可插拔** - 任务选择、反思策略都是可替换的
3. **不可变优先** - Event 日志不可变，状态通过重放计算
4. **Git 友好** - 所有数据可读、可 merge
5. **AI 友好** - 结构化输出，易于解析

---

## 文档计划

- [ ] ARCHITECTURE.md - 整体架构说明
- [ ] DATA_MODEL.md - 数据模型详解
- [ ] STORAGE.md - 存储层设计
- [ ] REFLECTION.md - 反思机制
- [ ] MCP_INTEGRATION.md - MCP 集成指南
- [ ] CONTRIBUTING.md - 贡献指南

---

## 当前进度

- [x] 项目规格定义
- [ ] Rust 项目初始化
- [ ] 核心数据模型实现
- [ ] 存储层实现
- [ ] 执行循环实现
- [ ] CLI 完成
- [ ] MCP Server 完成
- [ ] 反思系统实现
- [ ] 进化层实现

---

*最后更新: 2026-01-29*
