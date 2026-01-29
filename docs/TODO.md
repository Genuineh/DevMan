# DevMan 开发规划 v2

> AI 的认知工作管理系统 - 外部大脑 + 项目经理 + 质检员

## 项目定位

```
不是：AI 执行任务的平台
而是：AI 的外部认知和工程管理基础设施

核心价值：
├── 认知存储与复用（减少重复思考）
├── 进度可视化（工作透明化）
├── 质量保证（自动化 + 人工质检）
├── Token 优化（工具化稳定操作）
└── 可追溯性（完整工作日志）
```

---

## 核心架构（五层模型）

```
Layer 5: Knowledge Service    (知识检索与复用)
Layer 4: Quality Assurance     (质量检验)
Layer 3: Progress Tracking     (进度管理)
Layer 2: Work Management       (工作执行)
Layer 1: Storage & State       (存储与状态)
```

---

## 开发路线图

### Phase 1：核心数据模型重构
- [x] 项目初始化
- [ ] 重构核心数据结构
  - [ ] `Goal` - 顶层目标，带成功标准和进度
  - [ ] `Project` - 工程上下文和配置
  - [ ] `Phase` - 阶段划分和验收标准
  - [ ] `Task` - 保留并增强（添加质量门、执行步骤）
  - [ ] `WorkRecord` - 详细工作日志
  - [ ] `Knowledge` - 多类型知识资产
  - [ ] `QualityCheck` - 通用 + 业务质检

### Phase 2：存储层
- [ ] 扩展 `Storage` trait（支持新模型）
- [ ] 更新 `GitJsonStorage` 实现
- [ ] 事务支持
- [ ] 查询接口优化

### Phase 3：质量保证（核心）
- [ ] `QualityEngine` trait
- [ ] 通用质检实现
  - [ ] 编译检查
  - [ ] 测试检查
  - [ ] 格式检查
  - [ ] Lint 检查
  - [ ] 文档检查
- [ ] 业务质检扩展机制
  - [ ] `CustomCheckSpec` 设计
  - [ ] 命令执行
  - [ ] 输出解析
- [ ] 人机协作接口
  - [ ] `HumanReviewSpec`
  - [ ] 通知机制
  - [ ] 评审表单
- [ ] 质检编排
  - [ ] `QualityProfile`
  - [ ] `QualityGate`
  - [ ] 策略配置

### Phase 4：知识服务
- [ ] `KnowledgeService` trait
- [ ] 知识检索
  - [ ] 标签检索
  - [ ] 相似度匹配
  - [ ] 上下文推荐
- [ ] 知识模板
  - [ ] 参数化模板
  - [ ] 模板实例化
- [ ] 知识分类
  - [ ] 经验教训
  - [ ] 最佳实践
  - [ ] 代码模式
  - [ ] 解决方案

### Phase 5：进度追踪
- [ ] `ProgressTracker` trait
- [ ] 目标进度计算
- [ ] 阶段里程碑追踪
- [ ] 阻塞检测
- [ ] 完成时间预估

### Phase 6：工作管理
- [ ] `WorkManager` trait
- [ ] 任务创建和执行
- [ ] 上下文管理
- [ ] 事件记录
- [ ] 工作记录生成

### Phase 7：工具集成
- [ ] `Tool` trait
- [ ] 内置工具
  - [ ] Cargo
  - [ ] Npm
  - [ ] Git
  - [ ] 文件系统
- [ ] 工作流编排
- [ ] 错误处理策略

### Phase 8：AI 接口
- [ ] `AIInterface` trait
- [ ] 高层 API 设计
- [ ] MCP Server 实现
- [ ] CLI 更新

---

## Crate 结构（重构后）

```
devman/
├── Cargo.toml
├── crates/
│   ├── core/                    # 核心数据模型
│   │   ├── goal.rs
│   │   ├── project.rs
│   │   ├── phase.rs
│   │   ├── task.rs
│   │   ├── work_record.rs
│   │   ├── knowledge.rs
│   │   ├── quality.rs
│   │   └── lib.rs
│   │
│   ├── storage/                 # 存储层
│   │   ├── trait.rs
│   │   ├── git_json.rs
│   │   └── lib.rs
│   │
│   ├── knowledge/               # 知识服务 (Layer 5)
│   │   ├── service.rs
│   │   ├── search.rs
│   │   ├── template.rs
│   │   └── lib.rs
│   │
│   ├── quality/                 # 质量保证 (Layer 4)
│   │   ├── engine.rs
│   │   ├── checks.rs          # 通用质检
│   │   ├── custom.rs          # 业务质检扩展
│   │   ├── registry.rs
│   │   ├── human_review.rs
│   │   └── lib.rs
│   │
│   ├── progress/                # 进度追踪 (Layer 3)
│   │   ├── tracker.rs
│   │   ├── estimator.rs
│   │   ├── blocker.rs
│   │   └── lib.rs
│   │
│   ├── work/                    # 工作管理 (Layer 2)
│   │   ├── manager.rs
│   │   ├── executor.rs
│   │   ├── context.rs
│   │   └── lib.rs
│   │
│   ├── tools/                   # 工具集成
│   │   ├── trait.rs
│   │   ├── builtin.rs
│   │   └── lib.rs
│   │
│   ├── ai/                      # AI 接口
│   │   ├── interface.rs
│   │   └── mcp_server.rs
│   │
│   └── cli/                     # 命令行
│       └── main.rs
│
└── docs/
    ├── DESIGN.md
    ├── TODO.md
    ├── API.md
    └── QUALITY_GUIDE.md
```

