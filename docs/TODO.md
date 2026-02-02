# DevMan 开发规划 v3

> AI 的认知工作管理系统 - 外部大脑 + 项目经理 + 质检员

> **v3 变更说明**: 移除存储层的 Git 功能，使用纯文件式 JSON 存储。版本管理由项目自身的 Git 仓库负责。

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

### Phase 1：核心数据模型 ✅
- [x] 项目初始化
- [x] 重构核心数据结构
  - [x] `Goal` - 顶层目标，带成功标准和进度
  - [x] `Project` - 工程上下文和配置
  - [x] `Phase` - 阶段划分和验收标准
  - [x] `Task` - 保留并增强（添加质量门、执行步骤）
  - [x] `WorkRecord` - 详细工作日志
  - [x] `Knowledge` - 多类型知识资产
  - [x] `QualityCheck` - 通用 + 业务质检
  - [x] `Event` - 事件系统

### Phase 2：存储层 ✅
- [x] 扩展 `Storage` trait（支持新模型）
- [x] 实现 `JsonStorage`（文件式 JSON 存储）
- [x] 元数据版本标记（meta.json 仅含版本号和时间戳）
- [x] 查询接口（按状态筛选任务、按关联查询等）

> **注意**: 移除了自动 Git 功能。全量快照由项目自身的 Git 仓库管理。

### Phase 3：质量保证（核心）✅
- [x] `QualityEngine` trait
- [x] 通用质检实现
  - [x] 编译检查
  - [x] 测试检查
  - [x] 格式检查
  - [x] Lint 检查
  - [x] 文档检查
- [x] 业务质检扩展机制
  - [x] `CustomCheckSpec` 设计
  - [x] 命令执行
  - [x] 输出解析（增强正则/JsonPath 解析）
- [x] 人机协作接口
  - [x] `HumanReviewSpec`
  - [x] 通知机制（Slack/Email/Webhook）
  - [x] 评审表单
- [x] 质检编排
  - [x] `QualityProfile`
  - [x] `QualityGate`
  - [x] 策略配置

### Phase 4：知识服务 ✅
- [x] `KnowledgeService` trait
- [x] 知识检索基础
  - [x] 上下文推荐
  - [x] 标签检索（OR/AND 逻辑）
  - [x] 标签统计和建议
  - [x] 相似度匹配（基于相关性评分）
  - [x] 按类型检索
- [x] 知识模板
  - [x] 参数化模板（TemplateBuilder）
  - [x] 模板注册表（TemplateRegistry）
  - [x] 模板实例化（支持参数替换）
  - [x] 模板验证（required/optional 参数）
- [x] 知识分类增强（无需向量库）
  - [x] 经验教训自动提取
  - [x] 代码模式识别
  - [x] 解决方案索引

### Phase 5：进度追踪 ✅
- [x] `ProgressTracker` trait
- [x] 目标进度计算
- [x] 阶段里程碑追踪
- [x] 阻塞检测（自动识别依赖关系）
  - [x] 依赖关系阻塞检测
  - [x] 循环依赖检测（DFS算法）
  - [x] 阻塞统计（按类型/严重程度）
  - [x] 解决建议生成
- [x] AI时间预估（分钟级精度 + 复杂度分级 + 置信度）

### Phase 6：工作管理 ✅
- [x] `WorkManager` trait
- [x] 任务创建和执行
- [x] 上下文管理
- [x] 事件记录
- [x] 工作记录生成

### Phase 7：工具集成 ✅
- [x] `Tool` trait
- [x] 内置工具
  - [x] Cargo
  - [x] Npm
  - [x] Git
  - [x] 文件系统
- [x] 工作流编排（多步骤流程定义）
- [x] 错误处理策略（重试、回滚、降级）

### Phase 8：AI 接口 ✅ 完成
- [x] `AIInterface` trait
- [x] 交互式任务管理系统
  - [x] 任务状态机实现（10状态 + AbandonReason）
  - [x] 状态转换校验（StateTransition + 负反馈）
  - [x] 任务引导逻辑（TaskGuidanceGenerator）
  - [x] 负反馈机制（通过 StateTransition::RejectedRequiredAction）
- [x] 任务控制功能
  - [x] 暂停/恢复任务（InteractiveAI trait 已定义）
  - [x] 放弃任务（统一处理）（InteractiveAI trait 已定义）
  - [x] 需求变更处理（InteractiveAI trait 已定义）
  - [x] 任务重新分配（InteractiveAI trait 已定义）
- [x] MCP Server 实现
  - [x] MCP Tool 注册（11个内置工具）
  - [x] MCP Resources（4个内置资源）
  - [x] stdio 传输
  - [x] Unix socket 传输
- [x] CLI 更新
- [x] AI 模块测试（59个测试用例）

---

## Crate 结构

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
│   │   ├── event.rs
│   │   ├── knowledge.rs
│   │   ├── quality.rs
│   │   └── lib.rs
│   │
│   ├── storage/                 # 存储层
│   │   ├── trait_.rs
│   │   ├── json_storage.rs
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
│   │   ├── interface.rs          # 基础 AI 接口
│   │   ├── interactive.rs       # 交互式 AI trait + BasicInteractiveAI
│   │   ├── validation.rs        # 状态转换验证
│   │   ├── guidance.rs          # 任务引导生成
│   │   └── mcp_server.rs        # MCP 服务器实现
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

## 存储层说明（v3 更新）

### JsonStorage 设计

文件式 JSON 存储，不包含 Git 集成：

