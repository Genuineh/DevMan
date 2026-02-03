# DevMan MCP Server API Reference

> Version: 1.0.0
> Last Updated: 2026-02-03
> Protocol: MCP 2024-11-05 (JSON-RPC 2.0)

## 概述

DevMan MCP Server 为 AI Assistant 提供标准化的接口，用于管理项目目标、任务、知识、质量检查等核心功能。通过 MCP 协议，AI 可以结构化地与 DevMan 系统交互，实现工作管理的自动化。

### 传输方式

- **stdio**：标准输入输出，适用于本地 CLI 和桌面应用
- **Unix Socket**：`/tmp/devman.sock`，适用于 MCP Client SDKs

### 连接示例

```bash
# stdio 模式
cargo run -p devman-ai

# Unix Socket 模式
cargo run -p devman-ai -- --socket /tmp/devman.sock
```

---

## 工具 (Tools)

### Goal Management

#### devman_create_goal

创建新目标。

**输入参数：**

```json
{
  "title": "string",              // 目标标题（必需）
  "description": "string",         // 目标描述（可选）
  "success_criteria": ["string"],  // 成功标准列表（可选）
  "project_id": "string"           // 关联项目 ID（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "goal_id": "goal_01jhvp5q2c1a00000001",
    "title": "目标标题",
    "status": "Active"
  },
  "version": "goal_01jhvp5q2c1a00000001@v1"
}
```

**错误码：**

| 码值 | 场景 |
|------|------|
| -32602 | 缺少必需参数 title |
| -32000 | 创建目标失败 |

---

#### devman_get_goal_progress

获取目标进度。

**输入参数：**

```json
{
  "goal_id": "string"  // 目标 ID（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "goal_id": "goal_01jhvp5q2c1a00000001",
    "percentage": 65.0,
    "completed_phases": ["设计", "框架"],
    "active_tasks": 3,
    "completed_tasks": 5
  }
}
```

**错误码：**

| 码值 | 场景 |
|------|------|
| -32602 | 缺少必需参数 goal_id |
| -32002 | 目标不存在 |

---

### Task Management

#### devman_create_task

创建新任务。

**输入参数：**

```json
{
  "title": "string",       // 任务标题（必需）
  "description": "string", // 任务描述（可选）
  "goal_id": "string",     // 关联目标 ID（可选）
  "phase_id": "string",    // 关联阶段 ID（可选）
  "priority": 1            // 优先级 1-5（可选，1 为最高）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1b00000002",
    "title": "任务标题",
    "status": "Created"
  },
  "version": "task_01jhvp5q2c1b00000002@v1"
}
```

---

#### devman_list_tasks

列出任务，支持多种筛选条件。

**输入参数：**

```json
{
  "state": "string",  // 状态筛选：Created, InProgress, Completed, Abandoned（可选）
  "goal_id": "string", // 按目标筛选（可选）
  "phase_id": "string", // 按阶段筛选（可选）
  "limit": 10          // 最大返回数量（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "task_id": "task_01jhvp5q2c1f00000006",
        "title": "完善工具调用映射",
        "status": "InProgress",
        "priority": 1,
        "goal_id": "goal_01jhvp5q2c1e00000005"
      }
    ],
    "total_count": 8
  },
  "version": "tasks@v28"
}
```

---

### Knowledge Management

#### devman_search_knowledge

搜索知识库。

**输入参数：**

```json
{
  "query": "string",  // 搜索查询（必需）
  "limit": 10         // 最大返回数量（可选，默认 10）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "results": [
      {
        "knowledge_id": "kn_01jhvp5q2c1g00000007",
        "title": "Rust 错误处理最佳实践",
        "knowledge_type": "BestPractice",
        "tags": ["rust", "error-handling"]
      }
    ],
    "total_count": 15
  }
}
```

---

#### devman_save_knowledge

保存知识到知识库。

**输入参数：**

```json
{
  "title": "string",                    // 知识标题（必需）
  "knowledge_type": "string",           // 类型：LessonLearned, BestPractice, CodePattern, Solution, Template, Decision（必需）
  "content": "string",                  // 知识内容（必需）
  "tags": ["string"]                    // 标签（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "knowledge_id": "kn_01jhvp5q2c1h00000008",
    "title": "知识标题"
  },
  "version": "kn_01jhvp5q2c1h00000008@v1"
}
```

---

### Quality Assurance

#### devman_run_quality_check

运行质量检查。

**输入参数：**