---

## 核心数据模型概要

### Goal（目标）
```rust
struct Goal {
    id: GoalId,
    title: String,
    success_criteria: Vec<SuccessCriterion>,
    progress: GoalProgress,
    current_phase: PhaseId,
    status: GoalStatus,
}
```

### Project（项目）
```rust
struct Project {
    id: ProjectId,
    name: String,
    config: ProjectConfig,  // 技术栈、目录结构、质检配置
    phases: Vec<Phase>,
    current_phase: PhaseId,
}
```

### QualityCheck（质检）
```rust
enum QualityCheckType {
    Generic(GenericCheckType),  // 通用（编译、测试...）
    Custom(CustomCheckSpec),    // 业务（用户扩展）
}

struct HumanReviewSpec {        // 人机协作
    reviewers: Vec<String>,
    review_guide: String,
    review_form: Vec<ReviewQuestion>,
}
```

### Knowledge（知识）
```rust
enum KnowledgeType {
    LessonLearned,
    BestPractice,
    CodePattern,
    Solution,
    Template,
    Decision,
}
```

---

## 质检扩展机制

### 通用质检（内置）
- 编译检查
- 测试检查（支持覆盖率）
- 格式检查
- Lint 检查
- 文档检查
- 类型检查
- 安全扫描

### 业务质检（用户扩展）
```rust
struct CustomCheckSpec {
    name: String,
    check_command: CommandSpec,
    validation: ValidationSpec,
    human_review: Option<HumanReviewSpec>,
}
```

### 人机协作流程
```
1. 系统运行自动质检
2. 发现需要人工判断的问题
3. 发送通知（Slack/Email/Webhook）
4. 业务人员评审（填写表单）
5. 系统记录结果，更新知识
```

---

## AI 使用流程

### 场景：AI 开始新项目
```
1. AI: "创建一个 Rust Web 框架"
2. 系统：创建 Goal + Project + Phase
3. 系统：初始化 QualityProfile
4. 系统：返回 WorkContext
```

### 场景：AI 执行任务
```
1. AI: "实现 HTTP 路由"
2. 系统：查询知识库，返回最佳实践
3. AI：创建 Task，定义 ExecutionStep
4. 系统：执行工具（cargo, git 等）
5. 系统：自动运行质检
6. 系统：记录 WorkRecord
```

---

## 优先级（按依赖顺序）

### P0 - 基础结构
1. 核心数据模型（Goal, Project, Phase, Task, WorkRecord）
2. 存储层更新
3. CLI 基础命令

### P1 - 质量保证
1. QualityEngine
2. 通用质检实现
3. 业务质检扩展机制
4. 质检编排

### P2 - 知识服务
1. 知识存储和检索
2. 模板系统
3. 相似度匹配

### P3 - 进度与工作管理
1. ProgressTracker
2. WorkManager
3. 上下文管理

### P4 - 工具与 AI 接口
1. Tool trait
2. 内置工具
3. AIInterface
4. MCP Server

---

## 设计原则（更新）

1. **质检可扩展** - 通用 + 业务 + 人机协作
2. **知识可复用** - 检索、模板、推荐
3. **工具化执行** - 减少_token_消耗
4. **进度可视化** - 目标 → 阶段 → 任务
5. **存储抽象** - 可替换存储后端
6. **AI 友好** - 结构化接口
7. **可追溯性** - 完整工作日志

---

## 文档计划

- [x] DESIGN.md - 完整设计方案
- [ ] API.md - API 参考
- [ ] QUALITY_GUIDE.md - 质检扩展指南
- [ ] KNOWLEDGE.md - 知识管理指南
- [ ] ARCHITECTURE.md - 架构详解
- [ ] CONTRIBUTING.md - 贡献指南

---

## 当前进度

- [x] 项目规格定义（v1）
- [x] 设计方案 v2
- [x] Rust 项目初始化
- [ ] 核心数据模型重构
- [ ] 存储层更新
- [ ] 质量保证实现
- [ ] 知识服务实现
- [ ] 进度追踪实现
- [ ] 工作管理实现
- [ ] 工具集成实现
- [ ] AI 接口实现

---

*最后更新: 2026-01-29*
