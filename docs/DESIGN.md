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

#### 工作流定义

```rust
pub struct Workflow {
    /// 工作流名称
    pub name: String,

    /// 描述
    pub description: String,

    /// 工作流步骤
    pub steps: Vec<WorkflowStep>,

    /// 整体错误处理策略
    pub on_failure: FailureStrategy,

    /// 工作流变量
    pub variables: HashMap<String, String>,

    /// 是否启用回滚
    pub enable_rollback: bool,
}

/// 工作流步骤
pub struct WorkflowStep {
    /// 步骤名称
    pub name: String,

    /// 描述
    pub description: String,

    /// 要执行的工具
    pub tool: String,

    /// 工具输入
    pub input: ToolInput,

    /// 失败处理策略
    pub on_failure: FailureStrategy,

    /// 条件执行
    pub condition: Option<StepCondition>,

    /// 失败时是否继续
    pub continue_on_failure: bool,

    /// 最大重试次数
    pub max_retries: usize,

    /// 重试延迟（毫秒）
    pub retry_delay: u64,
}

/// 失败处理策略
pub enum FailureStrategy {
    /// 立即停止工作流
    Stop,

    /// 跳过此步骤并继续
    Skip,

    /// 回滚之前的步骤
    Rollback,

    /// 继续执行（标记为失败）
    Continue,
}

/// 步骤执行条件
pub enum StepCondition {
    /// 仅当前一步骤成功时运行
    PreviousSuccess(String),

    /// 仅当前一步骤失败时运行
    PreviousFailed(String),

    /// 仅当变量等于特定值时运行
    VariableEquals { name: String, value: String },

    /// 仅当变量存在时运行
    VariableExists(String),

    /// 自定义条件
    Custom(String),
}
```

#### 工作流执行器

```rust
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    /// 执行工作流
    async fn execute(&self, workflow: &Workflow) -> Result<WorkflowResult, WorkflowError>;

    /// 使用自定义变量执行工作流
    async fn execute_with_vars(
        &self,
        workflow: &Workflow,
        variables: &HashMap<String, String>,
    ) -> Result<WorkflowResult, WorkflowError>;
}

/// 基础工作流执行器
pub struct BasicWorkflowExecutor {
    tools: HashMap<String, Arc<dyn Tool>>,
}
```

#### 执行结果

```rust
pub struct WorkflowResult {
    /// 工作流是否成功完成
    pub success: bool,

    /// 每个步骤的结果
    pub step_results: Vec<StepResult>,

    /// 总执行时长
    pub duration: std::time::Duration,

    /// 错误信息
    pub error: Option<String>,
}

pub struct StepResult {
    /// 步骤名称
    pub name: String,

    /// 是否成功
    pub success: bool,

    /// 工具输出
    pub output: Option<ToolOutput>,

    /// 执行时长
    pub duration: std::time::Duration,

    /// 错误信息
    pub error: Option<String>,

    /// 是否被跳过
    pub skipped: bool,
}
```

#### 使用示例

```rust
// 创建工作流
let workflow = Workflow::new("Release Process")
    .description("Build, test, and release the project")
    .variable("project", "myproject")
    .variable("version", "1.0.0")
    .with_rollback()
    .on_failure(FailureStrategy::Rollback)
    .step(
        WorkflowStepBuilder::new("Build", "cargo")
            .args(vec!["build".to_string(), "--release".to_string()])
            .on_failure(FailureStrategy::Stop)
            .max_retries(2)
            .build()
    )
    .step(
        WorkflowStepBuilder::new("Test", "cargo")
            .args(vec!["test".to_string()])
            .condition(StepCondition::PreviousSuccess("Build".to_string()))
            .on_failure(FailureStrategy::Rollback)
            .build()
    )
    .step(
        WorkflowStepBuilder::new("Package", "cargo")
            .args(vec!["pack".to_string()])
            .condition(StepCondition::PreviousSuccess("Test".to_string()))
            .build()
    );

// 执行工作流
let executor = BasicWorkflowExecutor::new(vec![
    Arc::new(CargoTool),
    Arc::new(NpmTool),
    Arc::new(GitTool),
    Arc::new(FsTool),
]);

let result = executor.execute(&workflow).await?;

if result.success {
    println!("Workflow completed in {:?}", result.duration);
} else {
    eprintln!("Workflow failed: {:?}", result.error);
}
```

