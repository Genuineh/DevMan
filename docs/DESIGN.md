# DevMan 设计方案 v3

> **v3 变更说明**: 移除存储层的 Git 功能，使用纯文件式 JSON 存储。版本管理由项目自身的 Git 仓库负责。

> AI 的认知工作管理系统 - 外部大脑 + 项目经理 + 质检员

## 核心定位

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

## 架构分层（五层模型）

```
Layer 5: Knowledge Service    (知识检索与复用)
├── 语义检索
├── 相似度匹配
├── 模板复用
└── 经验沉淀

Layer 4: Quality Assurance     (质量检验)
├── 通用质检（编译、测试、格式）
├── 业务质检（可扩展）
├── 质检编排
└── 人机协作接口

Layer 3: Progress Tracking     (进度管理)
├── Goal 进度
├── Phase 里程碑
├── 依赖关系
└── 阻塞检测

Layer 2: Work Management       (工作执行)
├── Task 管理
├── WorkRecord 记录
├── Context 快照
└── 变更追踪

Layer 1: Storage & State       (存储与状态）
├── 文件式 JSON 存储
├── 状态机
├── 事务管理
└── 缓存层
```

---

## 核心数据模型

### 1. Goal（目标）

系统的顶层概念，AI 要达成的长期目标。

```rust
struct Goal {
    id: GoalId,
    title: String,
    description: String,

    // 成功标准
    success_criteria: Vec<SuccessCriterion>,

    // 进度
    progress: GoalProgress,

    // 关联
    project_id: ProjectId,
    current_phase: PhaseId,

    // 元数据
    created_at: Time,
    updated_at: Time,
    status: GoalStatus,  // Active | Completed | Paused | Cancelled
}

struct SuccessCriterion {
    id: CriterionId,
    description: String,
    verification: VerificationMethod,
    status: CriterionStatus,  // NotStarted | InProgress | Met | NotMet
}

enum VerificationMethod {
    // 自动验证
    Automated(QualityCheckSpec),

    // 人工验证
    Manual { reviewer: String },

    // 混合
    Hybrid { automated: QualityCheckSpec, reviewer: String },
}

struct GoalProgress {
    percentage: f32,           // 0-100
    completed_phases: Vec<PhaseId>,
    active_tasks: usize,
    completed_tasks: usize,
    estimated_completion: Option<Time>,
    blockers: Vec<Blocker>,
}
```

### 2. Project（项目）

目标的具体化，包含工程上下文。

```rust
struct Project {
    id: ProjectId,
    name: String,
    description: String,

    // 工程配置
    config: ProjectConfig,

    // 结构
    phases: Vec<Phase>,
    tasks: Vec<Task>,

    // 状态
    current_phase: PhaseId,

    created_at: Time,
}

struct ProjectConfig {
    // 技术栈
    tech_stack: Vec<String>,

    // 目录结构
    structure: DirStructure,

    // 质检配置
    quality_profile: QualityProfile,

    // 工具配置
    tools: ToolConfig,
}

struct DirStructure {
    dirs: Vec<String>,      // ["src", "tests", "docs"]
    conventions: Vec<String>, // 命名约定、文件组织规则
}

struct ToolConfig {
    // 构建工具
    build: BuildTool,       // Cargo | Npm | Make

    // 测试框架
    test_framework: TestFramework,

    // 代码检查
    linters: Vec<String>,

    // 格式化
    formatters: Vec<String>,
}
```

### 3. Phase（阶段）

目标的阶段性划分。

```rust
struct Phase {
    id: PhaseId,
    name: String,
    description: String,

    // 阶段目标
    objectives: Vec<String>,

    // 验收标准
    acceptance_criteria: Vec<AcceptanceCriterion>,

    // 包含的任务
    tasks: Vec<TaskId>,

    // 依赖
    depends_on: Vec<PhaseId>,

    // 进度
    status: PhaseStatus,
    progress: PhaseProgress,

    estimated_duration: Option<Duration>,
    actual_duration: Option<Duration>,
}

struct AcceptanceCriterion {
    description: String,
    quality_checks: Vec<QualityCheckId>,
}

struct PhaseProgress {
    completed_tasks: usize,
    total_tasks: usize,
    percentage: f32,
}
```

### 4. Task（任务）

可执行的工作单元。

```rust
struct Task {
    id: TaskId,
    title: String,
    description: String,

    // 执行意图（AI 理解）
    intent: TaskIntent,

    // 执行步骤（工具调用）
    steps: Vec<ExecutionStep>,

    // 输入输出
    inputs: Vec<Input>,
    expected_outputs: Vec<ExpectedOutput>,

    // 质检点
    quality_gates: Vec<QualityGate>,

    // 状态
    status: TaskStatus,
    progress: TaskProgress,

    // 关联
    phase_id: PhaseId,
    depends_on: Vec<TaskId>,
    blocks: Vec<TaskId>,

    // 工作记录
    work_records: Vec<WorkRecordId>,

    created_at: Time,
    updated_at: Time,
}

struct TaskIntent {
    // 自然语言描述
    natural_language: String,

    // 上下文信息
    context: TaskContext,

    // 成功标准
    success_criteria: Vec<String>,
}

struct TaskContext {
    // 相关的知识
    relevant_knowledge: Vec<KnowledgeId>,

    // 类似的任务
    similar_tasks: Vec<TaskId>,

    // 相关的文件/模块
    affected_files: Vec<String>,
}

struct ExecutionStep {
    order: usize,
    description: String,

    // 工具调用（避免 token 消耗）
    tool: ToolInvocation,

    // 验证
    verify: Option<Verification>,
}

struct ToolInvocation {
    tool: String,           // "cargo", "npm", "git"
    args: Vec<String>,
    env: Vec<(String, String)>,
    timeout: Option<Duration>,
}

struct QualityGate {
    name: String,
    description: String,

    // 质检检查
    checks: Vec<QualityCheckId>,

    // 通过条件
    pass_condition: PassCondition,

    // 失败动作
    on_failure: FailureAction,
}

enum PassCondition {
    AllPassed,
    AtLeast { count: usize },
    Custom { expression: String },
}

enum FailureAction {
    Block,           // 阻塞继续
    Warn,            // 警告但继续
    Escalate,        // 升级给人工
}
```

