# Plan: 向量检索支持知识服务 - 语义搜索能力

## 目标
为 DevMan 知识服务添加基于向量的语义搜索能力，使用 Qwen3-Embedding-0.6B 作为本地 Embedding 模型。

## 技术选型
- **Embedding 模型**: Qwen3-Embedding-0.6B (Ollama 本地运行)
- **向量存储**: 简化的本地向量索引 (ndarray + ball tree 或简单暴力搜索)
- **接口**: Ollama REST API

## 实现步骤

### Step 1: 添加配置和类型定义
- [ ] 在 `crates/core/src/` 添加向量相关类型
  - `EmbeddingModel` enum (Qwen3, OpenAI, Local)
  - `KnowledgeEmbedding` struct (存储 Knowledge ID -> 向量映射)
  - `VectorSearchConfig` 配置结构

### Step 2: 创建向量服务模块
- [ ] 在 `crates/knowledge/src/` 创建 `vector.rs`
  - `VectorKnowledgeService` trait
  - `OllamaEmbeddingClient` - Ollama API 客户端
  - `LocalVectorIndex` - 简化向量索引实现
  - `VectorKnowledgeServiceImpl` - 主实现

### Step 3: 扩展 KnowledgeService
- [ ] 在 `service.rs` 添加向量搜索方法
  - `search_semantic_with_vector()` - 纯向量搜索
  - `search_hybrid()` - 关键词 + 向量混合搜索

### Step 4: 添加配置支持
- [ ] 在 CLI 和 MCP Server 添加配置
- [ ] 环境变量: `DEVMAN_OLLAMA_URL`

### Step 5: 集成测试
- [ ] 测试 Ollama 连接
- [ ] 测试 Embedding 生成
- [ ] 测试向量搜索

## 文件变更

```
crates/core/src/
  + knowledge.rs (扩展 KnowledgeEmbedding, EmbeddingModel)

crates/knowledge/src/
  + vector.rs (新文件 - 向量服务)
  ~ lib.rs (导出新模块)
  ~ service.rs (扩展 KnowledgeService)

Cargo.toml
  + dependencies: reqwest, ndarray (可选)
```

## API 设计

```rust
// Ollama Embedding 客户端
async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    // POST to Ollama /api/embeddings
}

// 向量搜索服务
async fn search_by_vector(
    &self,
    query: &str,
    limit: usize,
    threshold: f32,
) -> Result<Vec<(Knowledge, f32)>> {
    // 1. 生成查询向量
    // 2. 暴力搜索或索引搜索
    // 3. 返回相似度 > threshold 的结果
}

// 混合搜索
async fn search_hybrid(
    &self,
    query: &str,
    limit: usize,
) -> Result<Vec<Knowledge>> {
    // 1. 关键词搜索 (现有功能)
    // 2. 向量搜索
    // 3. RRF 融合排名
}
```

## Ollama 集成

```bash
# 启动 Ollama
ollama run qwen3-embedding:0.6b

# 或使用 OpenAI 兼容接口
OLLAMA_EMBEDDINGS=True ollama serve
```

## 配置示例

```toml
[knowledge.vector]
enabled = true
model = "qwen3-embedding:0.6b"
ollama_url = "http://localhost:11434"
dimension = 1024
threshold = 0.75
```

## 性能考虑

- Embedding 缓存 (memory-mapped 文件)
- 批量生成 embedding
- 简化向量索引 (小规模数据用暴力搜索即可)