#### 变量替换

工作流支持在工具输入中使用变量：

```rust
// 定义工作流变量
workflow.variable("project", "myapp");
workflow.variable("version", "1.0.0");

// 在步骤中使用变量
WorkflowStepBuilder::new("Build", "cargo")
    .args(vec!["build".to_string(), "{project}".to_string()])
    .env("VERSION", "{version}")  // 环境变量也会被替换
    .stdin("{project} build data")  // stdin 也会被替换
    .build()
```

#### 条件执行

步骤可以根据条件决定是否执行：

```rust
// 仅当测试失败时运行
WorkflowStepBuilder::new("Debug", "cargo")
    .args(vec!["test".to_string(), "--no-fail-fast".to_string()])
    .condition(StepCondition::PreviousFailed("Test".to_string()))
    .build()

// 仅当特定变量存在时运行
WorkflowStepBuilder::new("Deploy", "npm")
    .args(vec!["publish".to_string()])
    .condition(StepCondition::VariableExists("DEPLOY_KEY".to_string()))
    .build()
```

#### 错误策略

```rust
// 遇到错误立即停止
.on_failure(FailureStrategy::Stop)

// 遇到错误跳过此步骤
.on_failure(FailureStrategy::Skip)

// 遇到错误回滚之前的步骤
.on_failure(FailureStrategy::Rollback)

// 遇到错误继续执行
.on_failure(FailureStrategy::Continue)
```

---

## AI 交互接口（Phase 8）

### 交互式任务管理设计

DevMan 采用**严格状态管控**和**交互式引导**的方式，确保 AI 按照正确的流程完成任务。

#### 核心设计理念

1. **状态机管控** - 任务通过状态机严格控制进度
2. **交互式引导** - 系统主动告诉 AI 下一步应该做什么
3. **负反馈机制** - 跳过步骤会被拒绝并提示
4. **状态校验** - 只有完成前置条件才能进入下一状态

#### 任务状态机

```rust
/// 任务状态 - 简化设计（10 个状态）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// 任务已创建
    Created { created_at: Time, created_by: String },

    /// 上下文已读取
    ContextRead { read_at: Time },

    /// 相关知识已学习
    KnowledgeReviewed { knowledge_ids: Vec<KnowledgeId>, reviewed_at: Time },

    /// 执行中
    InProgress { started_at: Time, checkpoint: Option<String> },

    /// 工作已记录，等待质检
    WorkRecorded { record_id: WorkRecordId, recorded_at: Time },

    /// 质检中
    QualityChecking { check_id: QualityCheckId, started_at: Time },

    /// 质检完成
    QualityCompleted { result: QualityCheckResult, completed_at: Time },

    /// 暂停（可恢复）
    Paused { paused_at: Time, reason: String, previous_state: Box<TaskState> },

    /// 已放弃（统一所有无法继续完成的情况）
    Abandoned { abandoned_at: Time, reason: AbandonReason },

    /// 已完成
    Completed { completed_at: Time, completed_by: String },
}

/// 状态转换流程（正常流程）
Created → ContextRead → KnowledgeReviewed → InProgress → WorkRecorded → QualityChecking → QualityCompleted → Completed

/// 状态转换（异常流程）
- 任何状态 → Paused → 恢复到之前的状态
- 任何状态 → Abandoned
- QualityCompleted → InProgress（修复问题）
```

#### 放弃任务原因（统一处理）