### 5. WorkRecord（工作记录）

执行过程中的详细记录。

```rust
struct WorkRecord {
    id: WorkRecordId,
    task_id: TaskId,

    // 执行者
    executor: Executor,

    // 时间线
    started_at: Time,
    completed_at: Option<Time>,
    duration: Option<Duration>,

    // 过程记录
    events: Vec<WorkEvent>,

    // 结果
    result: WorkResult,

    // 生成的内容
    artifacts: Vec<Artifact>,

    // 问题和解决
    issues: Vec<Issue>,
    resolutions: Vec<Resolution>,
}

enum Executor {
    AI { model: String },
    Human { name: String },
    Hybrid { ai: String, human: String },
}

struct WorkEvent {
    timestamp: Time,
    event_type: WorkEventType,
    description: String,
    data: serde_json::Value,
}

enum WorkEventType {
    StepStarted,
    StepCompleted,
    StepFailed,
    QualityCheckStarted,
    QualityCheckPassed,
    QualityCheckFailed,
    IssueDiscovered,
    IssueResolved,
    KnowledgeCreated,
}

struct WorkResult {
    status: CompletionStatus,
    outputs: Vec<Output>,
    metrics: WorkMetrics,
}

struct WorkMetrics {
    token_used: Option<usize>,
    time_spent: Duration,
    tools_invoked: usize,
    quality_checks_run: usize,
    quality_checks_passed: usize,
}
```

### 6. Knowledge（知识）

可复用的认知资产。

```rust
struct Knowledge {
    id: KnowledgeId,
    title: String,

    // 类型
    knowledge_type: KnowledgeType,

    // 内容
    content: KnowledgeContent,

    // 元数据
    metadata: KnowledgeMetadata,

    // 关联
    tags: Vec<String>,
    related_to: Vec<KnowledgeId>,
    derived_from: Vec<WorkRecordId>,

    // 使用统计
    usage_stats: UsageStats,

    created_at: Time,
    updated_at: Time,
}

enum KnowledgeType {
    // 经验教训
    LessonLearned { lesson: String, context: String },

    // 最佳实践
    BestPractice { practice: String, rationale: String },

    // 代码模式
    CodePattern { pattern: CodeSnippet, usage: String },

    // 问题解决方案
    Solution { problem: String, solution: String, verified: bool },

    // 模板
    Template { template: TemplateContent,适用场景: Vec<String> },

    // 决策记录
    Decision { decision: String, alternatives: Vec<String>, reasoning: String },
}

struct KnowledgeContent {
    // 自然语言描述
    summary: String,

    // 详细内容
    detail: String,

    // 代码示例
    examples: Vec<CodeSnippet>,

    // 参考链接
    references: Vec<String>,
}

struct KnowledgeMetadata {
    // 领域/标签
    domain: Vec<String>,

    // 技术栈
    tech_stack: Vec<String>,

    // 适用场景
    scenarios: Vec<String>,

    // 质量评分
    quality_score: f32,

    // 验证状态
    verified: bool,
}

struct UsageStats {
    times_used: usize,
    last_used: Option<Time>,
    success_rate: f32,
    feedback: Vec<Feedback>,
}
```

### 7. QualityCheck（质检检查）