```json
{
  "check_type": "string",  // 检查类型：compile, test, lint, format, doc（必需）
  "target": "string"       // 目标（如测试套件名）（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "passed": true,
    "execution_time_ms": 1523,
    "findings_count": 0
  }
}
```

**错误码：**

| 码值 | 场景 |
|------|------|
| -32602 | 缺少必需参数 check_type |

---

### Tool Execution

#### devman_execute_tool

执行构建/开发工具。

**输入参数：**

```json
{
  "tool": "string",        // 工具类型：cargo, git, npm, fs, bash（必需）
  "command": "string",     // 命令（必需）
  "args": ["string"],      // 参数列表（可选）
  "timeout": 300,          // 超时秒数（可选，默认 30）
  "env": {"string": "string"}, // 环境变量（可选）
  "working_dir": "string"  // 工作目录（可选）
}
```

**同步执行响应（timeout ≤ 30s）：**

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "Build succeeded",
  "stderr": "",
  "duration_ms": 1523
}
```

**异步执行响应（timeout > 30s）：**

```json
{
  "success": true,
  "async": true,
  "job_id": "job_01jhvp5q2c1c00000003",
  "status": "running"
}
```

---

### Context & Progress

#### devman_get_context

获取当前工作上下文。

**输入参数：**

```json
{}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "message": "Context retrieval - use devman://context/* resources"
  }
}
```

---

#### devman_list_blockers

列出当前阻塞项。

**输入参数：**

```json
{}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "blockers": [
      {
        "reason": "等待前置任务完成",
        "severity": "Warning"
      }
    ],
    "total_count": 1
  }
}
```

---

### Async Job Management

#### devman_get_job_status

获取异步任务状态。

**输入参数：**

```json
{
  "job_id": "string"  // 任务 ID（必需）
}
```

**响应（运行中）：**

```json
{
  "success": true,
  "data": {
    "job_id": "job_01jhvp5q2c1c00000003",
    "status": "running",
    "progress": 45,
    "progress_message": "Running tests..."
  }
}
```

**响应（已完成）：**

```json
{
  "success": true,
  "data": {
    "job_id": "job_01jhvp5q2c1c00000003",
    "status": "completed",
    "completed_at": "2026-02-02T10:05:23Z",
    "result": {
      "exit_code": 0,
      "stdout": "..."
    }
  }
}
```

**响应（失败）：**

```json
{
  "success": true,
  "data": {
    "job_id": "job_01jhvp5q2c1c00000003",
    "status": "failed",
    "error": {
      "code": -32003,
      "message": "Job execution timeout"
    }
  }
}
```

**错误码：**

| 码值 | 场景 |
|------|------|
| -32602 | 缺少必需参数 job_id |
| -32002 | 任务不存在 |
| -32001 | 无法取消非运行中任务 |

---

#### devman_cancel_job

取消异步任务。

**输入参数：**

```json
{
  "job_id": "string"  // 任务 ID（必需）
}
```

**响应（成功）：**

```json
{
  "success": true,
  "message": "Job job_01jhvp5q2c1c00000003 cancelled"
}
```

**错误响应：**

```json
{
  "success": false,
  "error": {
    "code": -32004,
    "message": "Job cancelled by user",
    "data": {
      "hint": "The job was cancelled. You can retry or create a new job."
    },
    "retryable": true
  }
}
```

---

### Task Guidance (任务引导)

DevMan 提供完整的任务引导系统，AI 助手应按照系统引导的流程完成任务。

#### devman_get_task_guidance

获取任务当前状态及下一步引导。**AI 每次操作前都应该调用此接口。**

**输入参数：**

```json
{
  "task_id": "string"  // 任务 ID（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "current_state": "Created",
    "next_action": "read_context",
    "guidance_message": "请调用 devman_read_task_context() 读取任务上下文",
    "allowed_operations": ["devman_read_task_context"],
    "prerequisites_satisfied": true,
    "missing_prerequisites": [],
    "health": "healthy"
  }
}
```

**响应字段说明：**

| 字段 | 说明 |
|------|------|
| `current_state` | 当前任务状态：Created, ContextRead, KnowledgeReviewed, InProgress, WorkRecorded, QualityChecking, QualityCompleted, Paused, Abandoned, Completed |
| `next_action` | 下一步操作：read_context, review_knowledge, start_execution, log_work, submit_work, run_quality_check, confirm_result, complete_task |
| `guidance_message` | 系统给出的引导消息 |
| `allowed_operations` | 当前状态允许的操作列表 |
| `health` | 任务健康状态：healthy, warning, attention, critical |

---

#### devman_read_task_context

读取任务上下文（Created → ContextRead）。

**输入参数：**

```json
{
  "task_id": "string"  // 任务 ID（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "state": "ContextRead",
    "task_info": {
      "title": "实现用户认证功能",
      "description": "添加 JWT 基础认证",
      "goal_id": "goal_01jhvp5q2c1e00000005"
    },
    "project": {
      "name": "DevMan",
      "tech_stack": ["Rust", "Tokio"]
    }
  }
}
```

---

#### devman_review_knowledge

根据任务查询相关知识。

**输入参数：**

```json
{
  "task_id": "string",  // 任务 ID（必需）
  "query": "string"     // 知识搜索查询（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "knowledge_items": [
      {
        "knowledge_id": "kn_01jhvp5q2c1h00000008",
        "title": "Rust JWT 认证最佳实践",
        "type": "BestPractice",
        "summary": "使用 jsonwebtoken crate 实现 JWT 认证",
        "relevance_score": 0.95
      }
    ],
    "total_count": 3
  }
}
```

---

#### devman_confirm_knowledge_reviewed

确认知识学习完成（ContextRead → KnowledgeReviewed）。

**输入参数：**

```json
{
  "task_id": "string",        // 任务 ID（必需）
  "knowledge_ids": ["string"] // 已学习的知识 ID 列表（必需）
}
```

---

#### devman_start_execution

开始任务执行（KnowledgeReviewed → InProgress）。

**输入参数：**

```json
{
  "task_id": "string"  // 任务 ID（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "state": "InProgress",
    "session_id": "session_task_01jhvp5q2c1f00000006",
    "message": "开始执行，请使用 devman_log_work() 记录工作进展"
  }
}
```

---

#### devman_log_work

记录工作进展（执行过程中可多次调用）。

**输入参数：**

```json
{
  "task_id": "string",   // 任务 ID（必需）
  "action": "string",    // 操作类型：created, modified, tested, documented, debugged, refactored（必需）
  "description": "string", // 工作描述（必需）
  "files": ["string"]    // 涉及的文件列表（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "recorded": true,
    "log_id": "log_01jhvp5q2c1i00000009"
  }
}
```

---

#### devman_finish_work

提交工作成果（InProgress → WorkRecorded）。

**输入参数：**

```json
{
  "task_id": "string",       // 任务 ID（必需）
  "description": "string",   // 工作总结（必需）
  "artifacts": [             // 产出物（可选）
    {
      "name": "auth.rs",
      "type": "code",
      "path": "src/auth.rs"
    }
  ],
  "lessons_learned": "string" // 学到的经验（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "state": "WorkRecorded",
    "record_id": "record_01jhvp5q2c1j0000000a",
    "next_action": "run_quality_check"
  }
}
```

---

#### devman_run_task_quality_check

运行任务质检（WorkRecorded → QualityChecking）。

**输入参数：**

```json
{
  "task_id": "string",         // 任务 ID（必需）
  "check_types": ["string"]    // 质检类型：compile, test, lint, format, doc（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "state": "QualityChecking",
    "check_id": "check_01jhvp5q2c1k0000000b",
    "message": "质检运行中，请使用 devman_get_quality_result() 获取结果"
  }
}
```

---

#### devman_get_quality_result

获取质检结果。

**输入参数：**

```json
{
  "check_id": "string"  // 质检 ID（必需）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "check_id": "check_01jhvp5q2c1k0000000b",
    "status": "completed",
    "overall_status": "passed",
    "findings_count": 0,
    "warnings_count": 2,
    "findings": [
      {
        "severity": "warning",
        "message": "函数未添加文档注释",
        "location": "src/auth.rs:45"
      }
    ]
  }
}
```

---

#### devman_confirm_quality_result

确认质检结果并决定下一步操作（QualityCompleted）。

**输入参数：**

```json
{
  "task_id": "string",   // 任务 ID（必需）
  "check_id": "string",  // 质检 ID（必需）
  "decision": "string"   // 决策：accept_and_complete, fix_and_continue, redo_execution（必需）
}
```

---

#### devman_complete_task

完成任务（仅当质检通过时可用）。

**输入参数：**

```json
{
  "task_id": "string",         // 任务 ID（必需）
  "summary": "string",         // 完成总结（必需）
  "artifacts": [...],          // 最终产出物（可选）
  "created_knowledge_ids": ["string"] // 创建的知识 ID（可选）
}
```

**响应：**

```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "state": "Completed",
    "message": "任务已完成"
  }
}
```

---

#### devman_pause_task

暂停任务。

**输入参数：**

```json
{
  "task_id": "string",  // 任务 ID（必需）
  "reason": "string"    // 暂停原因（必需）
}
```

---

#### devman_resume_task

恢复已暂停的任务。

**输入参数：**

```json
{
  "task_id": "string"  // 任务 ID（必需）
}
```

---

#### devman_abandon_task

放弃任务（统一入口，处理所有无法继续完成的情况）。

**输入参数：**

```json
{
  "task_id": "string",   // 任务 ID（必需）
  "reason_type": "string", // 原因类型（必需）
  "reason": "string"      // 详细原因（必需）
}
```

**reason_type 取值：**

| 值 | 说明 |
|------|------|
| `voluntary` | AI/开发者主动放弃 |
| `project_cancelled` | 项目取消 |
| `goal_cancelled` | 目标取消 |
| `requirement_changed` | 需求变更 |
| `dependency_failed` | 依赖任务失败 |
| `insufficient_info` | 缺少必要信息 |
| `technical_limitation` | 技术限制 |
| `resource_unavailable` | 资源不可用 |
| `timeout` | 超时 |
| `quality_failed` | 质检持续失败 |
| `other` | 其他原因 |

**响应：**

```json
{
  "success": true,
  "data": {
    "state": "Abandoned",
    "reason_type": "voluntary",
    "reason": "对技术栈不熟悉",
    "can_be_reassigned": true,
    "work_preserved": true
  }
}
```

---

## 资源 (Resources)

资源为只读接口，用于获取项目状态信息。

### devman://context/project

获取当前项目配置和状态。

**响应格式：**

```json
{
  "version": "project@v15",
  "data": {
    "project_id": "proj_01jhvp5q2c1d00000004",
    "name": "DevMan",
    "root_path": "/home/user/DevMan",
    "config": {
      "storage_path": ".devman",
      "quality": { "enabled": true }
    }
  }
}
```

### devman://context/goal

获取当前活跃目标及进度。

**响应格式：**

```json
{
  "version": "goal_01jhvp5q2c1e00000005@v12",
  "data": {
    "goal_id": "goal_01jhvp5q2c1e00000005",
    "title": "完成 MCP Server 实现",
    "status": "Active",
    "progress": {
      "percentage": 65.0,
      "active_tasks": 3,
      "completed_tasks": 5,
      "completed_phases": ["设计", "框架"],
      "blockers": []
    },
    "current_phase": "实现",
    "success_criteria": ["协议文档完成", "工具测试通过"]
  }
}
```

### devman://tasks/queue

获取待办任务队列。

**查询参数：**
- `state`：状态筛选
- `limit`：返回数量限制
- `goal_id`：按目标筛选

**响应格式：**

```json
{
  "version": "tasks@v28",
  "data": {
    "tasks": [...],
    "total_count": 8,
    "view": "queue"
  }
}
```

### devman://knowledge/recent

获取最近更新的知识。

**查询参数：**
- `type`：知识类型筛选
- `limit`：返回数量限制
- `tags`：标签筛选

**响应格式：**

```json
{
  "version": "knowledge@v42",
  "data": {
    "knowledge": [...],
    "total_count": 15,
    "view": "recent"
  }
}
```

---

## 错误处理

### 错误码规范

| 码值 | 类型 | 含义 | 示例 |
|------|------|------|------|
| -32700 | 协议 | JSON 解析错误 | 请求体不是有效的 JSON |
| -32600 | 协议 | 无效请求 | 缺少必需字段 |
| -32601 | 协议 | 方法不存在 | 调用了未注册的工具 |
| -32602 | 协议 | 参数无效 | 参数类型错误或值超出范围 |
| -32603 | 协议 | 内部错误 | 服务器内部异常 |
| -32000 | 业务 | 通用业务错误 | 权限拒绝、操作不允许 |
| -32001 | 中断 | 状态冲突 | 任务已结束、目标已完成 |
| -32002 | 中断 | 资源不存在 | 错误的 ID、找不到对象 |
| -32003 | 可重试 | 异步任务超时 | 任务执行超过超时限制 |
| -32004 | 可重试 | 异步任务被取消 | 用户主动取消执行 |

### 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": -32001,
    "message": "Cannot start task in Completed state",
    "data": {
      "hint": "Task is already completed. Create a new task instead.",
      "retryable": false
    }
  }
}
```