```rust
/// 放弃任务的原因（涵盖所有无法继续完成的情况）
#[derive(Debug, Clone)]
pub enum AbandonReason {
    /// AI/开发者主动放弃
    Voluntary { reason: String, can_be_reassigned: bool },

    /// 项目取消
    ProjectCancelled { reason: String, cancelled_by: String },

    /// 目标取消
    GoalCancelled { goal_id: GoalId, reason: String },

    /// 需求变更（无法适应）
    RequirementChanged {
        old_requirement: String,
        new_requirement: String,
        impact: ChangeImpact,
    },

    /// 依赖任务失败
    DependencyFailed { dependency_task_id: TaskId, failure_reason: String },

    /// 缺少必要信息
    InsufficientInformation { missing_info: Vec<String> },

    /// 技术限制（无法实现）
    TechnicalLimitation { limitation: String, suggested_alternative: Option<String> },

    /// 资源不可用
    ResourceUnavailable { resource: String, reason: String },

    /// 超时
    Timeout { deadline: Time, actual_completion: Option<Time> },

    /// 质检持续失败
    QualityCheckFailed { attempts: usize, remaining_issues: Vec<String> },

    /// 其他原因
    Other { reason: String, details: Option<String> },
}

/// 变更影响
#[derive(Debug, Clone)]
pub enum ChangeImpact {
    /// 可以继续
    CanContinue,

    /// 需要重新学习知识
    NeedsReview,

    /// 需要重新执行
    NeedsReexecution,

    /// 需要完全重新开始
    NeedsRestart,
}
```

#### 交互式 AI 接口

```rust
#[async_trait]
pub trait InteractiveAI: Send + Sync {
    // ==================== 任务生命周期 ====================

    /// 创建新任务
    async fn create_task(&self, request: CreateTaskRequest) -> Result<TaskId>;

    /// 放弃任务（统一入口）
    async fn abandon_task(
        &self,
        task_id: TaskId,
        reason: AbandonReason,
    ) -> Result<AbandonResult>;

    /// 完成任务
    async fn complete_task(
        &self,
        task_id: TaskId,
        summary: TaskCompletionSummary,
    ) -> Result<()>;

    // ==================== 任务引导 ====================

    /// 获取任务当前状态及下一步引导（AI 每次操作前应调用）
    async fn get_task_guidance(&self, task_id: TaskId) -> Result<TaskGuidance>;

    /// 列出任务
    async fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<TaskSummary>>;

    // ==================== 正常流程 ====================

    /// 阶段 1: 读取上下文
    async fn read_task_context(&self, task_id: TaskId) -> Result<TaskContext>;

    /// 阶段 2: 学习知识
    async fn review_knowledge(
        &self,
        task_id: TaskId,
        query: &str,
    ) -> Result<KnowledgeReviewResult>;

    /// 确认知识学习完成
    async fn confirm_knowledge_reviewed(
        &self,
        task_id: TaskId,
        knowledge_ids: Vec<KnowledgeId>,
    ) -> Result<()>;

    /// 阶段 3: 开始执行
    async fn start_execution(&self, task_id: TaskId) -> Result<ExecutionSession>;

    /// 记录工作进展
    async fn log_work(&self, task_id: TaskId, log: WorkLogEntry) -> Result<()>;

    /// 提交工作成果
    async fn finish_work(
        &self,
        task_id: TaskId,
        result: WorkSubmission,
    ) -> Result<WorkRecordId>;

    /// 阶段 4: 运行质检
    async fn run_quality_check(
        &self,
        task_id: TaskId,
        checks: Vec<QualityCheckType>,
    ) -> Result<QualityCheckId>;

    /// 获取质检结果
    async fn get_quality_result(&self, check_id: QualityCheckId) -> Result<QualityCheckResult>;

    /// 确认质检结果
    async fn confirm_quality_result(
        &self,
        task_id: TaskId,
        check_id: QualityCheckId,
        decision: QualityDecision,
    ) -> Result<()>;

    // ==================== 任务控制 ====================

    /// 暂停任务
    async fn pause_task(&self, task_id: TaskId, reason: String) -> Result<()>;

    /// 恢复任务
    async fn resume_task(&self, task_id: TaskId) -> Result<()>;

    // ==================== 需求变更 ====================

    /// 处理需求变更
    async fn handle_requirement_change(
        &self,
        task_id: TaskId,
        change: RequirementChange,
    ) -> Result<ChangeHandlingResult>;

    // ==================== 任务重新分配 ====================

    /// 请求重新分配
    async fn request_reassignment(
        &self,
        task_id: TaskId,
        reason: String,
    ) -> Result<ReassignmentRequest>;

    /// 接受重新分配的任务
    async fn accept_reassigned_task(
        &self,
        task_id: TaskId,
        request_id: ReassignmentRequestId,
    ) -> Result<TaskHandover>;
}
```

#### 任务引导信息

