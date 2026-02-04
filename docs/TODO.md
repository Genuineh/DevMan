# DevMan 开发规划 v4

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

## 当前待办

### 📋 实现 MCP Server 协议对接

基于 `docs/plans/2026-02-02-mcp-server-design.md` 设计文档完善 MCP Server：

- [x] **工具接口对接**
  - [x] 实现 `devman_create_goal` → AIInterface.create_goal()
  - [x] 实现 `devman_list_tasks` → AIInterface.list_tasks()
  - [x] 实现 `devman_get_job_status` → JobManager 查询接口
  - [x] 实现 `devman_cancel_job` → JobManager 取消接口

- [x] **资源返回完善**
  - [x] 对接 `devman://context/project` → 项目配置和状态
  - [x] 对接 `devman://context/goal` → 活跃目标及进度
  - [x] 对接 `devman://tasks/{view}` → 任务队列/历史
  - [x] 对接 `devman://knowledge/{view}` → 知识库查询
  - [x] 资源响应添加 version/etag 字段

- [x] **异步任务管理**
  - [x] 实现 `JobManager` Trait 和默认实现
  - [x] 实现 `create_job()` / `get_job_status()` / `cancel_job()`
  - [x] 同步执行（timeout ≤ 30s）与异步执行（timeout > 30s）
  - [x] 异步任务持久化快照（jobs.json）

- [x] **错误处理**
  - [x] 实现自定义错误码（-32000 ~ -32004）
  - [x] 错误响应添加 hint 和 retryable 字段
  - [x] 保证异步任务错误与 job.status 一致性

- [x] **AIInterface 扩展**
  - [x] 新增 `create_goal(spec)` 方法
  - [x] 新增 `list_tasks(filter)` 方法
  - [x] 实现返回值资源化（返回 URI 而非大体量数据）

- [x] **测试**
  - [x] 编写 MCP Server 集成测试
  - [x] 测试 stdio 和 unix socket 传输
  - [x] 测试同步/异步执行模式
  - [x] 测试错误处理和资源版本化
  - [x] **E2E 测试** (6 个测试用例)
    - [x] `test_e2e_create_and_list_task` - 创建和列出任务
    - [x] `test_e2e_task_workflow` - 完整任务工作流
    - [x] `test_e2e_create_multiple_tasks` - 创建多个任务
    - [x] `test_e2e_create_task_with_phase` - 带阶段的任务创建
    - [x] `test_e2e_search_knowledge` - 知识搜索
    - [x] `test_e2e_get_goal_progress_no_goal` - 获取目标进度（无目标）

---

## Crate 结构

```
devman/
├── Cargo.toml
├── crates/
│   ├── core/                    # 核心数据模型
│   ├── storage/                 # 存储层
│   ├── knowledge/               # 知识服务 (Layer 5)
│   ├── quality/                 # 质量保证 (Layer 4)
│   ├── progress/                # 进度追踪 (Layer 3)
│   ├── work/                    # 工作管理 (Layer 2)
│   ├── tools/                   # 工具集成
│   ├── ai/                      # AI 接口
│   │   ├── interface.rs          # AIInterface
│   │   ├── interactive.rs       # 交互式 AI
│   │   ├── validation.rs        # 状态验证
│   │   ├── guidance.rs          # 任务引导
│   │   └── mcp_server.rs        # MCP 服务器
│   └── cli/                     # 命令行
└── docs/
    ├── DESIGN.md
    ├── API.md
    ├── QUALITY_GUIDE.md
    ├── KNOWLEDGE.md
    ├── ARCHITECTURE.md
    ├── CONTRIBUTING.md
    ├── plans/
    │   └── 2026-02-02-mcp-server-design.md
    └── archive/
        └── v3-2026-02-02.md     # 历史归档
```

---

## 设计原则

1. **质检可扩展** - 通用 + 业务 + 人机协作
2. **知识可复用** - 检索、模板、推荐
3. **工具化执行** - 减少 token 消耗
4. **进度可视化** - 目标 → 阶段 → 任务
5. **存储抽象** - 可替换存储后端
6. **AI 友好** - 结构化接口
7. **可追溯性** - 完整工作日志
8. **轻量级** - 文件式存储，无外部依赖

---

## 历史归档

| 版本 | 日期 | 链接 |
|------|------|------|
| v3 | 2026-02-02 | [docs/archive/v3-2026-02-02.md](./archive/v3-2026-02-02.md) |

**v3 归档内容**:
- Phase 1-8 完整实现（核心模型、存储、质量、知识、进度、工作、工具、AI接口）
- 所有核心文档（DESIGN.md, API.md, QUALITY_GUIDE.md, KNOWLEDGE.md, ARCHITECTURE.md, CONTRIBUTING.md）

---

*最后更新: 2026-02-04*

---

## 已完成功能

### feat: 向量检索支持知识服务 - 语义搜索能力 ✅

**GitHub Issue**: #3

**实现状态**：已完成

**新增文件**：
- `crates/knowledge/src/vector.rs` - 向量服务模块

**新增类型** (`crates/core/src/knowledge.rs`)：
- `EmbeddingModel` - Embedding 模型类型 (Qwen3, OpenAI, Ollama)
- `VectorSearchConfig` - 向量搜索配置
- `KnowledgeEmbedding` - 知识向量缓存
- `ScoredKnowledge` - 带相似度分数的知识项

**新增 Storage 方法** (`crates/storage/src/trait_.rs`)：
- `save_vector_embedding()` - 保存向量
- `load_vector_embedding()` - 加载向量
- `list_vector_embeddings()` - 列出所有向量