```rust
struct QualityCheck {
    id: QualityCheckId,
    name: String,
    description: String,

    // 检查类型
    check_type: QualityCheckType,

    // 配置
    config: QualityCheckConfig,

    // 严重程度
    severity: Severity,

    // 类别
    category: QualityCategory,
}

enum QualityCheckType {
    // 通用质检（内置）
    Generic(GenericCheckType),

    // 业务质检（用户扩展）
    Custom(CustomCheckSpec),
}

enum GenericCheckType {
    // 编译检查
    Compiles { target: BuildTarget },

    // 测试检查
    TestsPass {
        test_suite: String,
        min_coverage: Option<f32>,
    },

    // 格式检查
    Formatted { formatter: String },

    // Lint 检查
    LintsPass { linter: String },

    // 文档检查
    DocumentationExists { paths: Vec<String> },

    // 类型检查
    TypeCheck { }

    // 依赖检查
    DependenciesValid { }

    // 安全检查
    SecurityScan { scanner: String },
}

// 业务质检扩展点
struct CustomCheckSpec {
    // 检查名称
    name: String,

    // 检查脚本/命令
    check_command: CommandSpec,

    // 验证逻辑
    validation: ValidationSpec,

    // 人机协作
    human_review: Option<HumanReviewSpec>,
}

struct CommandSpec {
    command: String,
    args: Vec<String>,
    timeout: Duration,
    expected_exit_code: Option<i32>,
}

struct ValidationSpec {
    // 输出解析
    output_parser: OutputParser,

    // 通过条件
    pass_condition: String,

    // 提取指标
    extract_metrics: Vec<MetricExtractor>,
}

/// 输出解析器 - 支持多种解析策略
///
/// # 使用示例
///
/// ## 1. 正则表达式解析（提取命名捕获组）
/// ```rust
/// // 输出: "Coverage: 85.5%"
/// OutputParser::Regex {
///     pattern: r"Coverage: (?P<coverage>[0-9.]+)%".to_string()
/// }
/// // 解析结果: { "coverage": "85.5" }
/// ```
///
/// ## 2. JsonPath 解析（从 JSON 输出中提取值）
/// ```rust
/// // 输出: {"status": "passed", "coverage": 85.5}
/// OutputParser::JsonPath {
///     path: "status".to_string()
/// }
/// // 解析结果: { "value": "passed", "status": "passed" }
///
/// // 支持嵌套路径
/// OutputParser::JsonPath {
///     path: "result.status".to_string()
/// }
///
/// // 支持数组索引
/// OutputParser::JsonPath {
///     path: "items[0].name".to_string()
/// }
/// ```
///
/// ## 3. 行包含检查（简单文本匹配）
/// ```rust
/// // 输出: "Build succeeded\nAll tests passed"
/// OutputParser::LineContains {
///     text: "succeeded".to_string()
/// }
/// // 解析结果: { "contains": "true" }
/// ```
///
/// # 通过条件表达式
///
/// pass_condition 支持以下表达式：
///
/// - `true` - 始终通过
/// - `false` - 始终失败
/// - `field == "value"` - 字符串相等
/// - `field != "value"` - 字符串不等
/// - `field > 10` - 数值大于
/// - `field >= 10` - 数值大于等于
/// - `field < 10` - 数值小于
/// - `field <= 10` - 数值小于等于
/// - `field` - 检查字段是否存在且非空
///
/// # 完整示例
///
/// ```rust
/// // 检查测试覆盖率是否达标
/// CustomCheckSpec {
///     name: "coverage-check".to_string(),
///     check_command: CommandSpec {
///         command: "cargo".to_string(),
///         args: vec!["test".to_string(), "--".to_string(), "--nocapture".to_string()],
///         timeout: Duration::from_secs(300),
///         expected_exit_code: Some(0),
///     },
///     validation: ValidationSpec {
///         output_parser: OutputParser::Regex {
///             pattern: r"Coverage: (?P<coverage>[0-9.]+)%".to_string(),
///         },
///         pass_condition: "coverage >= 80".to_string(),
///         extract_metrics: vec![
///             MetricExtractor {
///                 name: "coverage".to_string(),
///                 extractor: OutputParser::Regex {
///                     pattern: r"(?P<value>[0-9.]+)%".to_string(),
///                 },
///                 unit: Some("%".to_string()),
///             },
///         ],
///     },
///     human_review: None,
/// }
/// ```
enum OutputParser {
    JsonPath { path: String },
    Regex { pattern: String },
    LineContains { text: String },
    Custom { script: String },
}

struct MetricExtractor {
    name: String,
    extractor: OutputParser,
    unit: Option<String>,
}

// 人机协作接口
struct HumanReviewSpec {
    // 评审者
    reviewers: Vec<String>,

    // 评审指南
    review_guide: String,

    // 评审表单
    review_form: Vec<ReviewQuestion>,

    // 超时
    timeout: Duration,

    // 自动通过阈值
    auto_pass_threshold: Option<f32>,
}

struct ReviewQuestion {
    question: String,
    answer_type: AnswerType,
    required: bool,
}

enum AnswerType {
    YesNo,
    Rating { min: i32, max: i32 },
    Text,
    Choice { options: Vec<String> },
}

enum QualityCategory {
    Correctness,    // 正确性
    Performance,    // 性能
    Security,       // 安全
    Maintainability,// 可维护性
    Documentation,  // 文档
    Testing,        // 测试
    Business,       // 业务逻辑
    Compliance,     // 合规
}

// 质检结果
struct QualityCheckResult {
    check_id: QualityCheckId,
    passed: bool,
    execution_time: Duration,

    // 详细结果
    details: CheckDetails,

    // 发现的问题
    findings: Vec<Finding>,

    // 指标
    metrics: Vec<Metric>,

    // 人工评审
    human_review: Option<HumanReviewResult>,
}

struct CheckDetails {
    output: String,
    exit_code: Option<i32>,
    error: Option<String>,
}

struct Finding {
    severity: Severity,
    category: QualityCategory,
    message: String,
    location: Option<FileLocation>,
    suggestion: Option<String>,
}

struct FileLocation {
    file: String,
    line: Option<usize>,
    column: Option<usize>,
}

struct Metric {
    name: String,
    value: f64,
    unit: Option<String>,
}

struct HumanReviewResult {
    reviewer: String,
    reviewed_at: Time,
    answers: Vec<ReviewAnswer>,
    comments: String,
    approved: bool,
}