```rust
/// 任务引导信息（系统告诉 AI 应该做什么）
pub struct TaskGuidance {
    /// 当前状态
    pub current_state: TaskState,

    /// 下一步应该做什么
    pub next_action: NextAction,

    /// 前置条件是否满足
    pub prerequisites_satisfied: bool,

    /// 如果不满足，缺少什么
    pub missing_prerequisites: Vec<String>,

    /// 当前状态允许的操作
    pub allowed_operations: Vec<String>,

    /// 引导消息
    pub guidance_message: String,

    /// 任务健康状态
    pub health: TaskHealth,
}

/// 下一步操作指引
pub enum NextAction {
    /// 需要读取上下文
    ReadContext,

    /// 需要学习知识
    ReviewKnowledge { suggested_queries: Vec<String> },

    /// 可以开始执行
    StartExecution { suggested_workflow: Option<String> },

    /// 继续执行并记录
    ContinueExecution { required_logs: Vec<String> },

    /// 需要提交工作
    SubmitWork,

    /// 需要运行质检
    RunQualityCheck { required_checks: Vec<QualityCheckType> },

    /// 需要修复质检问题
    FixQualityIssues { issues: Vec<Finding> },

    /// 可以完成任务
    CompleteTask,

    /// 任务已完成/已放弃
    TaskFinished,
}

/// 任务健康状态
#[derive(Debug, Clone)]
pub enum TaskHealth {
    Healthy,
    Warning { warnings: Vec<String> },
    Attention { issues: Vec<TaskIssue> },
    Critical { blockers: Vec<String> },
}
```

#### 操作结果（带负反馈）

```rust
/// 操作结果（带反馈）
pub enum OperationResult<T> {
    /// 成功
    Success(T),

    /// 拒绝 - 状态不允许
    Rejected {
        reason: String,
        current_state: TaskState,
        required_state: TaskState,
        guidance: String,
    },

    /// 拒绝 - 缺少前置条件
    MissingPrerequisites {
        missing: Vec<Prerequisite>,
        hints: Vec<String>,
    },

    /// 警告 - 但允许继续
    Warning {
        result: T,
        warnings: Vec<String>,
    },
}
```

---

## AI 使用工作流程

### 交互式任务执行流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        AI 开始任务                              │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 1. 创建任务                                                     │
│   AI: create_task({title: "实现用户认证"})                      │
│   系统: 返回 task_id, 引导"请调用 read_task_context()"          │
│   状态: Created                                                 │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. 读取上下文（必须）                                           │
│   AI: get_task_guidance(task_id)                               │
│   系统: "请先调用 read_task_context() 读取上下文"               │
│   AI: read_task_context(task_id)                               │
│   系统: 返回项目信息、依赖、质检要求                            │
│   状态: ContextRead                                             │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. 学习知识（必须）                                             │
│   AI: get_task_guidance(task_id)                               │
│   系统: "请调用 review_knowledge() 学习相关知识"                │
│   AI: review_knowledge(task_id, "authentication rust")          │
│   系统: 返回相关知识, 建议必读内容                              │
│   AI: confirm_knowledge_reviewed(task_id, [knowledge_ids])     │
│   状态: KnowledgeReviewed                                      │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. 开始执行（必须学完知识）                                     │
│   AI: start_execution(task_id)                                 │
│   系统: 返回执行会话, 引导"使用 log_work() 记录工作"            │
│   状态: InProgress                                              │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. 记录工作进展（执行过程中）                                   │
│   AI: log_work(task_id, {action: "implemented JWT middleware"}) │
│   AI: log_work(task_id, {action: "wrote unit tests"})          │
│   系统: 记录每次工作进展                                        │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. 提交工作（必须记录）                                         │
│   AI: finish_work(task_id, {artifacts: [...]})                  │
│   系统: 检查是否有工作记录, 返回 record_id                      │
│   状态: WorkRecorded                                            │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 7. 运行质检（必须）                                             │
│   AI: run_quality_check(task_id, [compile, test, lint])         │
│   系统: 返回 check_id, 状态变为 QualityChecking                 │
│   AI: get_quality_result(check_id)                             │
│   系统: 返回质检报告                                            │
│   状态: QualityCompleted                                        │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ 8. 完成任务或修复问题                                           │
│   质检通过: complete_task(task_id, {summary: ...})             │
│   质检失败: start_execution(task_id) 重新开始                  │
│   状态: Completed 或回到 InProgress                             │
└─────────────────────────────────────────────────────────────────┘
```

### 负反馈场景示例

#### 场景 1: AI 跳过读取上下文

```
AI: start_execution("task_001")  // 直接开始执行

