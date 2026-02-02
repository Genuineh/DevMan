# DevMan 质检扩展指南

> 如何使用和扩展 DevMan 的质量保证系统

## 目录

- [概述](#概述)
- [内置检查器](#内置检查器)
- [自定义检查器](#自定义检查器)
- [质量门配置](#质量门配置)
- [质量配置文件](#质量配置文件)
- [人机协作](#人机协作)
- [输出解析](#输出解析)
- [最佳实践](#最佳实践)

---

## 概述

DevMan 质量保证系统采用分层设计：

```
┌─────────────────────────────────────┐
│         Quality Profile             │  ← 质检配置集合
├─────────────────────────────────────┤
│           Quality Gate              │  ← 质检门（检查点）
├─────────────────────────────────────┤
│          Quality Check              │  ← 单个检查
├─────────────────────────────────────┤
│      Output Parser / Validator      │  ← 输出解析
└─────────────────────────────────────┘
```

### 核心概念

- **QualityCheck**: 单个质量检查的定义
- **QualityGate**: 多个检查的组合，必须全部通过
- **QualityProfile**: 一组预配置的质量门
- **GenericCheckType**: 内置检查类型（编译、测试等）
- **CustomCheckSpec**: 自定义检查规范

---

## 内置检查器

### 编译检查 (Compiles)

```rust
use devman_core::{QualityCheck, QualityCheckType, GenericCheckType, QualityCategory};

let compile_check = QualityCheck {
    id: QualityCheckId::new(),
    name: "编译检查".to_string(),
    description: "确保代码可以成功编译".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::Compiles {
        target: "x86_64-unknown-linux-gnu".to_string(),
    }),
    severity: Severity::Error,
    category: QualityCategory::Correctness,
};
```

**配置选项**:
- `target`: 编译目标平台（可选，默认当前平台）

### 测试检查 (TestsPass)

```rust
let test_check = QualityCheck {
    name: "测试检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::TestsPass {
        test_suite: "lib".to_string(),      // 测试套件名称
        min_coverage: Some(80.0),           // 最低覆盖率要求（可选）
    }),
    ..Default::default()
};
```

**配置选项**:
- `test_suite`: 测试套件名称（空字符串表示所有测试）
- `min_coverage`: 最低测试覆盖率（可选）

### 格式检查 (Formatted)

```rust
let format_check = QualityCheck {
    name: "代码格式检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::Formatted {
        formatter: "rustfmt".to_string(),
    }),
    ..Default::default()
};
```

**支持工具**:
- `rustfmt` - Rust 代码格式化
- `black` - Python 代码格式化
- `prettier` - JavaScript/TypeScript 格式化

### Lint 检查 (LintsPass)

```rust
let lint_check = QualityCheck {
    name: "Lint 检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::LintsPass {
        linter: "clippy".to_string(),
    }),
    ..Default::default()
};
```

**支持工具**:
- `clippy` - Rust linter
- `eslint` - JavaScript/TypeScript linter
- `pylint` - Python linter

### 文档检查 (DocumentationExists)

```rust
let doc_check = QualityCheck {
    name: "文档检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::DocumentationExists {
        paths: vec![
            "README.md".to_string(),
            "API.md".to_string(),
            "docs/".to_string(),
        ],
    }),
    ..Default::default()
};
```

此检查会验证指定的文件或目录是否存在。

### 类型检查 (TypeCheck)

```rust
let type_check = QualityCheck {
    name: "类型检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::TypeCheck {}),
    ..Default::default()
};
```

运行 `cargo check` 进行静态类型检查。

### 依赖检查 (DependenciesValid)

```rust
let deps_check = QualityCheck {
    name: "依赖检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::DependenciesValid {}),
    ..Default::default()
};
```

检查依赖是否有效、无过期或安全漏洞。

### 安全扫描 (SecurityScan)

```rust
let security_check = QualityCheck {
    name: "安全扫描".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::SecurityScan {
        scanner: "cargo-audit".to_string(),
    }),
    ..Default::default()
};
```

**支持工具**:
- `cargo-audit` - Rust 安全审计
- `npm audit` - Node.js 安全审计
- `bandit` - Python 安全扫描

---

## 自定义检查器

### 使用 CustomCheckBuilder

```rust
use devman_quality::{CustomCheckBuilder, OutputParser};
use devman_core::{MetricExtractor, Severity, QualityCategory};

let custom_check = CustomCheckBuilder::new("my-custom-check")
    .description("自定义质量检查")
    .severity(Severity::Warning)
    .category(QualityCategory::Performance)
    .command("my-tool")
    .arg("--analyze")
    .arg("src/")
    .timeout(std::time::Duration::from_secs(60))
    .expected_exit_code(0)
    .output_parser(OutputParser::Regex {
        pattern: r"Performance score: (?P<score>[0-9.]+)".to_string(),
    })
    .pass_condition("score >= 80")
    .extract_metric(MetricExtractor {
        name: "performance_score".to_string(),
        extractor: OutputParser::Regex {
            pattern: r"(?P<value>[0-9.]+)".to_string(),
        },
        unit: Some("points".to_string()),
    })
    .build();
```

### 检查器属性说明

| 属性 | 说明 | 默认值 |
|------|------|--------|
| `name` | 检查名称 | 必填 |
| `description` | 检查描述 | 空字符串 |
| `severity` | 严重程度 | Warning |
| `category` | 质量类别 | Correctness |
| `command` | 要执行的命令 | 必填 |
| `args` | 命令参数 | 空数组 |
| `timeout` | 超时时间 | 60秒 |
| `expected_exit_code` | 期望的退出码 | 0 |
| `output_parser` | 输出解析器 | LineContains |
| `pass_condition` | 通过条件 | "true" |

---

## 质量门配置

### 基本配置

```rust
use devman_core::{QualityGate, PassCondition, FailureAction};

let gate = QualityGate {
    name: "代码提交门".to_string(),
    description: "代码提交前必须通过的质量检查".to_string(),
    checks: vec![compile_check_id, test_check_id, lint_check_id],
    pass_condition: PassCondition::AllPassed,
    on_failure: FailureAction::Block,
};
```

### PassCondition 类型

```rust
// 所有检查必须通过
PassCondition::AllPassed

// 至少 N 个检查通过
PassCondition::AtLeast { count: 3 }

// 自定义表达式（未来功能）
PassCondition::Custom { expression: "score >= 80".to_string() }
```

### FailureAction 类型

```rust
// 阻止继续（推荐用于关键检查）
FailureAction::Block

// 警告但继续
FailureAction::Warn

// 升级到人工审核
FailureAction::Escalate
```

### 阶段门 (PhaseGate)

```rust
use devman_core::{PhaseGate, GateStrategy};

let phase_gate = PhaseGate {
    phase: phase_id,
    checks: vec![check_id_1, check_id_2],
    strategy: GateStrategy::AllMustPass,
};
```

**GateStrategy**:
- `AllMustPass` - 所有检查必须通过
- `WarningsAllowed { max_warnings: 5 }` - 允许最多 N 个警告
- `ManualDecision` - 需要人工决策
- `Custom { rule: String }` - 自定义规则

---

## 质量配置文件

### 创建质量配置

```rust
use devman_core::{QualityProfile, QualityProfileId};

let profile = QualityProfile {
    name: "Rust 项目标准配置".to_string(),
    description: "适用于 Rust 项目的标准质量配置".to_string(),
    checks: vec![check1_id, check2_id, check3_id],
    phase_gates: vec![
        PhaseGate {
            phase: phase_id_1,
            checks: vec![check1_id],
            strategy: GateStrategy::AllMustPass,
        },
        PhaseGate {
            phase: phase_id_2,
            checks: vec![check1_id, check2_id],
            strategy: GateStrategy::AllMustPass,
        },
    ],
    default_strategy: GateStrategy::AllMustPass,
};
```

### 使用 QualityProfileBuilder

```rust
use devman_quality::QualityProfileBuilder;

let profile = QualityProfileBuilder::new("my-profile")
    .with_description("我的质量配置")
    .with_default_strategy(GateStrategy::AllMustPass)
    .add_check(check1_id)
    .add_check(check2_id)
    .add_phase_gate(phase_gate)
    .build();
```

---

## 人机协作

### 配置人工审核

```rust
use devman_core::{HumanReviewSpec, ReviewQuestion, AnswerType};

let human_review = HumanReviewSpec {
    reviewers: vec![
        "tech-lead@example.com".to_string(),
        "senior-dev@example.com".to_string(),
    ],
    review_guide: "请审查代码变更的业务逻辑是否正确实现。".to_string(),
    review_form: vec![
        ReviewQuestion {
            question: "业务逻辑是否正确？".to_string(),
            answer_type: AnswerType::YesNo,
            required: true,
        },
        ReviewQuestion {
            question: "代码质量评分（1-5）".to_string(),
            answer_type: AnswerType::Rating { min: 1, max: 5 },
            required: true,
        },
        ReviewQuestion {
            question: "改进建议".to_string(),
            answer_type: AnswerType::Text,
            required: false,
        },
    ],
    timeout: std::time::Duration::from_secs(24 * 60 * 60), // 24小时
    auto_pass_threshold: Some(4.0), // 平均评分 >= 4 自动通过
};
```

### 通知渠道

```rust
use devman_core::NotificationChannel;

// Slack 通知
let slack_channel = NotificationChannel::Slack {
    webhook: "https://hooks.slack.com/services/xxx".to_string(),
};

// 邮件通知
let email_channel = NotificationChannel::Email {
    recipients: vec!["reviewer@example.com".to_string()],
};

// Webhook 通知
let webhook_channel = NotificationChannel::Webhook {
    url: "https://example.com/webhook".to_string(),
};
```

### 使用 HumanReviewService

```rust
use devman_quality::{HumanReviewService, NotificationChannel};

let service = HumanReviewService::new(NotificationChannel::Console);

// 发送审核请求
service.send_notification(&human_review, &context).await;

// 处理审核结果
let result = service.process_response(&human_review, answers);
```

---

## 输出解析

### 正则表达式解析

```rust
OutputParser::Regex {
    pattern: r"Tests: (?P<passed>\d+) passed, (?P<failed>\d+) failed".to_string(),
}
```

**命名捕获组**:
- `(?P<name>pattern)` - 命名捕获组，提取为变量
- `(?<name>pattern)` - 备用语法

**示例输出**:
```
Tests: 150 passed, 2 failed
```

**提取结果**:
```rust
values.get("passed")  // "150"
values.get("failed")  // "2"
```

### JSON 路径解析

```rust
OutputParser::JsonPath {
    path: "result.testCoverage".to_string(),
}
```

**支持的路径语法**:
- `field` - 根字段
- `field.nested` - 嵌套字段
- `array[0]` - 数组索引
- `field.array[0].nested` - 组合

**示例输出**:
```json
{
  "status": "success",
  "result": {
    "testCoverage": 85.5
  }
}
```

**提取结果**:
```rust
values.get("value")       // "85.5"
values.get("result.testCoverage")  // "85.5"
```

### 行包含解析

```rust
OutputParser::LineContains {
    text: "Build succeeded".to_string(),
}
```

检查输出中是否包含指定文本。

**提取结果**:
```rust
values.get("contains")  // "true" 或 "false"
```

### 通过条件表达式

```rust
pass_condition: "coverage >= 80 && passed > 0"
```

**支持的运算符**:
| 运算符 | 说明 |
|--------|------|
| `==` | 等于 |
| `!=` | 不等于 |
| `>` | 大于 |
| `>=` | 大于等于 |
| `<` | 小于 |
| `<=` | 小于等于 |
| `&&` | 逻辑与 |
| `\|\|` | 逻辑或 |

### 指标提取

```rust
MetricExtractor {
    name: "coverage".to_string(),
    extractor: OutputParser::Regex {
        pattern: r"Coverage: (?P<value>[0-9.]+)%".to_string(),
    },
    unit: Some("%".to_string()),
}
```

---

## 最佳实践

### 1. 检查器命名规范

```rust
// Good
let check = CustomCheckBuilder::new("security-check-owasp")
    .description("OWASP Top 10 安全检查")
    .build();

// Bad
let check = CustomCheckBuilder::new("check")
    .description("some check")
    .build();
```

### 2. 合理的超时时间

```rust
// Good: 根据检查复杂度设置超时
.timeout(std::time::Duration::from_secs(300))  // 5分钟

// Bad: 超时设置不合理
.timeout(std::time::Duration::from_secs(1))    // 太短
.timeout(std::time::Duration::from_secs(3600)) // 太长
```

### 3. 使用有意义的通过条件

```rust
// Good
.pass_condition("coverage >= 80 && critical_issues == 0")

// Bad
.pass_condition("true")  // 总是通过
```

### 4. 配置适当的严重程度

```rust
// 编译错误应该是 Error
severity: Severity::Error,

// 代码风格可以是 Warning
severity: Severity::Warning,

// 文档提示可以是 Info
severity: Severity::Info,
```

### 5. 质量门策略

```rust
// 开发阶段可以宽松一些
GateStrategy::WarningsAllowed { max_warnings: 10 }

// 生产环境必须严格
GateStrategy::AllMustPass
```

### 6. 定期更新检查器

```rust
// 依赖检查应该定期运行
GenericCheckType::DependenciesValid {}

// 安全扫描应该持续进行
GenericCheckType::SecurityScan { scanner: "cargo-audit".to_string() }
```

---

## 完整示例

### Rust 项目的完整质量配置

```rust
use devman_core::{
    QualityCheck, QualityCheckType, GenericCheckType, QualityCategory,
    QualityProfile, QualityGate, PassCondition, FailureAction, PhaseGate,
    GateStrategy, QualityProfileId,
};
use devman_quality::{CustomCheckBuilder, QualityProfileBuilder, OutputParser};

// 1. 创建内置检查
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

let test_check = QualityCheck {
    id: QualityCheckId::new(),
    name: "测试检查".to_string(),
    description: "运行所有测试并检查覆盖率".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::TestsPass {
        test_suite: "".to_string(),
        min_coverage: Some(80.0),
    }),
    severity: Severity::Error,
    category: QualityCategory::Testing,
};

let lint_check = QualityCheck {
    id: QualityCheckId::new(),
    name: "Lint 检查".to_string(),
    description: "运行 clippy 进行代码质量检查".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::LintsPass {
        linter: "clippy".to_string(),
    }),
    severity: Severity::Warning,
    category: QualityCategory::Maintainability,
};

let doc_check = QualityCheck {
    id: QualityCheckId::new(),
    name: "文档检查".to_string(),
    description: "检查必需文档是否存在".to_string(),
    check_type: QualityCheckType::Generic(GenericCheckType::DocumentationExists {
        paths: vec!["README.md".to_string(), "API.md".to_string()],
    }),
    severity: Severity::Warning,
    category: QualityCategory::Documentation,
};

// 2. 创建自定义检查（覆盖率）
let coverage_check = CustomCheckBuilder::new("coverage-check")
    .description("检查测试覆盖率是否达标")
    .command("cargo")
    .arg("tarpaulin")
    .arg("--out")
    .arg("Json")
    .output_parser(OutputParser::JsonPath {
        path: "report.metrics.line_coverage".to_string(),
    })
    .pass_condition("value >= 80")
    .severity(Severity::Error)
    .category(QualityCategory::Testing)
    .build();

// 3. 创建质量门
let commit_gate = QualityGate {
    name: "提交门".to_string(),
    description: "代码提交前必须通过的质量检查".to_string(),
    checks: vec![compile_check.id, test_check.id, lint_check.id],
    pass_condition: PassCondition::AllPassed,
    on_failure: FailureAction::Block,
};

let release_gate = QualityGate {
    name: "发布门".to_string(),
    description: "发布前必须通过的质量检查".to_string(),
    checks: vec![
        compile_check.id,
        test_check.id,
        lint_check.id,
        doc_check.id,
        coverage_check.id,
    ],
    pass_condition: PassCondition::AllPassed,
    on_failure: FailureAction::Escalate,
};

// 4. 创建质量配置
let profile = QualityProfileBuilder::new("rust-project-standard")
    .with_description("Rust 项目的标准质量配置")
    .with_default_strategy(GateStrategy::AllMustPass)
    .add_check(compile_check.id)
    .add_check(test_check.id)
    .add_check(lint_check.id)
    .add_check(doc_check.id)
    .add_check(coverage_check.id)
    .add_phase_gate(PhaseGate {
        phase: phase_id_development,
        checks: vec![compile_check.id, lint_check.id],
        strategy: GateStrategy::WarningsAllowed { max_warnings: 5 },
    })
    .add_phase_gate(PhaseGate {
        phase: phase_id_release,
        checks: vec![compile_check.id, test_check.id, lint_check.id, doc_check.id],
        strategy: GateStrategy::AllMustPass,
    })
    .build();
```

---

*最后更新: 2026-02-02*