struct ReviewAnswer {
    question: String,
    answer: AnswerValue,
}

enum AnswerValue {
    YesNo(bool),
    Rating(i32),
    Text(String),
    Choice(String),
}

// 质检编排
struct QualityProfile {
    name: String,
    description: String,

    // 检查集合
    checks: Vec<QualityCheckId>,

    // 阶段质检
    phase_gates: Vec<PhaseGate>,

    // 默认策略
    default_strategy: GateStrategy,
}

struct PhaseGate {
    phase: PhaseId,
    checks: Vec<QualityCheckId>,
    strategy: GateStrategy,
}

enum GateStrategy {
    // 全部通过
    AllMustPass,

    // 允许警告
    WarningsAllowed { max_warnings: usize },

    // 人工决策
    ManualDecision,

    // 自定义
    Custom { rule: String },
}
```

---

## 知识服务（Layer 5）

### 知识检索

```rust
#[async_trait]
pub trait KnowledgeService: Send + Sync {
    /// 语义搜索（基于相关性评分）
    async fn search_semantic(&self, query: &str, limit: usize) -> Vec<Knowledge>;

    /// 查找相似任务（TODO: 待实现）
    async fn find_similar_tasks(&self, task: &Task) -> Vec<Task>;

    /// 获取领域最佳实践
    async fn get_best_practices(&self, domain: &str) -> Vec<Knowledge>;

    /// 基于任务上下文推荐知识
    async fn recommend_knowledge(&self, context: &TaskContext) -> Vec<Knowledge>;

    /// 标签检索（OR 逻辑 - 任一标签匹配）
    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Vec<Knowledge>;

    /// 标签检索（AND 逻辑 - 所有标签必须匹配）
    async fn search_by_tags_all(&self, tags: &[String], limit: usize) -> Vec<Knowledge>;

    /// 获取所有唯一标签
    async fn get_all_tags(&self) -> HashSet<String>;

    /// 获取标签统计（标签 -> 使用次数）
    async fn get_tag_statistics(&self) -> HashMap<String, usize>;

    /// 查找相似知识（基于内容相似度）
    async fn find_similar_knowledge(&self, knowledge: &Knowledge, limit: usize) -> Vec<Knowledge>;

    /// 按类型获取知识
    async fn get_by_type(&self, knowledge_type: KnowledgeType) -> Vec<Knowledge>;

    /// 根据查询建议标签
    async fn suggest_tags(&self, query: &str, limit: usize) -> Vec<String>;
}
```

#### 相关性评分算法

```rust
fn calculate_relevance_score(&self, knowledge: &Knowledge, query_lower: &str) -> f64 {
    let mut score = 0.0;

    // 摘要匹配（最高权重）
    if knowledge.content.summary.to_lowercase().contains(query_lower) {
        score += 10.0;
    }

    // 详情匹配（中等权重）
    if knowledge.content.detail.to_lowercase().contains(query_lower) {
        score += 5.0;
    }

    // 标签匹配（高权重）
    for tag in &knowledge.tags {
        if tag.to_lowercase().contains(query_lower) {
            score += 7.0;
        }
    }

    // 领域匹配（较低权重）
    for domain in &knowledge.metadata.domain {
        if domain.to_lowercase().contains(query_lower) {
            score += 3.0;
        }
    }

    // 最佳实践或解决方案的加成
    if matches!(knowledge.knowledge_type, KnowledgeType::BestPractice { .. } | KnowledgeType::Solution { .. }) {
        score *= 1.2;
    }

    score
}
```

### 知识模板

#### 模板参数

```rust
#[derive(Debug, Clone)]
pub struct TemplateParameter {
    /// 参数名称
    pub name: String,

    /// 描述
    pub description: String,

    /// 默认值
    pub default_value: Option<String>,

    /// 是否必需
    pub required: bool,
}
```

#### 模板验证

```rust
pub struct TemplateValidation {
    /// 是否通过验证
    pub valid: bool,

    /// 缺失的必需参数
    pub missing_required: Vec<String>,

    /// 错误信息
    pub errors: Vec<String>,
}

impl TemplateValidation {
    pub fn success() -> Self { ... }
    pub fn failure(missing_required: Vec<String>, errors: Vec<String>) -> Self { ... }
}
```

#### 模板注册表

```rust
pub struct TemplateRegistry {
    templates: Vec<KnowledgeTemplate>,
}

impl TemplateRegistry {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, template: KnowledgeTemplate) { ... }
    pub fn get_by_name(&self, name: &str) -> Option<&KnowledgeTemplate> { ... }
    pub fn list(&self) -> &[KnowledgeTemplate] { ... }
    pub fn find_by_tag(&self, tag: &str) -> Vec<&KnowledgeTemplate> { ... }
}
```

#### 模板构建器

```rust
pub struct TemplateBuilder { ... }