系统: Rejected {
    "reason": "状态不允许",
    "current_state": "Created",
    "required_state": "KnowledgeReviewed",
    "guidance": "请按以下顺序操作：
        1. 调用 read_task_context() 读取上下文
        2. 调用 review_knowledge() 学习知识
        3. 再调用 start_execution() 开始执行"
}
```

#### 场景 2: AI 跳过学习知识

```
AI: read_task_context("task_001")  // 读取了上下文
→ 状态: ContextRead

AI: start_execution("task_001")  // 直接开始执行

系统: Rejected {
    "reason": "缺少前置条件",
    "missing_prerequisites": [
        {
            "name": "knowledge_review",
            "description": "学习相关知识",
            "how_to_satisfy": "调用 review_knowledge() 查询相关知识"
        }
    ],
    "hints": [
        "系统建议查询: authentication best practices",
        "系统建议查询: JWT token rust implementation"
    ]
}
```

#### 场景 3: AI 执行未记录工作

```
AI: finish_work("task_001", {...})

系统: Rejected {
    "reason": "缺少工作记录",
    "missing_prerequisites": [
        {
            "name": "work_logs",
            "description": "工作进展记录",
            "how_to_satisfy": "调用 log_work() 记录你的工作进展"
        }
    ],
    "hints": [
        "请记录: 实现了哪些功能",
        "请记录: 运行了哪些测试",
        "请记录: 遇到了什么问题"
    ]
}
```

### 任务放弃场景

#### 场景 1: 项目取消

```
AI: abandon_task("task_001", ProjectCancelled {
    reason: "产品方向调整，此功能不再需要",
    cancelled_by: "product_manager"
})

系统: {
    "success": true,
    "new_state": "Abandoned",
    "work_preserved": true,
    "message": "任务已标记为放弃（项目取消）。工作记录已保存。"
}
```

#### 场景 2: 需求变更太大

```
系统: handle_requirement_change("task_001", {
    "description": "认证方式从 JWT 改为完整的 OAuth2 + SSO",
    "impact": "NeedsRestart"
})

系统: {
    "result": "RecommendNewTask",
    "reason": "变更影响太大，建议放弃当前任务创建新任务",
    "reusable_content": [
        "已学习的 JWT 知识可能仍有参考价值",
        "单元测试框架搭建经验可以复用"
    ]
}

AI: abandon_task("task_001", RequirementChanged {
    old_requirement: "实现 JWT 认证",
    new_requirement: "实现完整的 OAuth2 + SSO 系统",
    impact: ChangeImpact::NeedsRestart
})
```

#### 场景 3: AI 主动放弃（缺少信息）

```
AI: abandon_task("task_001", InsufficientInformation {
    missing_info: ["第三方 API 文档", "数据库 schema 定义"]
})

系统: {
    "success": true,
    "can_be_reassigned": true,
    "work_reusable": true,
    "suggestions_for_next": [
        "在开始执行前，确保所有依赖信息都齐全",
        "已学习的 JWT 相关知识仍然有用"
    ]
}
```

### 任务重新分配场景

```
// AI A 放弃任务
AI A: abandon_task("task_001", Voluntary {
    reason: "我对这个技术栈不熟悉",
    can_be_reassigned: true
})

// 系统通知管理员，管理员批准重新分配

// AI B 接受重新分配的任务
AI B: accept_reassigned_task("task_001", "reassign_req_001")

系统: {
    "task": {...},
    "current_state": "Abandoned",
    "completed_work": [...],
    "reviewed_knowledge": ["knowledge_jwt_01"],
    "abandonment_reason": "对技术栈不熟悉",
    "suggestions": [
        "上一个 AI 已经学习了 JWT 相关知识",
        "可以直接参考已完成的工作"
    ],
    "warnings": [
        "注意：这个任务有特殊的 OAuth2 要求"
    ],
    "reusable_artifacts": [...]
}