### AI 处理策略

| 错误类型 | 处理策略 |
|---------|---------|
| -32700 ~ -32602 | 修复请求格式后重试 |
| -32603 | 指数退避后重试 |
| -32001, -32002 | 不自动重试，调整请求参数 |
| -32003, -32004 | 根据情况决定是否重试 |

---

## 版本与资源控制

### Version 字段

资源响应包含 `version` 或 `etag` 字段，用于：

1. **变更检测**：比较两次请求的 version 判断数据是否更新
2. **条件查询**：支持 `If-None-Match` 请求头
3. **乐观并发控制**：提交状态变更时携带 version

**version 格式：**

```
{resource_type}_{resource_id}@v{version_number}
```

示例：`goal_01jhvp5q2c1e00000005@v12`

---

## 完整工具列表

| 工具名称 | 描述 | 必需参数 |
|---------|------|---------|
| **Goal Management** | **目标管理** | |
| `devman_create_goal` | 创建新目标 | title |
| `devman_get_goal_progress` | 获取目标进度 | goal_id |
| **Task Management** | **任务管理** | |
| `devman_create_task` | 创建新任务 | title |
| `devman_list_tasks` | 列出任务 | - |
| **Task Guidance** | **任务引导** | |
| `devman_get_task_guidance` | 获取任务引导 | task_id |
| `devman_read_task_context` | 读取任务上下文 | task_id |
| `devman_review_knowledge` | 查询相关知识 | task_id, query |
| `devman_confirm_knowledge_reviewed` | 确认知识学习完成 | task_id, knowledge_ids |
| `devman_start_execution` | 开始执行 | task_id |
| `devman_log_work` | 记录工作 | task_id, action, description |
| `devman_finish_work` | 提交工作 | task_id, description |
| `devman_run_task_quality_check` | 运行质检 | task_id, check_types |
| `devman_get_quality_result` | 获取质检结果 | check_id |
| `devman_confirm_quality_result` | 确认质检结果 | task_id, check_id, decision |
| `devman_complete_task` | 完成任务 | task_id, summary |
| `devman_pause_task` | 暂停任务 | task_id, reason |
| `devman_resume_task` | 恢复任务 | task_id |
| `devman_abandon_task` | 放弃任务 | task_id, reason_type, reason |
| **Knowledge** | **知识管理** | |
| `devman_search_knowledge` | 搜索知识库 | query |
| `devman_save_knowledge` | 保存知识 | title, knowledge_type, content |
| **Quality** | **质量检查** | |
| `devman_run_quality_check` | 运行质量检查 | check_type |
| **Tools** | **工具执行** | |
| `devman_execute_tool` | 执行工具 | tool, command |
| **Context** | **上下文** | |
| `devman_get_context` | 获取工作上下文 | - |
| `devman_list_blockers` | 列出阻塞项 | - |
| **Async Jobs** | **异步任务** | |
| `devman_get_job_status` | 获取任务状态 | job_id |
| `devman_cancel_job` | 取消任务 | job_id |