impl TemplateBuilder {
    pub fn new(name: impl Into<String>) -> Self { ... }
    pub fn description(mut self, desc: impl Into<String>) -> Self { ... }
    pub fn required_parameter(mut self, name: impl Into<String>, description: impl Into<String>) -> Self { ... }
    pub fn optional_parameter(mut self, name: impl Into<String>, description: impl Into<String>, default: impl Into<String>) -> Self { ... }
    pub fn tag(mut self, tag: impl Into<String>) -> Self { ... }
    pub fn domain(mut self, domain: impl Into<String>) -> Self { ... }
    pub fn build(self, summary: impl Into<String>, detail: impl Into<String>) -> KnowledgeTemplate { ... }
}
```

#### 模板实例化

```rust
impl KnowledgeTemplate {
    /// 验证模板参数
    pub fn validate(&self, params: &HashMap<String, String>) -> TemplateValidation {
        // 检查所有必需参数是否提供
        let mut missing_required: Vec<String> = Vec::new();
        for param in &self.parameters {
            if param.required && !params.contains_key(&param.name) {
                missing_required.push(param.name.clone());
            }
        }
        if !missing_required.is_empty() {
            return TemplateValidation::failure(missing_required, vec!["Missing required parameters".to_string()]);
        }
        TemplateValidation::success()
    }

    /// 实例化模板
    pub fn instantiate(&self, params: &HashMap<String, String>) -> Result<Knowledge, String> {
        // 1. 验证参数
        let validation = self.validate(params);
        if !validation.valid {
            return Err(format!("Template validation failed: {:?}", validation.missing_required));
        }

        // 2. 克隆模板并生成新 ID
        let mut knowledge = self.template.clone();
        knowledge.id = KnowledgeId::new();

        // 3. 构建完整参数映射（包含默认值）
        let mut full_params = HashMap::new();
        for param in &self.parameters {
            if let Some(value) = params.get(&param.name) {
                full_params.insert(param.name.clone(), value.clone());
            } else if let Some(default) = &param.default_value {
                full_params.insert(param.name.clone(), default.clone());
            }
        }

        // 4. 替换占位符
        for (key, value) in &full_params {
            let placeholder = format!("{{{{{}}}}}", key);
            knowledge.content.summary = knowledge.content.summary.replace(&placeholder, value);
            knowledge.content.detail = knowledge.content.detail.replace(&placeholder, value);

            // 在示例中也要替换
            for example in &mut knowledge.content.examples {
                example.code = example.code.replace(&placeholder, value);
                example.description = example.description.replace(&placeholder, value);
            }
        }

        Ok(knowledge)
    }
}
```

#### 使用示例

```rust
// 创建模板
let template = TemplateBuilder::new("REST API Endpoint")
    .description("Standard REST API endpoint template")
    .required_parameter("endpoint_name", "Name of the endpoint")
    .required_parameter("method", "HTTP method (GET, POST, etc.)")
    .optional_parameter("auth_required", "Whether auth is required", "false")
    .tag("api")
    .tag("rest")
    .domain("backend")
    .build(
        "{{method}} /api/{{endpoint_name}}",
        "Implement a {{method}} endpoint for {{endpoint_name}}. Auth: {{auth_required}}"
    );

// 注册模板
let mut registry = TemplateRegistry::new();
registry.register(template);

// 实例化模板
let template = registry.get_by_name("REST API Endpoint").unwrap();
let mut params = HashMap::new();
params.insert("endpoint_name".to_string(), "users".to_string());
params.insert("method".to_string(), "GET".to_string());
// auth_required uses default value

let knowledge = template.instantiate(&params)?;
// knowledge.content.summary == "GET /api/users"
// knowledge.content.detail == "Implement a GET endpoint for users. Auth: false"
```

### 知识分类（TODO - 可选扩展）

以下功能需要向量库支持，当前版本未实现：

- [ ] 经验教训自动提取
- [ ] 最佳实践推荐
- [ ] 代码模式识别
- [ ] 解决方案索引

这些功能可以通过集成向量数据库（如 Qdrant、Milvus）或使用嵌入 API（如 OpenAI Embeddings）来实现。

---

## 质量保证（Layer 4）

### 质检引擎

```rust
trait QualityEngine {
    // 运行质检
    fn run_check(&self, check: &QualityCheck, context: &WorkContext) -> QualityCheckResult;

    // 批量运行
    fn run_checks(&self, checks: &[QualityCheck], context: &WorkContext) -> Vec<QualityCheckResult>;

    // 运行质检门
    fn run_gate(&self, gate: &QualityGate, context: &WorkContext) -> GateResult;
}

struct GateResult {
    gate_name: String,
    passed: bool,
    check_results: Vec<QualityCheckResult>,
    decision: GateDecision,
}

enum GateDecision {
    Pass,          // 通过
    Fail,          // 失败
    PassWithWarnings,  // 带警告通过
    Escalate,      // 升级给人工
}
```

### 质检扩展

```rust
// 用户注册业务质检
trait QualityCheckRegistry {
    fn register(&mut self, check: QualityCheck) -> Result<()>;
    fn unregister(&mut self, id: QualityCheckId) -> Result<()>;
    fn get(&self, id: QualityCheckId) -> Option<&QualityCheck>;
    fn list(&self, category: Option<QualityCategory>) -> Vec<&QualityCheck>;
}

// 业务质检示例：电商业务规则
struct BusinessRulesCheck {
    rules: Vec<BusinessRule>,
}

struct BusinessRule {
    name: String,
    check_fn: fn(&BusinessContext) -> RuleResult,
}

struct BusinessContext {
    order_data: Order,
    inventory: Inventory,
    pricing: Pricing,
}

