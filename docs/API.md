# DevMan API 参考

> AI 认知工作管理系统 - API 参考手册

## 目录

- [核心类型 (Core)](#核心类型-core)
- [存储 (Storage)](#存储-storage)
- [质量保证 (Quality)](#质量保证-quality)
- [知识服务 (Knowledge)](#知识服务-knowledge)
- [进度追踪 (Progress)](#进度追踪-progress)
- [工作管理 (Work)](#工作管理-work)
- [工具集成 (Tools)](#工具集成-tools)

---

## 核心类型 (Core)

### 标识符类型

所有标识符都使用 ULID（Universally Unique Lexicographically Sortable Identifier）。

```rust
// Goal ID
let goal_id = GoalId::new();

// Project ID
let project_id = ProjectId::new();

// Task ID
let task_id = TaskId::new();

// Phase ID
let phase_id = PhaseId::new();

// QualityCheck ID
let quality_check_id = QualityCheckId::new();

// Knowledge ID
let knowledge_id = KnowledgeId::new();
```

### 目标 (Goal)

```rust
let goal = Goal {
    id: GoalId::new(),
    title: "实现用户认证功能".to_string(),
    description: "添加邮箱/密码登录和注册功能".to_string(),
    success_criteria: vec![
        SuccessCriterion {
            id: CriterionId::new(),
            description: "用户可以注册新账号".to_string(),
            status: CriterionStatus::NotStarted,
        },
        SuccessCriterion {
            id: CriterionId::new(),
            description: "用户可以登录系统".to_string(),
            status: CriterionStatus::NotStarted,
        },
    ],
    progress: GoalProgress::NotStarted,
    current_phase: PhaseId::new(),
    status: GoalStatus::NotStarted,
    created_at: chrono::Utc::now(),
};
```

**GoalStatus**:
- `NotStarted` - 尚未开始
- `InProgress` - 进行中
- `Met` - 已达成
- `NotMet` - 未达成

### 项目 (Project)

```rust
let project = Project {
    id: ProjectId::new(),
    name: "用户认证服务".to_string(),
    description: "提供用户认证和授权功能".to_string(),
    config: ProjectConfig {
        tech_stack: vec!["Rust".to_string(), "Actix-web".to_string()],
        structure: DirStructure {
            dirs: vec!["src".to_string(), "tests".to_string()],
            conventions: vec!["CRUD 操作放在 services 目录".to_string()],
        },
        quality_profile: QualityProfileId::new(),
        tools: ToolConfig {
            build: BuildTool::Cargo,
            test_framework: TestFramework::Rust,
            linters: vec!["clippy".to_string()],
            formatters: vec!["rustfmt".to_string()],
        },
    },
    phases: vec![],
    current_phase: PhaseId::new(),
    created_at: chrono::Utc::now(),
};
```

**BuildTool**: `Cargo`, `Npm`, `Yarn`, `Make`, `Gradle`, `Maven`

**TestFramework**: `Rust`, `Jest`, `Pytest`, `GoTest`

### 阶段 (Phase)

```rust
let phase = Phase {
    id: PhaseId::new(),
    goal_id: GoalId::new(),
    name: "后端开发".to_string(),
    description: "实现核心认证逻辑".to_string(),
    status: PhaseStatus::NotStarted,
    acceptance_criteria: vec![
        AcceptanceCriterion {
            id: CriterionId::new(),
            description: "API 返回正确的用户数据".to_string(),
        },
    ],
    tasks: vec![],
    progress: PhaseProgress::NotStarted,
    created_at: chrono::Utc::now(),
};
```

### 任务 (Task)

```rust
let task = Task {
    id: TaskId::new(),
    title: "实现 JWT 令牌生成".to_string(),
    description: "使用 HS256 算法生成 JWT 令牌".to_string(),
    status: TaskStatus::Pending,
    state: TaskState::Pending,
    intent: TaskIntent::Implementation,
    context: TaskContext {
        goal_id: GoalId::new(),
        project_id: ProjectId::new(),
        phase_id: PhaseId::new(),
        parent_task_id: None,
        created_by: "ai".to_string(),
    },
    steps: vec![
        ExecutionStep::ToolInvocation(ToolInvocation {
            name: "cargo".to_string(),
            args: vec!["new".to_string(), "auth-service".to_string()],
        }),
    ],
    quality_gates: vec![],
    input: None,
    expected_output: None,
    links: vec![],
    created_at: chrono::Utc::now(),
    started_at: None,
    completed_at: None,
    abandoned_at: None,
    state_transitions: vec![],
};
```

**TaskStatus**: `Pending`, `InProgress`, `Completed`, `Abandoned`, `Blocked`

**TaskIntent**: `Implementation`, `Investigation`, `Documentation`, `Testing`, `Refactoring`, `Research`

---

## 质量保证 (Quality)

### 质量检查 (QualityCheck)

```rust
use devman_core::{QualityCheck, QualityCheckType, GenericCheckType, QualityCategory};

// 内置检查
let compile_check = QualityCheck {
    id: QualityCheckId::new(),
    name: "编译检查".to_string(),
    description: "确保代码编译通过".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::Compiles {
        target: "x86_64-unknown-linux-gnu".to_string(),
    }),
    severity: Severity::Error,
    category: QualityCategory::Correctness,
};
```

### 通用检查类型 (GenericCheckType)

```rust
// 编译检查
GenericCheckType::Compiles { target: "x86_64-unknown-linux-gnu".to_string() }

// 测试检查
GenericCheckType::TestsPass {
    test_suite: "lib".to_string(),
    min_coverage: Some(80.0),
}

// 格式检查
GenericCheckType::Formatted { formatter: "rustfmt".to_string() }

// Lint 检查
GenericCheckType::LintsPass { linter: "clippy".to_string() }

// 文档检查
GenericCheckType::DocumentationExists {
    paths: vec!["README.md".to_string(), "API.md".to_string()],
}

// 类型检查
GenericCheckType::TypeCheck {}

// 依赖检查
GenericCheckType::DependenciesValid {}

// 安全扫描
GenericCheckType::SecurityScan { scanner: "cargo-audit".to_string() }
```

### 自定义检查 (CustomCheck)

```rust
use devman_quality::{CustomCheckBuilder, OutputParser};

let custom_check = CustomCheckBuilder::new("coverage-check")
    .description("检查测试覆盖率")
    .command("cargo")
    .arg("test")
    .output_parser(OutputParser::Regex {
        pattern: r"Coverage: (?P<coverage>[0-9.]+)%".to_string(),
    })
    .pass_condition("coverage >= 80")
    .extract_metric(MetricExtractor {
        name: "coverage".to_string(),
        extractor: OutputParser::Regex {
            pattern: r"(?P<value>[0-9.]+)%".to_string(),
        },
        unit: Some("%".to_string()),
    })
    .build();
```

### 输出解析器 (OutputParser)

```rust
// 正则表达式解析
OutputParser::Regex {
    pattern: r"Tests: (?P<passed>\d+) passed".to_string(),
}

// JSON 路径解析
OutputParser::JsonPath {
    path: "result.status".to_string(),
}

// 行包含解析
OutputParser::LineContains {
    text: "Build succeeded".to_string(),
}
```

### 质量门 (QualityGate)

```rust
let gate = QualityGate {
    name: "代码质量门".to_string(),
    description: "提交前必须通过的质量检查".to_string(),
    checks: vec![quality_check_id],
    pass_condition: PassCondition::AllPassed,
    on_failure: FailureAction::Block,
};
```

**PassCondition**:
- `AllPassed` - 所有检查必须通过
- `AtLeast { count: usize }` - 至少 N 个检查通过
- `Custom { expression: String }` - 自定义表达式

**FailureAction**:
- `Block` - 阻止继续
- `Warn` - 警告但继续
- `Escalate` - 升级到人工审核

### 人机协作 (HumanReview)

```rust
let human_review = HumanReviewSpec {
    reviewers: vec!["reviewer@example.com".to_string()],
    review_guide: "请审查业务规则的实现".to_string(),
    review_form: vec![
        ReviewQuestion {
            question: "业务规则是否正确实现？".to_string(),
            answer_type: AnswerType::YesNo,
            required: true,
        },
        ReviewQuestion {
            question: "代码质量评分".to_string(),
            answer_type: AnswerType::Rating { min: 1, max: 5 },
            required: true,
        },
    ],
    timeout: chrono::Duration::days(1),
    auto_pass_threshold: None,
};
```

**AnswerType**:
- `YesNo` - 是/否
- `Rating { min, max }` - 评分
- `Text` - 文本
- `Choice { options }` - 选项

### 质量引擎 (QualityEngine)

```rust
use devman_quality::{QualityEngine, BasicQualityEngine, WorkContext};

let engine = BasicQualityEngine::new(storage, tool_executor);

let context = WorkContext::new(task_id);
let result = engine.run_check(&check, &context).await;

let gate_result = engine.run_gate(&gate, &context).await;
```

---

## 知识服务 (Knowledge)

### 知识条目 (Knowledge)

```rust
let knowledge = Knowledge {
    id: KnowledgeId::new(),
    title: "Rust 异步编程最佳实践".to_string(),
    knowledge_type: KnowledgeType::BestPractice {
        practice: "使用 Tokio 运行异步任务".to_string(),
        rationale: "Tokio 是最成熟的异步运行时".to_string(),
    },
    content: KnowledgeContent {
        summary: "Rust 异步编程的最佳实践总结".to_string(),
        detail: "详细说明...".to_string(),
        examples: vec![
            CodeSnippet {
                language: "rust".to_string(),
                code: "tokio::spawn(async move { ... })".to_string(),
                description: "异步任务示例".to_string(),
            },
        ],
        references: vec!["https://tokio.rs".to_string()],
    },
    metadata: KnowledgeMetadata {
        domain: vec!["Rust".to_string(), "异步".to_string()],
        tech_stack: vec!["Tokio".to_string()],
        scenarios: vec!["网络编程".to_string()],
        quality_score: 0.9,
        verified: true,
    },
    tags: vec!["async".to_string(), "tokio".to_string()],
    related_to: vec![],
    derived_from: vec![],
    usage_stats: UsageStats {
        times_used: 5,
        last_used: Some(chrono::Utc::now()),
        success_rate: 0.95,
        feedback: vec![],
    },
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
```

**KnowledgeType**:
- `LessonLearned` - 经验教训
- `BestPractice` - 最佳实践
- `CodePattern` - 代码模式
- `Solution` - 解决方案
- `Template` - 模板
- `Decision` - 决策

### 知识服务 (KnowledgeService)

```rust
use devman_knowledge::{KnowledgeService, BasicKnowledgeService, KnowledgeSearch};

// 创建服务
let service = BasicKnowledgeService::new(storage);

// 搜索知识
let results = service.search(&KnowledgeSearch {
    query: Some("异步编程".to_string()),
    knowledge_type: None,
    tags: vec!["rust".to_string()],
    limit: 10,
}).await;

// 获取推荐
let recommendations = service.get_recommendations(
    &context,
    5,
).await;

// 获取相似知识
let similar = service.get_similar(knowledge_id, 5).await;

// 创建知识
service.create_knowledge(knowledge).await;

// 更新知识
service.update_knowledge(knowledge_id, updated_knowledge).await;
```

---

## 进度追踪 (Progress)

### 阻塞检测 (Blocker)

```rust
use devman_core::{Blocker, BlockerId};

// 检测阻塞
let blockers = tracker.detect_blockers(&goal_id).await;

// 获取被阻塞的项
let blocked_items = tracker.get_blocked_items(&goal_id).await;

// 获取解决建议
let suggestions = blocker.get_resolution_suggestions();
```

**BlockerType**:
- `Dependency` - 依赖阻塞
- `Resource` - 资源阻塞
- `Decision` - 决策阻塞
- `External` - 外部阻塞

### 时间预估 (TimeEstimate)

```rust
use devman_progress::TimeEstimate;

let estimate = TimeEstimate {
    estimated_minutes: 120,
    confidence: 0.85,
    complexity: ComplexityLevel::Moderate,
    factors: vec!["代码规模中等".to_string(), "需要测试".to_string()],
    historical_base: Some(100),
};
```

**ComplexityLevel**:
- `VeryLow` - 非常低 (< 30 分钟)
- `Low` - 低 (30-60 分钟)
- `Moderate` - 中等 (1-4 小时)
- `High` - 高 (4-8 小时)
- `VeryHigh` - 非常高 (> 8 小时)

---

## 工作管理 (Work)

### 工作记录 (WorkRecord)

```rust
let work_record = WorkRecord {
    id: WorkRecordId::new(),
    task_id: TaskId::new(),
    work_type: WorkEventType::Execution,
    description: "实现了 JWT 令牌生成功能".to_string(),
    executor: Executor::Ai,
    result: WorkResult::Success,
    artifacts: vec![
        Artifact {
            name: "src/auth/token.rs".to_string(),
            artifact_type: "file".to_string(),
        },
    ],
    metrics: WorkMetrics {
        lines_added: 150,
        lines_removed: 0,
        files_changed: 1,
        duration_minutes: 45,
    },
    timestamp: chrono::Utc::now(),
};
```

**WorkEventType**: `Execution`, `Review`, `Decision`, `Blocker`, `Collaboration`

**WorkResult**: `Success`, `Partial`, `Failed`, `Blocked`

### 事件 (Event)

```rust
let event = Event {
    id: EventId::new(),
    event_type: "task.completed".to_string(),
    payload: serde_json::json!({
        "task_id": "01HX3...",
        "duration": 45,
    }),
    timestamp: chrono::Utc::now(),
};
```

---

## 工具集成 (Tools)

### 工具执行 (ToolExecutor)

```rust
use devman_tools::{ToolExecutor, BuiltinToolExecutor, ToolInput};

// 创建执行器
let executor = BuiltinToolExecutor::new();

// 执行命令
let output = executor.execute_tool("cargo", ToolInput {
    args: vec!["check".to_string()],
    env: Default::default(),
    stdin: None,
    timeout: Some(Duration::from_secs(300)),
}).await;
```

### 内置工具

```rust
// Cargo 工具
ToolInvocation {
    name: "cargo".to_string(),
    args: vec!["build".to_string(), "--release".to_string()],
}

// Git 工具
ToolInvocation {
    name: "git".to_string(),
    args: vec!["commit".to_string(), "-m".to_string(), "message".to_string()],
}

// NPM 工具
ToolInvocation {
    name: "npm".to_string(),
    args: vec!["install".to_string()],
}
```

---

## 存储 (Storage)

### Storage Trait

```rust
use devman_storage::Storage;

// 加载
let goal: Goal = storage.load_goal(goal_id).await?;

// 保存
storage.save_goal(&goal).await?;

// 查询
let tasks = storage.list_tasks(TaskFilter::ByStatus(TaskStatus::InProgress)).await?;

// 删除
storage.delete_goal(goal_id).await?;
```

---

## 常用类型别名

```rust
// 时间类型
type Time = chrono::DateTime<chrono::Utc>;

// 存储结果
type Result<T> = std::result::Result<T, StorageError>;
```

---

*最后更新: 2026-02-02*