系统引导: "你已接受此任务。请先查看已完成的工作，状态将重置为 Created。"
→ 状态: Abandoned → Created (对 AI B 而言)
```

### 需求变更处理场景

#### 场景 1: 小变更 - 可以继续

```
AI: handle_requirement_change("task_001", {
    "description": "token 过期时间从 1小时 改为 2小时",
    "impact": "CanContinue"
})

系统: {
    "result": "CanContinue",
    "message": "需求变更影响较小，可以直接继续执行"
}
```

#### 场景 2: 中等变更 - 需要重新学习

```
AI: handle_requirement_change("task_001", {
    "description": "增加 refresh token 支持",
    "impact": "NeedsReview"
})

系统: {
    "result": "NeedsReview",
    "suggested_knowledge": ["refresh token best practices", "token rotation"],
    "guidance": "请学习新知识后继续执行"
}

AI: review_knowledge("task_001", "refresh token rust")

系统: "知识学习完成，可以继续执行"
```

#### 场景 3: 大变更 - 需要重新执行

```
AI: handle_requirement_change("task_001", {
    "description": "认证方式从 JWT 改为 OAuth2",
    "impact": "NeedsReexecution"
})

系统: {
    "result": "NeedsReexecution",
    "affected_work": ["src/auth/jwt.rs", "src/auth/middleware.rs"],
    "guidance": "需求变更影响中等。已做的工作需要调整，请重新执行。"
}

AI: start_execution("task_001")  // 重新开始执行
```

---

## 典型工作流程（更新版）

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

### MCP Server 设计

> **当前状态**: 基础框架已完成，缺少引导性工具对接

#### MCP 协议结构

```
DevMan MCP Server
├── Tools (已实现 12 个基础工具)
│   ├── devman_create_goal              # 创建目标
│   ├── devman_get_goal_progress        # 获取目标进度
│   ├── devman_create_task              # 创建任务
│   ├── devman_list_tasks               # 列出任务
│   ├── devman_search_knowledge         # 搜索知识库
│   ├── devman_save_knowledge           # 保存知识
│   ├── devman_run_quality_check        # 运行质检
│   ├── devman_execute_tool             # 执行工具
│   ├── devman_get_context              # 获取上下文
│   ├── devman_list_blockers            # 列出阻塞项
│   ├── devman_get_job_status           # 获取任务状态
│   └── devman_cancel_job               # 取消任务
│
├── Tools (待实现 - 引导性工具)
│   ├── devman_get_task_guidance        # 获取任务引导 ⭐ 核心
│   ├── devman_read_task_context        # 读取上下文
│   ├── devman_confirm_knowledge_reviewed # 确认知识学习
│   ├── devman_start_execution          # 开始执行
│   ├── devman_log_work                 # 记录工作
│   ├── devman_finish_work              # 提交工作
│   ├── devman_get_quality_result       # 获取质检结果
│   ├── devman_confirm_quality_result   # 确认质检结果
│   ├── devman_complete_task            # 完成任务
│   ├── devman_pause_task               # 暂停任务
│   ├── devman_resume_task              # 恢复任务
│   ├── devman_abandon_task             # 放弃任务
│   └── devman_handle_requirement_change # 处理需求变更
│
├── Resources (部分实现)
│   ├── devman://context/project        # 当前项目
│   ├── devman://context/goal           # 当前目标
│   ├── devman://tasks/queue            # 任务队列
│   ├── devman://knowledge/recent       # 最近知识
│   ├── devman://task/{id}              # 任务详情 (待实现)
│   ├── devman://project/current        # 当前项目 (待实现)
│   └── devman://quality/status/{id}    # 质检状态 (待实现)
│
└── Prompts (待实现)
    ├── devman_start_new_project
    ├── devman_implement_feature
    ├── devman_fix_bug
    └── devman_handle_issue
```

### MCP Tool 定义示例

```json
{
  "name": "devman_get_task_guidance",
  "description": "获取任务当前状态及下一步引导。AI 每次操作前都应该调用此接口了解应该做什么。",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task_id": {
        "type": "string",
        "description": "任务 ID"
      }
    },
    "required": ["task_id"]
  }
}