// 人机协作质检
struct HumanCollaborativeCheck {
    spec: HumanReviewSpec,
    notification_channel: NotificationChannel,
}

enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook: String },
    Webhook { url: String },
    Console,  // For testing/logging
}

/// 通知服务
///
/// # 通知渠道说明
///
/// ## 1. Slack Webhook
/// 发送格式化的消息到 Slack 频道：
/// ```rust
/// let channel = NotificationChannel::Slack {
///     webhook: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL".to_string()
/// };
/// ```
///
/// ## 2. 邮件通知
/// 发送评审请求到指定邮箱：
/// ```rust
/// let channel = NotificationChannel::Email {
///     recipients: vec![
///         "reviewer@example.com".to_string(),
///         "team@example.com".to_string(),
///     ]
/// };
/// ```
///
/// ## 3. 通用 Webhook
/// 发送 JSON 格式的 POST 请求到自定义端点：
/// ```rust
/// let channel = NotificationChannel::Webhook {
///     url: "https://your-server.com/api/devman/reviews".to_string()
/// };
/// ```
///
/// # Webhook 负载格式
///
/// 通用 Webhook 发送以下 JSON 格式：
/// ```json
/// {
///   "type": "review_request",
///   "guide": "Review the pricing calculation changes",
///   "reviewers": ["reviewer@example.com"],
///   "questions": ["Is the calculation correct?", "Are edge cases handled?"],
///   "context": {
///     "description": "Review pricing module changes",
///     "files": ["src/pricing.rs", "tests/pricing_test.rs"],
///     "check_results": ["Coverage: 85%", "All tests passed"]
///   }
/// }
/// ```
///
/// # 使用示例
///
/// ```rust
/// use devman_quality::human::{HumanReviewService, NotificationChannel, ReviewContext};
///
/// let service = HumanReviewService::new(
///     NotificationChannel::Slack {
///         webhook: std::env::var("SLACK_WEBHOOK_URL").unwrap()
///     }
/// );
///
/// let context = ReviewContext {
///     description: "Review pricing calculation changes".to_string(),
///     files: vec!["src/pricing.rs".to_string()],
///     check_results: vec!["Coverage: 85%".to_string()],
/// };
///
/// service.send_notification(&spec, &context).await?;
/// ```
```

### 业务质检示例

#### 示例 1：测试覆盖率检查
```rust
use devman_quality::custom::CustomCheckBuilder;
use devman_core::{OutputParser, MetricExtractor, QualityCategory};

let coverage_check = CustomCheckBuilder::new("test-coverage")
    .description("Ensure test coverage is at least 80%")
    .severity(devman_core::Severity::Error)
    .category(QualityCategory::Testing)
    .command("cargo")
    .arg("test")
    .arg("--no-fail-fast")
    .output_parser(OutputParser::Regex {
        pattern: r"Coverage: (?P<coverage>[0-9.]+)%".to_string()
    })
    .pass_condition("coverage >= 80")
    .extract_metric(MetricExtractor {
        name: "coverage_percent".to_string(),
        extractor: OutputParser::Regex {
            pattern: r"Coverage: (?P<value>[0-9.]+)%".to_string()
        },
        unit: Some("%".to_string()),
    })
    .build();
```

#### 示例 2：JSON 输出验证
```rust
use devman_quality::custom::CustomCheckBuilder;
use devman_core::OutputParser;

let api_check = CustomCheckBuilder::new("api-health-check")
    .description("Verify API health endpoint returns success")
    .command("curl")
    .arg("-s")
    .arg("https://api.example.com/health")
    .output_parser(OutputParser::JsonPath {
        path: "status".to_string()
    })
    .pass_condition("value == healthy")
    .build();
```

#### 示例 3：带人工评审的业务规则检查
```rust
use devman_quality::custom::CustomCheckBuilder;
use devman_core::{OutputParser, HumanReviewSpec, ReviewQuestion, AnswerType};

let business_check = CustomCheckBuilder::new("pricing-rules")
    .description("Verify pricing calculation follows business rules")
    .category(QualityCategory::Business)
    .command("python")
    .arg("scripts/validate_pricing.py")
    .output_parser(OutputParser::LineContains {
        text: "Validation complete".to_string()
    })
    .human_review(HumanReviewSpec {
        reviewers: vec!["pricing-team@example.com".to_string()],
        review_guide: "Review the pricing calculation validation results".to_string(),
        review_form: vec![
            ReviewQuestion {
                question: "Are all pricing rules correctly implemented?".to_string(),
                answer_type: AnswerType::YesNo,
                required: true,
            },
            ReviewQuestion {
                question: "Rate the confidence in the implementation".to_string(),
                answer_type: AnswerType::Rating { min: 1, max: 5 },
                required: true,
            },
        ],
        timeout: std::time::Duration::from_secs(24 * 60 * 60),
        auto_pass_threshold: Some(4.0), // Auto-pass if rating >= 4
    })
    .build();
```

---

## 进度追踪（Layer 3）

```rust
trait ProgressTracker {
    // 获取目标进度
    fn get_goal_progress(&self, goal_id: GoalId) -> GoalProgress;

    // 获取阶段进度
    fn get_phase_progress(&self, phase_id: PhaseId) -> PhaseProgress;

    // 获取任务进度
    fn get_task_progress(&self, task_id: TaskId) -> TaskProgress;

