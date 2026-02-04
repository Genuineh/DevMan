# DevMan 知识管理指南

> 如何使用 DevMan 的知识服务来存储、检索和复用认知资产

## 目录

- [概述](#概述)
- [知识类型](#知识类型)
- [创建知识条目](#创建知识条目)
- [知识检索](#知识检索)
- [向量搜索](#向量搜索)
- [Reranker 重排序](#reranker-重排序)
- [模板系统](#模板系统)
- [知识分类](#知识分类)
- [使用统计](#使用统计)
- [最佳实践](#最佳实践)

---

## 概述

DevMan 知识服务是系统的第五层，提供知识的存储、检索和复用能力：

```
Layer 5: Knowledge Service    (知识检索与复用)
    ├── 知识存储与检索
    ├── 上下文推荐
    ├── 模板系统
    ├── 相似度匹配
    └── 知识分类增强
```

### 核心特性

- **结构化存储**: 将知识组织为可搜索的条目
- **多类型支持**: 支持经验、模式、模板等多种知识类型
- **智能检索**: 基于标签、类型、内容的灵活搜索
- **模板复用**: 可参数化的知识模板系统
- **自动分类**: 基于关键词的自动分类（无需外部向量库）

---

## 知识类型

### 1. 经验教训 (LessonLearned)

记录项目中的经验教训，避免重复犯错。

```rust
use devman_core::{Knowledge, KnowledgeType, KnowledgeContent, KnowledgeMetadata};

let lesson = Knowledge {
    id: KnowledgeId::new(),
    title: "Rust 所有权规则的常见误解".to_string(),
    knowledge_type: KnowledgeType::LessonLearned {
        lesson: "Rust 的借用检查器比想象的更智能，临时借用不会阻止修改".to_string(),
        context: "在实现数据结构时，过早尝试借用导致编译错误".to_string(),
    },
    content: KnowledgeContent {
        summary: "理解 Rust 借用检查器的工作方式".to_string(),
        detail: r#"Rust 的借用检查器基于作用域工作，而不是整个变量生命周期。

示例代码：
```rust
fn example() {
    let mut vec = vec![1, 2, 3];
    let first = &vec[0];  // 不可变借用
    vec.push(4);          // 可变借用 - 编译失败！
}
```"#.to_string(),
        examples: vec![],
        references: vec!["https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html".to_string()],
    },
    metadata: KnowledgeMetadata {
        domain: vec!["Rust".to_string(), "内存安全".to_string()],
        tech_stack: vec!["Rust".to_string()],
        scenarios: vec!["所有权".to_string(), "借用检查".to_string()],
        quality_score: 0.95,
        verified: true,
    },
    tags: vec!["rust".to_string(), "ownership".to_string(), "borrow-checker".to_string()],
    related_to: vec![],
    derived_from: vec![],
    usage_stats: UsageStats {
        times_used: 10,
        last_used: Some(chrono::Utc::now()),
        success_rate: 0.9,
        feedback: vec![],
    },
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
```

### 2. 最佳实践 (BestPractice)

记录经过验证的最佳实践。

```rust
KnowledgeType::BestPractice {
    practice: "使用 Result<T, E> 而非 panic 处理错误".to_string(),
    rationale: "Result 类型提供了明确的错误处理路径，便于测试和维护".to_string(),
}
```

### 3. 代码模式 (CodePattern)

记录可复用的代码模式。

```rust
KnowledgeType::CodePattern {
    pattern: CodeSnippet {
        language: "rust".to_string(),
        code: r#"pub struct Cache<K, V> {
    data: HashMap<K, V>,
    max_size: usize,
}

impl<K, V> Cache<K, V> {
    pub fn new(max_size: usize) -> Self {
        Self {
            data: HashMap::new(),
            max_size,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.data.len() >= self.max_size {
            // 移除最早的元素
            if let Some(oldest) = self.data.iter().next().map(|(k, _)| k.clone()) {
                self.data.remove(&oldest);
            }
        }
        self.data.insert(key, value);
    }
}"#.to_string(),
        description: "固定大小的 LRU 缓存实现".to_string(),
    },
    usage: "适用于需要限制内存使用的场景，如 API 响应缓存".to_string(),
}
```

### 4. 解决方案 (Solution)

记录特定问题的解决方案。

```rust
KnowledgeType::Solution {
    problem: "如何在 Rust 中实现异步超时？".to_string(),
    solution: r#"使用 `tokio::time::timeout`：

```rust
use tokio::time::{timeout, Duration};

async fn operation_with_timeout() -> Result<String, String> {
    match timeout(Duration::from_secs(5), slow_operation()).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Operation timed out".to_string()),
    }
}
```"#.to_string(),
    verified: true,
}
```

### 5. 模板 (Template)

可参数化的模板，支持动态生成内容。

```rust
KnowledgeType::Template {
    template: TemplateContent {
        template: r#"# {title}

## 问题描述
{description}

## 解决方案
{solution}

## 代码示例
```{language}
{code}
```

## 注意事项
{notes}
"#.to_string(),
        parameters: vec![
            TemplateParameter {
                name: "title".to_string(),
                description: "文档标题".to_string(),
                default_value: None,
                required: true,
            },
            TemplateParameter {
                name: "description".to_string(),
                description: "问题描述".to_string(),
                default_value: None,
                required: true,
            },
            TemplateParameter {
                name: "solution".to_string(),
                description: "解决方案".to_string(),
                default_value: None,
                required: true,
            },
            TemplateParameter {
                name: "language".to_string(),
                description: "代码语言".to_string(),
                default_value: Some("rust".to_string()),
                required: false,
            },
            TemplateParameter {
                name: "code".to_string(),
                description: "代码内容".to_string(),
                default_value: None,
                required: true,
            },
            TemplateParameter {
                name: "notes".to_string(),
                description: "注意事项".to_string(),
                default_value: Some("无".to_string()),
                required: false,
            },
        ],
    },
    applicable_scenarios: vec!["技术文档".to_string(), "解决方案记录".to_string()],
}
```

### 6. 决策 (Decision)

记录架构和设计决策。

```rust
KnowledgeType::Decision {
    decision: "选择 Tokio 作为异步运行时".to_string(),
    alternatives: vec![
        "async-std".to_string(),
        "smol".to_string(),
        "actix".to_string(),
    ],
    reasoning: r#"选择 Tokio 的原因：
1. 社区最活跃，文档最完善
2. 与主流框架（Actix, Axum）兼容
3. 成熟的生态系统和工具链支持
4. 性能表现优异"#.to_string(),
}
```

---

## 创建知识条目

### 基本创建流程

```rust
use devman_knowledge::BasicKnowledgeService;

let service = BasicKnowledgeService::new(storage);

// 1. 创建知识条目
let knowledge = Knowledge {
    id: KnowledgeId::new(),
    title: "标题".to_string(),
    knowledge_type: KnowledgeType::LessonLearned {
        lesson: "经验教训".to_string(),
        context: "上下文".to_string(),
    },
    // ... 其他字段
};

// 2. 保存到存储
service.create_knowledge(knowledge).await?;
```

### 使用模板创建

```rust
// 假设已有一个模板知识
let template = Knowledge {
    // ...
    knowledge_type: KnowledgeType::Template { /* ... */ },
};

let filled_template = service.fill_template(
    &template.id,
    serde_json::json!({
        "title": "新文档标题",
        "description": "问题描述",
        "solution": "解决方案",
        "language": "rust",
        "code": "fn main() {}",
        "notes": "注意事项",
    })
).await?;
```

---

## 知识检索

### 基本搜索

```rust
use devman_knowledge::{KnowledgeService, KnowledgeSearch};

// 搜索知识
let results = service.search(&KnowledgeSearch {
    query: Some("异步编程".to_string()),
    knowledge_type: None,
    tags: vec![],
    limit: 10,
}).await?;
```

### 高级搜索

```rust
// 按类型搜索
let patterns = service.search(&KnowledgeSearch {
    query: None,
    knowledge_type: Some(KnowledgeType::CodePattern),
    tags: vec!["performance".to_string()],
    limit: 20,
}).await?;

// 多条件组合
let search = KnowledgeSearch {
    query: Some("缓存".to_string()),
    knowledge_type: Some(KnowledgeType::Solution),
    tags: vec!["rust".to_string(), "性能".to_string()],
    limit: 10,
};
```

### 获取推荐

```rust
// 基于当前上下文获取推荐
let recommendations = service.get_recommendations(
    &TaskContext {
        goal_id: goal_id.clone(),
        project_id: project_id.clone(),
        phase_id: phase_id.clone(),
        parent_task_id: None,
        created_by: "ai".to_string(),
    },
    5,  // 返回5条推荐
).await?;
```

### 获取相似知识

```rust
// 查找与指定知识相似的条目
let similar = service.get_similar(knowledge_id, 5).await?;
```

### 按标签检索

```rust
// OR 逻辑：包含任一标签
let or_results = service.search_by_tags(
    &vec!["rust".to_string(), "性能".to_string()],
    false,  // OR 逻辑
    10,
).await?;

// AND 逻辑：包含所有标签
let and_results = service.search_by_tags(
    &vec!["rust".to_string(), "async".to_string()],
    true,  // AND 逻辑
    10,
).await?;
```

---

## 向量搜索

DevMan 支持基于向量的语义搜索，通过将知识内容转换为高维向量，实现语义级别的相似度匹配。

### 架构概述

```
Query → Ollama Embedding API → Query Vector
                                   ↓
Knowledge → Ollama Embedding API → Knowledge Vectors
                                   ↓
                          LocalVectorIndex (余弦相似度)
                                   ↓
                            Top 50 Candidates
```

### 配置

**环境变量**：

```bash
# Ollama 配置
DEVMAN_OLLAMA_URL=http://localhost:11434

# Embedding 模型
DEVMAN_EMBEDDING_MODEL=qwen3-embedding:0.6b  # 默认
# 或: openai-embedding, ollama {name: "other-model"}

# 向量搜索阈值 (0.0-1.0)
DEVMAN_VECTOR_THRESHOLD=0.75
```

**代码配置**：

```rust
use devman_knowledge::VectorKnowledgeServiceImpl;
use devman_core::{VectorSearchConfig, EmbeddingModel};

let config = VectorSearchConfig {
    enabled: true,
    model: EmbeddingModel::Qwen3Embedding0_6B,
    ollama_url: "http://localhost:11434".to_string(),
    dimension: 1024,
    threshold: 0.75,
};

let vector_service = VectorKnowledgeServiceImpl::new(storage.clone(), config);
vector_service.initialize().await?;
```

### 使用向量搜索

```rust
use devman_knowledge::VectorKnowledgeService;

// 简单向量搜索
let results = vector_service.search_by_vector(
    "Rust 异步编程最佳实践",
    10,     // 返回数量
    0.75,   // 相似度阈值
).await?;
```

### 混合搜索

结合关键词搜索和向量搜索：

```rust
// 先进行向量搜索获取候选
let vector_results = vector_service.search_by_vector(query, 50, 0.5).await?;

// 再进行关键词过滤
let final_results: Vec<_> = vector_results
    .into_iter()
    .filter(|r| {
        let text = r.knowledge.content.summary.to_lowercase();
        query_terms.iter().all(|t| text.contains(t))
    })
    .take(10)
    .collect();
```

### 支持的 Embedding 模型

| 模型 | 维度 | 说明 |
|------|------|------|
| `Qwen3Embedding0_6B` | 1024 | Ollama 本地模型（默认） |
| `OpenAIAda002` | 1536 | OpenAI text-embedding-ada-002 |
| `Ollama { name }` | 可变 | 其他 Ollama 模型 |

### 性能优化

```rust
// 批量保存带 embedding 的知识
for knowledge in knowledge_batch {
    vector_service.save_with_embedding(&knowledge).await?;
}

// 预计算所有知识向量（初始化时）
vector_service.initialize().await?;  // 自动计算已有知识的 embedding
```

---

## Reranker 重排序

Reranker 在向量搜索后对候选结果进行精排，显著提升检索相关性。

### 架构概述

```
Query → 向量检索 (Top 50) → Reranker 重排序 → Top 10
```

**两阶段检索优势**：
1. **粗排阶段**：向量检索快速获取候选集（50条）
2. **精排阶段**：Reranker 模型精细排序（Top 10）

### 配置

**环境变量**：

```bash
# Reranker 启用
DEVMAN_RERANKER_ENABLED=true

# Reranker 模型
DEVMAN_RERANKER_MODEL=qwen3-reranker:0.6b  # 默认
# 或: openai-reranker, ollama {name: "other-model"}

# Reranker 参数
DEVMAN_RERANKER_MAX_CANDIDATES=50   # 粗排候选数
DEVMAN_RERANKER_FINAL_TOP_K=10      # 精排返回数
```

**代码配置**：

```rust
use devman_knowledge::RerankerServiceImpl;
use devman_core::{RerankerConfig, RerankerModel};

let config = RerankerConfig {
    enabled: true,
    model: RerankerModel::Qwen3Reranker0_6B,
    ollama_url: "http://localhost:11434".to_string(),
    max_candidates: 50,
    final_top_k: 10,
};

let reranker = RerankerServiceImpl::new(config);
```

### 使用 Reranker

```rust
use devman_knowledge::HybridKnowledgeService;

let results = hybrid_service.search_hybrid(
    "Rust 错误处理最佳实践",
    50,   // 向量搜索候选数
    10,   // 最终返回数
).await?;
```

### Reranker 模型

| 模型 | 说明 |
|------|------|
| `Qwen3Reranker0_6B` | Ollama 本地模型（默认） |
| `OpenAIReranker` | OpenAI Rerank API |
| `Ollama { name }` | 其他 Ollama 模型 |

### RRF 融合（备选方案）

当 Ollama Rerank API 不可用时，可以使用 Reciprocal Rank Fusion：

```rust
use devman_knowledge::RRFusion;

let rrf = RRFusion::default();

// 多个检索方法的结果
let results1 = vector_search_results;   // 向量检索
let results2 = keyword_search_results;  // 关键词检索

// RRF 融合
let fused = rrf.fuse(&[results1, results2]);
```

---

## 模板系统

### 创建模板

```rust
let template = Knowledge {
    title: "技术设计文档模板".to_string(),
    knowledge_type: KnowledgeType::Template {
        template: TemplateContent {
            template: r#"# 技术设计文档

## 概述
{overview}

## 背景
{background}

## 设计目标
- {goal1}
- {goal2}

## 架构设计
{architecture}

## API 设计
{api_design}

## 数据模型
{data_model}

## 风险评估
{risk_assessment}
"#.to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "overview".to_string(),
                    description: "功能概述",
                    default_value: None,
                    required: true,
                },
                TemplateParameter {
                    name: "background".to_string(),
                    description: "背景和动机",
                    default_value: None,
                    required: true,
                },
                // ... 更多参数
            ],
        },
        applicable_scenarios: vec!["技术设计".to_string()],
    },
    // ... 其他字段
};
```

### 使用模板注册表

```rust
use devman_knowledge::{TemplateRegistry, TemplateBuilder};

let mut registry = TemplateRegistry::new();

// 注册模板
registry.register(template).await?;

// 列出所有模板
let all_templates = registry.list().await?;

// 按场景查找
let design_templates = registry.find_by_scenario("技术设计").await?;
```

### 模板验证

```rust
// 检查参数是否完整
let validation = registry.validate_parameters(
    &template.id,
    &parameters,
).await?;

if !validation.valid {
    return Err(validation.errors.join(", "));
}
```

---

## 知识分类

### 自动分类

知识服务会根据内容自动提取关键词进行分类：

```rust
// 提取的关键词会影响搜索结果
let knowledge = Knowledge {
    // ...
    content: KnowledgeContent {
        summary: "Rust 异步编程最佳实践".to_string(),
        detail: "详细讨论 Tokio 和 async/await 的使用...".to_string(),
        // ...
    },
    metadata: KnowledgeMetadata {
        domain: vec!["Rust".to_string(), "异步".to_string()],
        tech_stack: vec!["Tokio".to_string()],
        scenarios: vec!["网络编程".to_string()],
        // ...
    },
    tags: vec!["async".to_string(), "tokio".to_string()],
    // ...
};
```

### 分类增强

系统支持基于关键词频率的自动分类（TF-IDF 简化版）：

```rust
// 搜索时相关性评分
let results = service.search(&KnowledgeSearch {
    query: Some("rust async".to_string()),
    // ...
}).await?;
```

### 质量评分

```rust
KnowledgeMetadata {
    quality_score: 0.85,  // 0.0 - 1.0
    verified: true,       // 是否经过验证
    // ...
}
```

---

## 使用统计

### 跟踪使用情况

```rust
// 更新使用统计
service.record_usage(knowledge_id).await?;

// 获取使用统计
let stats = knowledge.usage_stats;
println!("使用次数: {}", stats.times_used);
println!("最后使用: {:?}", stats.last_used);
println!("成功率: {}%", stats.success_rate * 100.0);
```

### 添加反馈

```rust
let feedback = Feedback {
    rating: 5,  // 1-5 分
    comment: "这个解决方案很有帮助！".to_string(),
    at: chrono::Utc::now(),
    from: "user@example.com".to_string(),
};

// 添加反馈（会更新 quality_score）
service.add_feedback(knowledge_id, feedback).await?;
```

### 效果分析

```rust
// 获取高价值知识
let valuable_knowledge = service.search(&KnowledgeSearch {
    // ...
    limit: 100,
}).await?
.into_iter()
.filter(|k| k.metadata.quality_score >= 0.8)
.filter(|k| k.usage_stats.times_used >= 5)
.collect::<Vec<_>>();
```

---

## 最佳实践

### 1. 及时记录

```rust
// Good: 完成任务后立即记录
work_record.description = "完成功能实现，发现了重要的边界情况";
let lesson = Knowledge {
    title: "处理边界情况的经验".to_string(),
    knowledge_type: KnowledgeType::LessonLearned {
        lesson: "处理 X 时要注意 Y 边界情况".to_string(),
        context: "实现 Z 功能时发现".to_string(),
    },
    // ...
};

// Bad: 很久以后才记录，细节可能遗忘
```

### 2. 标签规范化

```rust
// Good: 使用统一的标签
tags: vec![
    "rust".to_string(),           // 语言（小写）
    "async".to_string(),          // 特性（小写）
    "performance".to_string(),    // 质量属性（小写）
];

// Bad: 标签不一致
tags: vec![
    "Rust".to_string(),
    "ASYNC".to_string(),
    "性能".to_string(),
];
```

### 3. 关联相关知识

```rust
// Good: 建立知识之间的关联
knowledge.related_to = vec![
    rust_async_basics_id,
    tokio_tutorial_id,
];

// 便于后续发现相关知识
let related = service.get_related(knowledge_id).await?;
```

### 4. 定期审查

```rust
// 定期检查知识质量
let outdated_knowledge = service.search(&KnowledgeSearch {
    query: None,
    knowledge_type: None,
    tags: vec![],
    limit: 1000,
}).await?
.into_iter()
.filter(|k| {
    // 超过 6 个月未更新
    let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
    k.updated_at < six_months_ago
})
.collect::<Vec<_>>();
```

### 5. 验证解决方案

```rust
// 标记已验证的知识
knowledge.metadata.verified = true;

// 标记未验证的知识（需要谨慎使用）
knowledge.metadata.quality_score = 0.5;
```

### 6. 提供上下文

```rust
// Good: 提供完整的上下文
knowledge_type: KnowledgeType::Solution {
    problem: "在多线程环境下共享状态".to_string(),
    solution: "使用 Arc<Mutex<T>> 保护共享状态".to_string(),
    verified: true,
}

// Bad: 缺少上下文的知识难以复用
knowledge_type: KnowledgeType::Solution {
    problem: "状态共享".to_string(),
    solution: "用 Mutex".to_string(),
    verified: false,
}
```

### 7. 代码示例要完整

```rust
// Good: 完整可运行的示例
CodeSnippet {
    language: "rust".to_string(),
    code: r#"fn main() {
    let result = do_something();
    println!("Result: {:?}", result);
}

fn do_something() -> i32 {
    42
}"#.to_string(),
    description: "完整的示例程序".to_string(),
}

// Bad: 片段式代码难以理解
CodeSnippet {
    language: "rust".to_string(),
    code: r#"fn do_something() -> i32 { 42 }"#.to_string(),
    description: "".to_string(),
}
```

---

## 完整示例

### AI 任务完成后自动记录知识

```rust
async fn complete_task_and_record_knowledge(
    task: &Task,
    work_record: &WorkRecord,
    knowledge_service: &impl KnowledgeService,
) -> Result<(), String> {
    // 1. 分析工作记录，提取知识点
    let extracted_knowledge = analyze_work_record(work_record)?;

    // 2. 创建知识条目
    for knowledge in extracted_knowledge {
        knowledge_service.create_knowledge(knowledge).await?;
    }

    // 3. 更新相关知识的关联
    if let Some(parent_knowledge_id) = &task.context.parent_task_id {
        knowledge_service.add_related_link(
            parent_knowledge_id,
            &extracted_knowledge[0].id,
        ).await?;
    }

    Ok(())
}

fn analyze_work_record(work_record: &WorkRecord) -> Result<Vec<Knowledge>, String> {
    let mut knowledge_items = Vec::new();

    // 从工作记录中提取知识
    if work_record.result == WorkResult::Success {
        // 记录成功的模式
        if work_record.description.contains("发现") ||
           work_record.description.contains("解决") {
            knowledge_items.push(Knowledge {
                title: format!("问题解决: {}", work_record.description),
                knowledge_type: KnowledgeType::Solution {
                    problem: work_record.description.clone(),
                    solution: "已成功实现解决方案".to_string(),
                    verified: true,
                },
                // ... 其他字段
                ..Default::default()
            });
        }
    }

    Ok(knowledge_items)
}
```

---

*最后更新: 2026-02-04*