**核心组件**：
- `OllamaEmbeddingClient` - Ollama Embedding API 客户端
- `LocalVectorIndex` - 本地向量索引 (Cosine Similarity)
- `VectorKnowledgeService` - 向量搜索服务 trait
- `VectorKnowledgeServiceImpl` - 默认实现

**使用方式**：
```rust
use devman_knowledge::{VectorKnowledgeService, VectorKnowledgeServiceImpl};
use devman_storage::JsonStorage;

let storage = Arc::new(Mutex::new(JsonStorage::new(&storage_path).await?));
let config = VectorSearchConfig {
    enabled: true,
    model: EmbeddingModel::Qwen3Embedding0_6B,
    ollama_url: "http://localhost:11434".to_string(),
    dimension: 1024,
    threshold: 0.75,
};

let vector_service = VectorKnowledgeServiceImpl::new(storage, config);
vector_service.initialize().await?;

// 搜索
let results = vector_service.search_by_vector("error handling", 10, 0.75).await?;
```

**配置**（通过环境变量）：
- `DEVMAN_OLLAMA_URL` - Ollama 服务地址（默认 http://localhost:11434）
- `DEVMAN_EMBEDDING_MODEL` - Embedding 模型（默认 qwen3-embedding:0.6b）
- `DEVMAN_VECTOR_THRESHOLD` - 相似度阈值（默认 0.75）

---

## 待规划功能

### feat: Reranker 重排序支持 - 检索质量优化

**背景**：
- 向量搜索基于语义相似性，但可能遗漏细粒度相关性
- 需要 reranker 模型对候选结果进行精排
- Qwen3-Reranker-0.6B 专为重排序设计，推理开销小

**方案**：

#### 1. 两阶段检索架构
```
Query → 向量检索 (Top 50) → Reranker 重排序 → Top 10
```

#### 2. Reranker 模型集成
```rust
pub enum RerankerModel {
    /// Qwen3 Reranker (Ollama local)
    Qwen3Reranker0_6B,
    /// OpenAI text-embedding-3-small (reranking endpoint)
    OpenAIReranker,
    /// Custom Ollama model
    Ollama { name: String },
}

/// Ollama Reranker Client
pub struct OllamaRerankerClient {
    client: Client,
    url: String,
    model: String,
}

impl OllamaRerankerClient {
    /// Rerank documents given a query
    /// Returns scores for each document
    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<f32>> {
        // Ollama /api/rerank endpoint or use embeddings + cross-encoding
    }
}
```

#### 3. 配置扩展
```rust
pub struct RerankerConfig {
    /// Enable reranking
    pub enabled: bool,
    /// Reranker model
    pub model: RerankerModel,
    /// Ollama server URL
    pub ollama_url: String,
    /// Max candidates to rerank (after vector search)
    pub max_candidates: usize,
    /// Final top-k results after reranking
    pub final_top_k: usize,
}

impl Default for RerankerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            model: RerankerModel::Qwen3Reranker0_6B,
            ollama_url: "http://localhost:11434".to_string(),
            max_candidates: 50,
            final_top_k: 10,
        }
    }
}
```

#### 4. 混合搜索 API
```rust
#[async_trait]
pub trait HybridKnowledgeService: Send + Sync {
    /// Hybrid search with vector + reranking
    async fn search_hybrid(
        &self,
        query: &str,
        vector_top_k: usize,      // Vector search candidates
        rerank_top_k: usize,      // Final results after reranking
    ) -> Result<Vec<ScoredKnowledge>>;
}
```

#### 5. RRF 融合（备选方案）
如果 Ollama 不支持 reranking endpoint，可使用 Reciprocal Rank Fusion：
```rust
fn rrf_fuse(
    vector_results: &[ScoredKnowledge],
    keyword_results: &[Knowledge],
    k: u32,
) -> Vec<ScoredKnowledge> {
    // Combine results from different retrieval methods
}
```

**优先级**：中 - 向量搜索已可用，reranking 是优化增强

**依赖**：
- [Qwen3 Reranker](https://huggingface.co/Qwen/Qwen3-Reranker-0.6B)
- Ollama rerank API 或交叉编码方式

---

## 待规划功能

#### 2. 数据模型扩展
```rust
pub struct KnowledgeEmbedding {
    pub knowledge_id: String,
    pub embedding: Vec<f32>,        // 1536 维 (OpenAI) 或 768 维 (本地模型)
    pub model: EmbeddingModel,
}

pub enum EmbeddingModel {
    OpenAIAda002,      // OpenAI text-embedding-ada-002
    LocalBGE,           // BAAI/bge-base-en-v1.5 (本地运行)
    LocalMiniLM,        // sentence-transformers/all-MiniLM-L6-v2
}
```

#### 3. API 设计
```rust
#[async_trait]
pub trait VectorKnowledgeService: Send + Sync {
    /// 保存知识并生成 embedding
    async fn save_with_embedding(&self, knowledge: Knowledge) -> Result<()>;

    /// 向量相似度搜索
    async fn search_by_vector(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<Knowledge>>;

    /// 混合搜索（关键词 + 向量）
    async fn search_hybrid(&self, query: &str, limit: usize) -> Result<Vec<Knowledge>>;
}
```

**优先级**：高 - 语义搜索是 AI 知识服务的核心能力

**参考**：
- [Qdrant Client](https://github.com/qdrant/qdrant-client)
- [BGE Embeddings](https://huggingface.co/BAAI/bge-base-en-v1.5)