    // 检测阻塞
    fn detect_blockers(&self) -> Vec<Blocker>;

    // 预估完成时间
    fn estimate_completion(&self, goal_id: GoalId) -> Option<Time>;
}

struct Blocker {
    id: BlockerId,
    blocked_item: BlockedItem,
    reason: String,
    severity: Severity,
    created_at: Time,
    resolved_at: Option<Time>,
}

enum BlockedItem {
    Task(TaskId),
    Phase(PhaseId),
    Goal(GoalId),
}
```

---

## 工作管理（Layer 2）

```rust
trait WorkManager {
    // 创建任务
    fn create_task(&mut self, spec: TaskSpec) -> Result<Task>;

    // 执行任务
    fn execute_task(&mut self, task_id: TaskId, executor: Executor) -> Result<WorkRecord>;

    // 记录事件
    fn record_event(&mut self, task_id: TaskId, event: WorkEvent) -> Result<()>;

    // 更新进度
    fn update_progress(&mut self, task_id: TaskId, progress: TaskProgress) -> Result<()>;

    // 完成任务
    fn complete_task(&mut self, task_id: TaskId, result: WorkResult) -> Result<()>;
}

struct TaskSpec {
    title: String,
    description: String,
    intent: TaskIntent,
    phase_id: PhaseId,
    quality_gates: Vec<QualityGate>,
}
```

---

## 存储（Layer 1）

```rust
trait Storage: Send + Sync {
    // CRUD
    fn save_goal(&mut self, goal: &Goal) -> Result<()>;
    fn load_goal(&self, id: GoalId) -> Result<Option<Goal>>;
    fn list_goals(&self) -> Result<Vec<Goal>>;

    fn save_project(&mut self, project: &Project) -> Result<()>;
    fn load_project(&self, id: ProjectId) -> Result<Option<Project>>;

    fn save_phase(&mut self, phase: &Phase) -> Result<()>;
    fn load_phase(&self, id: PhaseId) -> Result<Option<Phase>>;

    fn save_task(&mut self, task: &Task) -> Result<()>;
    fn load_task(&self, id: TaskId) -> Result<Option<Task>>;

    fn save_work_record(&mut self, record: &WorkRecord) -> Result<()>;
    fn load_work_record(&self, id: WorkRecordId) -> Result<Option<WorkRecord>>;

    fn save_knowledge(&mut self, knowledge: &Knowledge) -> Result<()>;
    fn load_knowledge(&self, id: KnowledgeId) -> Result<Option<Knowledge>>;

    fn save_quality_check(&mut self, check: &QualityCheck) -> Result<()>;
    fn load_quality_check(&self, id: QualityCheckId) -> Result<Option<QualityCheck>>;

    // 查询
    fn find_tasks_by_phase(&self, phase_id: PhaseId) -> Result<Vec<Task>>;
    fn find_knowledge_by_tags(&self, tags: &[String]) -> Result<Vec<Knowledge>>;
    fn find_work_records_by_task(&self, task_id: TaskId) -> Result<Vec<WorkRecord>>;

    // 事务
    fn begin_transaction(&mut self) -> Result<Transaction>;
    fn commit(&mut self, tx: Transaction) -> Result<()>;
    fn rollback(&mut self, tx: Transaction) -> Result<()>;
}
```

---

## 工具集成（减少 Token 消耗）

### 工具抽象

```rust
trait Tool {
    fn name(&self) -> &str;
    fn execute(&self, input: &ToolInput) -> Result<ToolOutput>;
    fn schema(&self) -> ToolSchema;
}

struct ToolInput {
    args: Vec<String>,
    env: HashMap<String, String>,
    stdin: Option<String>,
    timeout: Option<Duration>,
}

struct ToolOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
    duration: Duration,
}

struct ToolSchema {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
    examples: Vec<Example>,
}

// 内置工具
struct CargoTool;
struct NpmTool;
struct GitTool;
struct FsTool;  // 文件操作

// 用户扩展工具
trait CustomTool: Tool {
    fn command(&self) -> &str;
    fn validate_output(&self, output: &ToolOutput) -> Result<()>;
}
```

### 工具编排

```rust
struct Workflow {
    name: String,
    steps: Vec<WorkflowStep>,
}

struct WorkflowStep {
    name: String,
    tool: String,
    parameters: HashMap<String, serde_json::Value>,
    // 条件执行
    condition: Option<Expression>,
    // 错误处理
    on_error: ErrorHandling,
}

enum ErrorHandling {
    Stop,
    Continue,
    Retry { max_attempts: usize },
    Fallback { alternative_step: String },
}
```

---

## AI 交互接口

### MCP/CLI 接口设计

```rust
// 给 AI 的高级接口
trait AIInterface {
    // 1. 上下文查询
    fn get_current_context(&self) -> WorkContext;

    // 2. 知识检索
    fn search_knowledge(&self, query: &str) -> Vec<Knowledge>;
    fn get_relevant_practices(&self, domain: &str) -> Vec<Knowledge>;

    // 3. 进度查询
    fn get_progress(&self, goal_id: GoalId) -> GoalProgress;
    fn list_blockers(&self) -> Vec<Blocker>;