{
  "name": "devman_create_task",
  "description": "创建新任务",
  "inputSchema": {
    "type": "object",
    "properties": {
      "title": {"type": "string", "description": "任务标题"},
      "description": {"type": "string", "description": "任务描述"},
      "goal_id": {"type": "string", "description": "关联目标 ID（可选）"},
      "phase_id": {"type": "string", "description": "关联阶段 ID（可选）"},
      "estimated_duration": {"type": "string", "description": "预估时长（可选）"}
    },
    "required": ["title", "description"]
  }
}

{
  "name": "devman_abandon_task",
  "description": "放弃任务（涵盖所有无法继续完成的情况：项目取消、需求变更、无法完成等）",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task_id": {"type": "string"},
      "reason_type": {
        "type": "string",
        "enum": [
          "voluntary",
          "project_cancelled",
          "goal_cancelled",
          "requirement_changed",
          "dependency_failed",
          "insufficient_info",
          "technical_limitation",
          "resource_unavailable",
          "timeout",
          "quality_failed",
          "other"
        ],
        "description": "放弃原因类型"
      },
      "reason": {"type": "string", "description": "详细原因说明"},
      "details": {"type": "object", "description": "附加信息"}
    },
    "required": ["task_id", "reason_type", "reason"]
  }
}

{
  "name": "devman_handle_requirement_change",
  "description": "处理需求变更，系统会根据影响决定如何处理",
  "inputSchema": {
    "type": "object",
    "properties": {
      "task_id": {"type": "string"},
      "change": {
        "type": "object",
        "properties": {
          "description": {"type": "string"},
          "change_type": {
            "type": "string",
            "enum": ["feature", "priority", "deadline", "dependency", "quality"]
          },
          "old_value": {},
          "new_value": {}
        }
      }
    },
    "required": ["task_id", "change"]
  }
}
```

### MCP Resource 定义

```
devman://task/{id}
→ 返回任务完整信息：状态、上下文、相关知识、工作记录、质检结果

devman://project/current
→ 返回当前项目信息： phases、active goals、配置

devman://tasks/pending
→ 返回所有待处理的任务列表

devman://tasks/in_progress
→ 返回所有进行中的任务列表

devman://knowledge/{id}
→ 返回知识详情：内容、示例、相关知识

devman://quality/status/{task_id}
→ 返回任务的质检状态和结果
```

### MCP Prompts 定义

```json
{
  "name": "devman_implement_feature",
  "description": "实现新功能的完整流程",
  "arguments": [
    {
      "name": "feature_description",
      "description": "功能描述",
      "required": true
    },
    {
      "name": "context",
      "description": "额外上下文信息",
      "required": false
    }
  ]
}
```

使用时，MCP 客户端会展开为完整的提示词，引导 AI 按照正确的流程操作。

### stdio 传输

```rust
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

#[rpc(server)]
pub trait DevManMcp {
    /// 获取任务引导
    #[method(name = "devman.get_task_guidance")]
    async fn get_task_guidance(&self, task_id: String) -> RpcResult<TaskGuidance>;

    /// 创建任务
    #[method(name = "devman.create_task")]
    async fn create_task(&self, request: CreateTaskRequest) -> RpcResult<String>;

    /// ... 其他方法
}

/// stdio 主循环
pub async fn run_stdio_server() -> anyhow::Result<()> {
    let server = DevManMcpServer::new(...).await?;

    let module = rpc_module! {
        server => DevManMcp,
    };

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    // JSON-RPC over stdio
    loop {
        let line = stdin.read_line().await?;
        let response = module.process_request(&line).await?;
        writeln!(stdout, "{}", response)?;
    }
}
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
│   ├── ai/                      # AI 接口（Phase 8）
│   │   ├── interface.rs          # 基础 AI 接口
│   │   ├── interactive.rs        # 交互式 AI 接口
│   │   ├── guidance.rs           # 任务引导逻辑
│   │   ├── validation.rs         # 状态校验逻辑
│   │   └── mcp_server.rs         # MCP Server 实现
│   │
│   └── cli/                     # 命令行工具
│       └── main.rs
│
└── docs/
    ├── DESIGN.md
    ├── TODO.md
    ├── API.md
    └── QUALITY_GUIDE.md