```
.devman/
├── goals/           # 目标数据（JSON）
├── projects/        # 项目数据（JSON）
├── phases/          # 阶段数据（JSON）
├── tasks/           # 任务数据（JSON）
├── events/          # 事件数据（JSON）
├── knowledge/       # 知识数据（JSON）
├── quality/         # 质检数据（JSON）
├── work_records/    # 工作记录（JSON）
└── meta/            # 元数据版本标记
    ├── goals/       # 每对象一个 .meta.json
    ├── projects/
    ├── phases/
    ├── tasks/
    ├── events/
    ├── knowledge/
    ├── quality/
    └── work_records/
```

### 元数据格式

每个对象的 `meta.json` 仅包含版本信息：

```json
{
  "version": 5,
  "updated_at": "2026-01-30T12:00:00Z"
}
```

### 版本管理

- **DevMan 存储**: 仅维护元数据版本标记
- **完整快照**: 由项目自身的 Git 仓库管理
- **Rollback**: `commit()` 和 `rollback()` 方法现为 no-op（仅清空 pending 状态）

---

## 质检扩展机制

### 通用质检（内置）✅
- 编译检查
- 测试检查（支持覆盖率）
- 格式检查
- Lint 检查
- 文档检查
- 类型检查
- 安全扫描

### 业务质检（用户扩展）🔄
```rust
struct CustomCheckSpec {
    name: String,
    check_command: CommandSpec,
    validation: ValidationSpec,
    human_review: Option<HumanReviewSpec>,
}
```

**待完成**:
- [ ] 增强输出解析（正则表达式、JsonPath）
- [ ] 通知机制实现

### 人机协作流程
```
1. 系统运行自动质检
2. 发现需要人工判断的问题
3. 发送通知（Slack/Email/Webhook）🔄 待实现
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

### P0 - 基础结构 ✅
1. 核心数据模型
2. 存储层（JsonStorage）
3. CLI 基础命令

### P1 - 质量保证 ⚙️
1. QualityEngine ✅
2. 通用质检实现 ✅
3. 业务质检扩展机制 🔄
4. 质检编排 ✅

### P2 - 知识服务 🔄
1. 知识存储和检索 ✅
2. 模板系统
3. 相似度匹配（可选）

### P3 - 进度与工作管理 🔄
1. ProgressTracker ✅
2. WorkManager ✅
3. 上下文管理 ✅
4. 阻塞检测
5. 时间预估

### P4 - 工具与 AI 接口 ⚙️
1. Tool trait ✅
2. 内置工具 ✅
3. AIInterface ✅
4. MCP Server 🔄
5. 工作流编排

---

## 设计原则

1. **质检可扩展** - 通用 + 业务 + 人机协作
2. **知识可复用** - 检索、模板、推荐
3. **工具化执行** - 减少_token_消耗
4. **进度可视化** - 目标 → 阶段 → 任务
5. **存储抽象** - 可替换存储后端
6. **AI 友好** - 结构化接口
7. **可追溯性** - 完整工作日志
8. **轻量级** - 文件式存储，无外部依赖

---

## 文档计划

- [x] DESIGN.md - 完整设计方案
- [x] API.md - API 参考
- [x] QUALITY_GUIDE.md - 质检扩展指南
- [x] KNOWLEDGE.md - 知识管理指南
- [ ] ARCHITECTURE.md - 架构详解
- [ ] CONTRIBUTING.md - 贡献指南

---

## 当前进度

### 已完成 ✅
- [x] 项目规格定义（v1）
- [x] 设计方案 v2
- [x] Rust 项目初始化
- [x] 核心数据模型重构
- [x] 存储层实现（JsonStorage）
- [x] 质量保证基础实现
- [x] 知识服务基础实现
- [x] 进度追踪基础实现
- [x] 工作管理基础实现
- [x] 工具集成基础实现
- [x] AI 接口 trait 定义
- [x] CLI 基础实现
- [x] AI 模块测试（59个测试用例）
- [x] MCP Server 实现
- [x] 知识分类增强（基于 TF-IDF/关键词，无需外部向量库）
- [x] 阻塞自动检测（依赖检测 + 循环依赖检测 + 解决建议）
- [x] AI时间预估（分钟级精度 + 复杂度分级 + 置信度）
- [x] 质检检查器增强
  - [x] DocumentationExists 检查器（检查文档文件是否存在）
  - [x] TypeCheck 检查器（类型检查）
  - [x] DependenciesValid 检查器（依赖有效性检查）
  - [x] SecurityScan 检查器（安全扫描）
  - [x] 新增质检引擎测试用例（18个新增测试）
  - [x] 修复文档缺失警告（id.rs, project.rs）
  - [x] 移除 dead_code（knowledge.rs 未使用的 NodeId）
- [x] 文档完善
  - [x] API.md - 完整 API 参考手册（核心类型、质检、知识、进度、工作、工具）
  - [x] QUALITY_GUIDE.md - 质检扩展指南（内置检查器、自定义检查器、配置示例）
  - [x] KNOWLEDGE.md - 知识管理指南（知识类型、检索、模板、分类）

### 进行中 ⚙️
- [ ] ARCHITECTURE.md - 架构详解
- [ ] CONTRIBUTING.md - 贡献指南

### 计划中 📋
- [ ] ARCHITECTURE.md - 架构详解
- [ ] CONTRIBUTING.md - 贡献指南

---

## 图例

- ✅ 已完成
- 🔄 部分完成
- ⚙️ 进行中
- 📋 计划中

---

*最后更新: 2026-02-02 (文档完善完成 - 新增 API.md, QUALITY_GUIDE.md, KNOWLEDGE.md)*