    // 4. 任务操作
    fn create_task(&mut self, spec: TaskSpec) -> Result<Task>;
    fn start_task(&mut self, task_id: TaskId) -> Result<WorkRecord>;
    fn complete_task(&mut self, task_id: TaskId, result: WorkResult) -> Result<()>;

    // 5. 质检操作
    fn run_quality_check(&mut self, check: QualityCheck) -> QualityCheckResult;
    fn get_quality_status(&self, task_id: TaskId) -> QualityStatus;

    // 6. 工具执行（减少 token）
    fn execute_tool(&mut self, tool: String, input: ToolInput) -> ToolOutput;

    // 7. 知识沉淀
    fn save_knowledge(&mut self, knowledge: Knowledge) -> Result<()>;
}

// 质检状态
struct QualityStatus {
    task_id: TaskId,
    total_checks: usize,
    passed_checks: usize,
    failed_checks: usize,
    warnings: usize,
    overall_status: QualityOverallStatus,
    pending_human_review: bool,
}

enum QualityOverallStatus {
    NotChecked,
    Passed,
    PassedWithWarnings,
    Failed,
    PendingReview,
}
```

---

## 典型工作流程

### 场景 1：AI 开始新项目

```
1. AI: "创建一个 Rust Web 框架项目"
   ↓
2. 系统创建 Goal 和 Project
   ↓
3. 系统自动生成 Phase（设计、实现、测试、文档）
   ↓
4. 系统初始化 QualityProfile（编译、测试、文档检查）
   ↓
5. 系统返回 WorkContext 给 AI
```

### 场景 2：AI 执行具体任务

```
1. AI: "我要实现 HTTP 路由"
   ↓
2. 系统查询知识库：
   - 找到 3 个类似实现
   - 推荐最佳实践
   - 返回代码模板
   ↓
3. AI 基于知识创建 Task：
   - 填充 TaskIntent
   - 定义 ExecutionStep（工具调用）
   ↓
4. 系统执行：
   - 运行 cargo new
   - 生成代码骨架
   - 运行 cargo check（工具调用，不消耗 token）
   ↓
5. 质检自动运行：
   - ✅ 编译通过
   - ✅ 测试通过
   - ⚠️ 缺少文档
   ↓
6. AI 补充文档，质检通过
   ↓
7. 系统记录 WorkRecord，提取 Knowledge
```

### 场景 3：业务质检的人机协作

```
1. 系统运行通用质检：✅ 通过
   ↓
2. 系统运行业务质检（自定义）：
   - 检查价格计算逻辑
   - 检测到潜在问题
   ↓
3. 系统触发人机协作：
   - 发送 Slack/Email 通知业务人员
   - 附上代码片段和问题描述
   ↓
4. 业务人员评审：
   - 回答评审问题
   - 添加评论
   - 批准/拒绝
   ↓
5. 系统记录评审结果
   - 更新 Knowledge（新的业务规则）
   - 继续或阻塞工作流
```

---

## Crate 结构（v3 更新：移除 Git 存储）

```
devman/
├── Cargo.toml
├── crates/
│   ├── core/                    # 核心数据模型
│   │   ├── goal
│   │   ├── project
│   │   ├── phase
│   │   ├── task
│   │   ├── work_record
│   │   ├── knowledge
│   │   └── quality
│   │
│   ├── storage/                 # 存储层
│   │   ├── trait_.rs
│   │   └── json_storage.rs
│   │
│   ├── knowledge/               # 知识服务（Layer 5）
│   │   ├── service
│   │   ├── search
│   │   ├── template
│   │   └── vector (可选)
│   │
│   ├── quality/                 # 质量保证（Layer 4）
│   │   ├── engine
│   │   ├── checks (通用质检)
│   │   ├── custom (业务质检扩展)
│   │   ├── registry
│   │   └── human_review
│   │
│   ├── progress/                # 进度追踪（Layer 3）
│   │   ├── tracker
│   │   ├── estimator
│   │   └── blocker_detection
│   │
│   ├── work/                    # 工作管理（Layer 2）
│   │   ├── manager
│   │   ├── executor
│   │   └── context
│   │
│   ├── tools/                   # 工具集成
│   │   ├── trait
│   │   ├── builtin (cargo, npm, git)
│   │   └── custom
│   │
│   ├── ai/                      # AI 接口
│   │   ├── interface
│   │   └── mcp_server
│   │
│   └── cli/                     # 命令行工具
│       └── main
│
└── docs/
    ├── DESIGN.md
    ├── TODO.md
    ├── API.md
    └── QUALITY_GUIDE.md
```

---

## 优先级实现计划

### Phase 1：核心数据模型
- Goal/Project/Phase/Task/WorkRecord
- QualityCheck 基础结构
- Knowledge 基础结构

### Phase 2：存储与基础服务
- Storage trait + JsonStorage 实现（文件式 JSON）
- 基础 CRUD
- 事务支持
- 元数据版本标记（meta.json）

### Phase 3：质量保证
- 通用质检（编译、测试）
- 质检引擎
- 质检编排

### Phase 4：知识服务
- 知识存储
- 标签检索
- 模板系统

### Phase 5：工具集成
- Tool trait
- 内置工具
- 工作流编排

### Phase 6：AI 接口
- 高层 API
- MCP Server

### Phase 7：高级特性
- 业务质检扩展
- 人机协作
- 向量检索