```

---

## 优先级实现计划

### Phase 1：核心数据模型 ✅
- [x] Goal/Project/Phase/Task/WorkRecord
- [x] QualityCheck 基础结构
- [x] Knowledge 基础结构

### Phase 2：存储与基础服务 ✅
- [x] Storage trait + JsonStorage 实现（文件式 JSON）
- [x] 基础 CRUD
- [x] 事务支持
- [x] 元数据版本标记（meta.json）

### Phase 3：质量保证 ✅
- [x] 通用质检（编译、测试）
- [x] 质检引擎
- [x] 质检编排
- [x] 业务质检扩展
- [x] 输出解析（Regex/JsonPath）
- [x] 人机协作

### Phase 4：知识服务 ✅
- [x] 知识存储
- [x] 标签检索
- [x] 模板系统
- [x] 相关性评分

### Phase 5：进度追踪 🔄
- [x] ProgressTracker trait
- [x] 目标进度计算
- [ ] 阻塞检测
- [ ] 时间预估

### Phase 6：工作管理 ✅
- [x] WorkManager trait
- [x] 任务创建和执行
- [x] 上下文管理
- [x] 事件记录

### Phase 7：工具集成 ✅
- [x] Tool trait
- [x] 内置工具（Cargo, Npm, Git, Fs）
- [x] 工作流编排
- [x] 错误处理策略

### Phase 8：AI 接口 ⚙️

#### 已完成 ✅

- [x] AIInterface trait - 基础 AI 接口
- [x] InteractiveAI trait - 交互式 AI 接口定义
- [x] TaskStateValidator - 状态校验逻辑
- [x] TaskGuidanceGenerator - 任务引导逻辑
- [x] MCP Server 实现 - 基础协议框架
- [x] 12 个 MCP 工具（基础 CRUD）
- [x] JobManager - 异步任务管理
- [x] 资源版本控制

#### 待完善/未实现 🔄

- [ ] InteractiveAI trait 实现（BasicInteractiveAI 目前全是 TODO）
- [x] 任务引导生成器（guidance.rs 已实现，但 MCP 未调用）
- [ ] 任务状态机完整集成（TaskState 枚举已定义）
- [ ] 负反馈机制 - MCP 返回拒绝原因和引导
- [ ] 任务控制（暂停/恢复/放弃）- MCP 工具缺失
- [ ] 需求变更处理 - MCP 工具缺失
- [ ] 任务重新分配 - MCP 工具缺失
- [x] MCP Prompts 模板 - 需要定义
- [ ] MCP Resources 完整实现 - 当前返回占位数据

#### 当前 MCP 工具状态

```bash
# 已实现（基础 CRUD）
devman_create_goal          ✅
devman_get_goal_progress    ✅
devman_create_task          ⚠️ (占位符)
devman_list_tasks           ✅
devman_search_knowledge     ✅
devman_save_knowledge       ⚠️ (占位符)
devman_run_quality_check    ✅
devman_execute_tool         ⚠️ (占位符)
devman_get_context          ✅
devman_list_blockers        ✅
devman_get_job_status       ✅
devman_cancel_job           ✅

# 待添加（引导性工具）
devman_get_task_guidance    ⬜
devman_read_task_context    ⬜
devman_confirm_knowledge_reviewed  ⬜
devman_start_execution      ⬜
devman_log_work             ⬜
devman_finish_work          ⬜
devman_get_quality_result   ⬜
devman_confirm_quality_result  ⬜
devman_complete_task        ⬜
devman_pause_task           ⬜
devman_resume_task          ⬜
devman_abandon_task         ⬜
devman_handle_requirement_change  ⬜
```

### 当前实现 vs 设计文档的差异

1. **InteractiveAI 已定义但未完整实现** - BasicInteractiveAI 中大部分方法返回占位符
2. **guidance.rs 已实现但 MCP 未调用** - TaskGuidanceGenerator 完整可用
3. **validation.rs 已实现** - TaskStateValidator 包含完整的状态转换校验
4. **MCP 缺少引导性工具** - 设计文档要求 16+ 工具，当前只有 12 个基础工具
5. **资源实现是占位符** - read_resource() 返回空数据
- 业务质检扩展
- 人机协作
- 向量检索