---

## 快速开始示例

### Python MCP Client

```python
import json
import subprocess
import threading

def send_request(request):
    # 通过 stdio 发送请求
    proc = subprocess.Popen(
        ["cargo", "run", "-p", "devman-ai"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True
    )
    stdout, _ = proc.communicate(json.dumps(request) + "\n")
    return json.loads(stdout.strip())

# 创建目标
response = send_request({
    "jsonrpc": "2.0",
    "id": "1",
    "method": "tools/call",
    "params": {
        "name": "devman_create_goal",
        "arguments": {
            "title": "实现用户认证功能",
            "description": "添加 JWT 基础认证",
            "success_criteria": ["登录功能测试通过", "安全性检查通过"]
        }
    }
})
print(response)
```

### Node.js MCP Client

```javascript
const { spawn } = require('child_process');

const proc = spawn('cargo', ['run', '-p', 'devman-ai'], {
    stdio: ['pipe', 'pipe', 'pipe']
});

proc.stdout.on('data', (data) => {
    const response = JSON.parse(data.toString());
    console.log(response);
});

// 发送请求
const request = {
    jsonrpc: "2.0",
    id: "1",
    method: "tools/call",
    params: {
        name: "devman_search_knowledge",
        arguments: {
            query": "Rust 错误处理"
        }
    }
};
proc.stdin.write(JSON.stringify(request) + '\n');
```

---

## 相关文档

- [项目 README](../README.md)
- [MCP Server 设计文档](./plans/2026-02-02-mcp-server-design.md)
- [架构文档](./ARCHITECTURE.md)
- [API 参考](./API.md)
- [质检指南](./QUALITY_GUIDE.md)
- [知识管理](./KNOWLEDGE.md)
