# DevMan MCP Server 设计文档

> 版本: 1.0
> 日期: 2026-02-02
> 状态: 草稿

## 概述

本文档定义 DevMan MCP Server 的完整设计方案。MCP (Model Context Protocol) 是 AI Assistant 与 DevMan 系统交互的标准接口协议，采用 JSON-RPC 2.0 作为传输层基础，兼容官方 MCP 2025-11-05 规范的工具与资源格式。

当前 MCP Server 框架已实现协议层和传输层，但业务逻辑仍为占位实现。本设计文档旨在明确协议规范、工具映射、执行策略、资源定义、错误处理以及架构设计，为后续实现提供完整指导。

## 1. 协议架构

### 1.1 传输协议

DevMan MCP Server 基于 JSON-RPC 2.0 标准构建，采用请求-响应模式进行通信。协议设计遵循兼容性与可扩展性原则，核心传输层与业务逻辑分离，便于适配不同的传输方式。

请求格式采用标准的 JSON-RPC 2.0 结构，包含 jsonrpc 版本标识、请求 id、方法名以及参数对象。响应格式同样遵循 JSON-RPC 2.0 规范，包含对应的 id、结果数据或错误信息。所有消息均采用 UTF-8 编码，通过换行符分隔的 JSON 行进行传输。

当前实现基于 MCP 2025-11-05 规范，兼容该版本及后续向后兼容版本。此设计确保未来协议升级时无需破坏现有客户端的兼容性，只需在适配器层处理版本协商即可。

### 1.2 消息格式

请求消息示例：

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "method": "tools/call",
  "params": {
    "name": "devman_create_task",
    "arguments": {
      "title": "实现用户认证功能",
      "description": "添加 JWT 基础认证",
      "goal_id": "goal_01jhvp5q2c1a00000001"
    }
  }
}
```

成功响应示例：

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "result": {
    "success": true,
    "data": {
      "task_id": "task_01jhvp5q2c1b00000002",
      "title": "实现用户认证功能",
      "status": "Created"
    },
    "version": "task_01jhvp5q2c1b00000002@v1"
  }
}
```

错误响应示例将在错误处理章节详细说明。

## 2. 工具调用映射

### 2.1 工具注册表

DevMan MCP Server 内置 12 个核心工具，覆盖目标管理、任务管理、知识管理、质量检查、工具执行以及上下文查询等核心功能。每个工具都定义了明确的输入模式（JSON Schema），确保 AI Assistant 传递的参数类型安全且完整。

工具与 AIInterface 的映射关系如下表所示。已实现的方法直接对接 AIInterface 接口，未实现的方法需要扩展 AIInterface 或在 MCP Server 层实现业务逻辑。

| MCP 工具名称 | 对应接口方法 | 实现状态 |
|-------------|-------------|---------|
| devman_create_goal | 需新增 create_goal(spec) | 待实现 |
| devman_get_goal_progress | get_progress(goal_id) | 已实现 |
| devman_create_task | create_task(spec) | 已实现 |
| devman_list_tasks | 需新增 list_tasks(filter) | 待实现 |
| devman_search_knowledge | search_knowledge(query) | 已实现 |
| devman_save_knowledge | save_knowledge(knowledge) | 已实现 |
| devman_run_quality_check | run_quality_check(check) | 已实现 |
| devman_execute_tool | execute_tool(tool, input) | 已实现 |
| devman_get_context | get_current_context() | 已实现 |
| devman_list_blockers | list_blockers() | 已实现 |
| devman_get_job_status | 需新增 JobManager 接口 | 待实现 |
| devman_cancel_job | 需新增 JobManager 接口 | 待实现 |

### 2.2 工具输入模式

以下为各核心工具的输入模式定义，采用 JSON Schema 格式描述。这些模式不仅用于协议解析，也为 AI Assistant 提供了参数结构的类型提示。

**devman_create_goal 工具：**

```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "目标标题，简明扼要地描述目标内容"
    },
    "description": {
      "type": "string",
      "description": "目标的详细描述，包括背景和期望成果"
    },
    "success_criteria": {
      "type": "array",
      "items": { "type": "string" },
      "description": "成功标准列表，用于判定目标是否完成"
    },
    "project_id": {
      "type": "string",
      "description": "关联的项目 ID，可选"
    }
  },
  "required": ["title"]
}
```

**devman_create_task 工具：**

```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "任务标题，应描述具体的可执行工作项"
    },
    "description": {
      "type": "string",
      "description": "任务的详细说明，包括验收标准和注意事项"
    },
    "goal_id": {
      "type": "string",
      "description": "关联的目标 ID"
    },
    "phase_id": {
      "type": "string",
      "description": "关联的阶段 ID"
    },
    "priority": {
      "type": "integer",
      "minimum": 1,
      "maximum": 5,
      "description": "任务优先级，1 为最高优先级"
    }
  },
  "required": ["title", "goal_id"]
}
```

**devman_execute_tool 工具：**

```json
{
  "type": "object",
  "properties": {
    "tool": {
      "type": "string",
      "enum": ["cargo", "git", "npm", "fs", "bash"],
      "description": "要执行的工具类型"
    },
    "command": {
      "type": "string",
      "description": "要执行的命令名称或子命令"
    },
    "args": {
      "type": "array",
      "items": { "type": "string" },
      "description": "命令参数列表"
    },
    "timeout": {
      "type": "integer",
      "minimum": 0,
      "description": "超时时间（秒），0 表示无超时，null 使用默认 30 秒"
    },
    "async_mode": {
      "type": "boolean",
      "description": "显式指定异步执行模式，可选"
    },
    "env": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "环境变量，可选"
    },
    "working_dir": {
      "type": "string",
      "description": "执行工作目录，可选"
    }
  },
  "required": ["tool", "command"]
}
```

## 3. 混合工具执行策略

### 3.1 同步与异步模式

DevMan MCP Server 采用超时参数决定执行模式的混合策略。对于短时任务（默认 30 秒以内），采用同步阻塞模式，AI Assistant 等待执行完成后直接返回结果。对于长时任务或需要后台执行的操作，采用异步模式，返回任务 ID 供 AI 轮询状态。

这种设计平衡了交互延迟与操作复杂度的需求。短任务同步执行避免了轮询带来的额外开销，长任务异步执行则防止 AI 会话长时间阻塞。AI 可以根据返回的 job_id 决定是继续等待还是执行其他任务。

执行流程如下：AI 调用工具时传递 timeout 参数，若超时时间不超过 30 秒或 async_mode 为 false，则同步执行并立即返回结果。若超时时间超过 30 秒或 async_mode 为 true，则创建异步任务并返回 job_id，AI 可通过 devman_get_job_status 轮询执行状态。

### 3.2 异步任务管理

异步任务由 JobManager 组件统一管理，JobManager 负责创建任务、跟踪状态、返回结果以及清理过期任务。JobManager 实现为 Trait 接口，便于单元测试时的 Mock 以及未来替换不同的调度策略。

同步执行响应格式如下：

```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "Build succeeded",
  "stderr": "",
  "duration_ms": 1523
}
```

异步执行响应格式如下：

```json
{
  "success": true,
  "async": true,
  "job_id": "job_01jhvp5q2c1c00000003",
  "status": "running",
  "created_at": "2026-02-02T10:00:00Z",
  "timeout_seconds": 300
}
```

轮询任务状态响应格式如下：

```json
{
  "success": true,
  "job_id": "job_01jhvp5q2c1c00000003",
  "status": "completed",
  "started_at": "2026-02-02T10:00:00Z",
  "completed_at": "2026-02-02T10:05:23Z",
  "result": {
    "exit_code": 0,
    "stdout": "...",
    "stderr": "",
    "duration_ms": 323456
  }
}
```

若任务执行失败，状态更新为 failed 并包含错误信息：

```json
{
  "success": true,
  "job_id": "job_01jhvp5q2c1c00000003",
  "status": "failed",
  "error": {
    "code": -32003,
    "message": "Job execution timeout",
    "duration_ms": 300000
  }
}
```

## 4. 资源 URI 定义

### 4.1 资源规范

DevMan MCP Server 提供按需读取的资源接口，AI Assistant 可通过资源 URI 获取项目状态信息。资源采用版本化设计，每个资源响应包含 version 或 etag 字段，支持变更检测、条件查询以及乐观并发控制。

所有资源均为只读，修改操作通过工具调用完成。资源设计遵循层级分明、语义明确的原则，便于 AI 理解和查询。

### 4.2 URI 规范与响应格式

**devman://context/project**

此资源返回当前项目的配置和状态信息，包括项目 ID、名称、根路径、配置内容等。适用于 AI 需要了解项目全局上下文场景。

响应格式：

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

**devman://context/goal**

此资源返回当前活跃目标及其进度信息。根据设计约束，在任一时刻此资源至多返回一个 active goal。若没有活跃目标，则返回空数据。适用于 AI 需要了解当前工作焦点的场景。

响应格式：

```json
{
  "version": "goal_01jhvp5q2c1e00000005@v12",
  "data": {
    "goal_id": "goal_01jhvp5q2c1e00000005",
    "title": "完成 MCP Server 实现",
    "description": "实现 DevMan 的 MCP 协议接口",
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

**devman://tasks/{view}**

此资源返回任务集合，通过 view 参数区分不同的任务视图。支持三种视图：queue（待办和进行中任务）、completed（已完成任务）、history（全部历史）。可通过查询参数过滤和限制返回结果。

响应格式：

```json
{
  "version": "tasks@v28",
  "data": {
    "tasks": [
      {
        "task_id": "task_01jhvp5q2c1f00000006",
        "title": "完善工具调用映射",
        "status": "InProgress",
        "priority": 1,
        "goal_id": "goal_01jhvp5q2c1e00000005",
        "progress": {
          "percentage": 40.0,
          "current_step": 2,
          "total_steps": 5
        }
      }
    ],
    "total_count": 8,
    "view": "queue"
  }
}
```

支持查询参数：
- state：按状态过滤，如 state=InProgress
- limit：限制返回数量，如 limit=10
- goal_id：按目标筛选
- phase_id：按阶段筛选

**devman://knowledge/{view}**

此资源返回知识库内容，通过 view 参数区分不同的知识视图。支持三种视图：recent（最近更新）、all（全部知识）、by-tag/{tag}（按标签筛选）。知识资源支持类型过滤和时间排序。

响应格式：

```json
{
  "version": "knowledge@v42",
  "data": {
    "knowledge": [
      {
        "knowledge_id": "kn_01jhvp5q2c1g00000007",
        "title": "Rust 错误处理最佳实践",
        "type": "BestPractice",
        "tags": ["rust", "error-handling"],
        "created_at": "2026-02-01T15:30:00Z"
      }
    ],
    "total_count": 15,
    "view": "recent"
  }
}
```

支持查询参数：
- type：按知识类型过滤，如 type=BestPractice
- limit：限制返回数量
- tags：按标签筛选（多标签为 AND 关系）

**devman://task/{task_id}**

获取单个任务的详细信息，包括完整的任务规格、执行步骤、质检门等信息。

**devman://goal/{goal_id}**

获取单个目标的详细信息，包括所有阶段、任务列表、进度百分比等。

**devman://job/{job_id}**

获取异步任务的执行状态和结果，用于轮询异步任务。

**devman://quality/report**

获取项目整体质量报告，包括各模块的检查状态、通过率、警告统计等。

### 4.3 版本字段作用

资源响应中的 version 字段具有多重作用。变更检测方面，AI 可以比较两次请求的 version 是否变化来判断数据是否更新。条件查询方面，支持 If-None-Match 请求头实现条件 GET，减少不必要的数据传输。乐观并发控制方面，提交状态变更时可以携带 version，防止并发修改冲突。

version 字段的格式为 `{resource_type}_{resource_id}@v{version_number}`，或使用内容的哈希值作为 etag。两种方式各有优劣，序号方式易于理解和管理，哈希方式则更紧凑且能检测内容变化。

## 5. 错误处理规范

### 5.1 错误响应格式

所有错误响应遵循 JSON-RPC 2.0 标准格式，包含错误码、错误消息以及可选的扩展数据。扩展数据中的 hint 字段为 AI 提供修复建议，减少 AI 的试错轮次。retryable 字段指示错误是否可重试，帮助 AI 决定是否自动重试。

错误响应格式示例：

```json
{
  "jsonrpc": "2.0",
  "id": "1",
  "error": {
    "code": -32001,
    "message": "Cannot start task in Completed state",
    "data": {
      "hint": "Task is already completed. Create a new task instead.",
      "retryable": false,
      "task_id": "task_01jhvp5q2c1f00000006"
    }
  }
}
```

### 5.2 错误码规范

DevMan MCP Server 的错误码分为两类：JSON-RPC 标准错误码（-32700 至 -32600）以及 DevMan 自定义业务错误码（-32000 至 -32099）。

JSON-RPC 标准错误码：

| 错误码 | 含义 | 典型场景 |
|--------|------|---------|
| -32700 | JSON 解析错误 | 请求体不是有效的 JSON |
| -32600 | 无效请求 | 缺少必需字段或结构错误 |
| -32601 | 方法不存在 | 调用了未注册的工具或资源 |
| -32602 | 参数无效 | 参数类型错误或值超出范围 |
| -32603 | 内部错误 | 服务器内部异常，如存储失败 |

DevMan 自定义业务错误码：

| 错误码 | 类型 | 含义 | 示例 |
|--------|------|------|------|
| -32000 | 业务 | 通用业务错误 | 权限拒绝、操作不允许 |
| -32001 | 中断 | 状态冲突 | 任务已结束、目标已完成 |
| -32002 | 中断 | 资源不存在 | 错误的 ID、找不到对象 |
| -32003 | 可重试 | 异步任务超时 | 任务执行超过超时限制 |
| -32004 | 可重试 | 异步任务被取消 | 用户主动取消执行 |

### 5.3 AI 处理策略

根据错误类型，AI 应采取不同的处理策略。协议错误（-32700 至 -32602）表示请求本身存在问题，AI 修复请求格式后可以重试。内部错误（-32603）通常是临时性故障，AI 可以指数退避后重试。

中断型错误（-32001 状态冲突、-32002 资源不存在）表示请求在当前状态下无法执行，AI 不应自动重试，而应调整请求参数或逻辑。可重试型错误（-32003 超时、-32004 被取消）可以根据具体情况决定是否重试，对于超时错误可以适当延长超时时间后重试。

### 5.4 异步任务错误的一致性

当异步任务执行失败时，错误响应与 job 资源的状态应保持一致。AI 不应同时收到错误响应和 job.status 状态的冲突信息。

错误响应：

```json
{
  "error": {
    "code": -32003,
    "message": "Job execution timeout"
  }
}
```

对应的 job 资源状态：

```json
{
  "status": "failed",
  "error": {
    "code": -32003,
    "message": "Job execution timeout"
  }
}
```

这种一致性设计确保 AI 能够可靠地获取任务状态，无需担心信息冲突。

### 5.5 错误码扩展规则

JSON-RPC 规范定义 -32000 至 -32099 用于服务器自定义错误。当前 DevMan 已使用 -32000 至 -32004，将来新增业务错误应顺延使用 -32005、-32006 等编号，避免与现有错误码冲突。

扩展错误码时，应同时更新本文档和错误处理代码，在错误数据中提供足够的上下文信息以便调试和问题排查。

## 6. 架构设计

### 6.1 分层架构

DevMan MCP Server 采用清晰的分层架构设计，从上到下依次为：传输层、中间件层、请求处理层、接口抽象层、核心模块层以及存储层。各层职责明确，依赖关系清晰，便于独立测试和维护。

传输层负责与外部 AI Assistant 的通信，支持多种传输方式。当前已实现 stdio 和 unix socket 两种本地传输方式，未来可扩展支持 HTTP 和 WebSocket 以满足 Web-based AI 的需求。传输层的可切换设计使得同一套业务逻辑可以适配不同的部署场景。

中间件层位于传输层与请求处理层之间，负责版本协商、认证授权、日志追踪等横切关注点。当前版本该层为预留扩展点，未来多 Agent 并行场景下需要实现权限控制机制。

请求处理层负责协议解析、请求路由以及响应序列化。工具调用被路由到对应的处理函数，处理结果被封装为标准的 JSON-RPC 响应格式。该层不包含业务逻辑，仅负责协议的编解码工作。

接口抽象层定义了 AIInterface 和 JobManager 两个核心 Trait，便于单元测试时的 Mock 以及未来替换不同的实现。各核心模块（Goals、Tasks、Knowledge 等）同样定义为 Trait 接口，实现关注点分离。

核心模块层包含业务逻辑实现，负责管理项目中的目标、任务、知识、质量检查等核心概念。各模块通过 Trait 接口对外提供功能，依赖倒置原则使得上层不直接依赖具体实现。

存储层采用 JsonStorage 实现，将所有数据以 JSON 文件形式保存在 .devman 目录下。jobs.json 由 JobManager 写入快照，核心模块不直接访问异步任务状态，确保调度层与业务层的分离。

### 6.2 组件关系

```
AI Assistant
    │
    ▼ JSON-RPC 2.0
┌─────────────────────────────────────────┐
│         Transport Layer                  │
│   stdio / unix socket / http / ws       │
└─────────────────────────────────────────┘
    │
    ▼ (Auth & Versioning Middleware)
┌─────────────────────────────────────────┐
│         Request Handler                  │
│   parse / route / serialize             │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│       Trait: AIInterface                 │
│   get_current_context() → URI           │
│   search_knowledge() → URI              │
│   create_task() → URI                   │
│   run_quality_check() → URI             │
│   execute_tool() → result/URI           │
└─────────────────────────────────────────┘
    │
┌─────────────────────────────────────────┐
│       Trait: JobManager                  │
│   create_job() → job_id                 │
│   get_job_status() → status/result      │
│   cancel_job()                          │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│      Core Modules (Trait)                │
│   Goals / Tasks / Knowledge / Quality    │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│        JsonStorage                       │
│   .devman/                               │
│   goals.json / tasks.json / knowledge.*  │
└─────────────────────────────────────────┘
```

### 6.3 设计原则

DevMan MCP Server 的架构设计遵循以下核心原则：

接口 Trait 化原则要求 AIInterface、JobManager 以及各 Core Module 均为 Trait 接口。这一设计便于单元测试时使用 Mock 对象替换真实实现，也便于未来替换不同的实现策略。多线程与多 Agent 并行场景下，Trait 接口确保了代码的可测试性和可扩展性。

返回值资源化原则要求所有接口方法尽量返回资源 URI 而非大体量数据。这一设计避免了大尺寸 JSON 数据在接口层传输导致的阻塞问题，也与第四部分的资源化策略保持一致。AI 需要具体数据时再通过资源 URI 查询。

JobManager 解耦原则要求异步任务状态存储在 JobManager 内存中，同时写入持久化快照。核心模块不直接访问 jobs.json，仅负责管理业务状态。这种设计确保调度逻辑与业务逻辑的分离，便于独立演进。

传输层可切换原则使得同一套业务逻辑可以适配不同的传输方式。stdio 适用于本地 CLI 和桌面应用，unix socket 适用于 MCP Client SDKs，HTTP 和 WebSocket 则为未来的 Web-based AI 和多 Agent 并行场景预留。

Auth 扩展预留原则要求在架构中预留认证授权的位置。版本协商机制确保客户端与服务器的协议兼容性，认证授权机制为多 Agent 并行场景提供权限控制基础。

## 7. 未来扩展规划

### 7.1 协议版本演进

当前实现基于 MCP 2025-11-05 规范，该规范采用日期式版本号。未来的协议演进应遵循以下原则：向后兼容的变更（如新增工具或资源）可在同一主版本内完成，不破坏现有行为；破坏性变更（如修改现有工具的语义）需要新的版本号，并通过适配器层处理版本协商。

协议演进策略确保现有客户端在新版本服务器上仍能正常工作，只需在适配器层处理版本差异。反之，新版本客户端也能够在旧版本服务器上降级运行（使用双方都支持的特性子集）。

### 7.2 多 Agent 并行支持

DevMan 的最终目标是支持多个 AI Agent 并行工作，这需要在以下方面进行扩展：

任务分配方面，Job 需要添加 agent_id 字段以标识执行任务的 Agent。调度策略需要支持公平分配、负载均衡等高级策略。

状态隔离方面，资源查询需要支持 agent_scope 参数，使 Agent 只能看到授权范围内的数据。这防止一个 Agent 看到另一个 Agent 的敏感信息。

权限控制方面，Auth Middleware 需要实现基于角色的访问控制（RBAC）。不同角色（如 Owner、Contributor、Observer）拥有不同的操作权限。

冲突检测方面，Job 和 Resource 需要添加 lock_version 字段支持乐观锁。当多个 Agent 同时修改同一资源时，系统能够检测冲突并提供解决建议。

### 7.3 语义知识检索

当前知识检索基于关键词匹配，未来可扩展支持语义检索功能。通过向量嵌入技术，将知识内容转换为高维向量，支持基于相似度的检索。语义检索能够发现关键词不同但语义相近的知识，提供更精准的相关性推荐。

实现语义检索需要在知识存储时额外保存向量索引，检索时使用向量相似度算法计算相关性。可以通过配置项控制是否启用语义检索，关键词检索作为后备方案。

### 7.4 传输层扩展

当前已支持 stdio 和 unix socket 两种本地传输方式。未来的传输层扩展规划如下：

HTTP 传输方式适用于 Web-based AI 场景，通过 RESTful 接口暴露 MCP 协议。需要处理 HTTP 连接管理、请求路由以及跨域问题。

WebSocket 传输方式适用于需要实时推送的场景。通过 WebSocket 连接，Server 可以主动向 Client 推送状态变更通知，减少 AI 的轮询开销。

### 7.5 可观测性

当前版本已集成 tracing 进行日志追踪，记录请求响应、工具执行耗时以及错误链路。未来可扩展的监控能力包括：

指标采集方面，可以统计工具调用成功率、异步任务完成率、端到端延迟等关键指标。这些指标有助于了解系统性能和稳定性。

分布式追踪方面，如果未来支持多实例部署，需要实现跨实例的追踪上下文传播，便于排查分布式环境下的性能瓶颈和问题。

告警机制方面，当错误率超过阈值或任务超时频繁时，触发告警通知运维人员及时介入。

## 附录

### 工具完整列表

| 工具名称 | 描述 | 输入参数 |
|---------|------|---------|
| devman_create_goal | 创建新目标 | title, description, success_criteria, project_id |
| devman_get_goal_progress | 获取目标进度 | goal_id |
| devman_create_task | 创建新任务 | title, description, goal_id, phase_id, priority |
| devman_list_tasks | 列出任务 | state, limit, goal_id, phase_id |
| devman_search_knowledge | 搜索知识库 | query, limit |
| devman_save_knowledge | 保存知识 | title, knowledge_type, content, tags |
| devman_run_quality_check | 运行质量检查 | check_type, target |
| devman_execute_tool | 执行工具 | tool, command, args, timeout, async_mode, env |
| devman_get_context | 获取工作上下文 | 无 |
| devman_list_blockers | 列出阻塞项 | 无 |
| devman_get_job_status | 获取任务状态 | job_id |
| devman_cancel_job | 取消任务 | job_id |

### 资源完整列表

| 资源 URI | 描述 |
|---------|------|
| devman://context/project | 项目上下文 |
| devman://context/goal | 当前活跃目标 |
| devman://tasks/queue | 待办任务队列 |
| devman://tasks/completed | 已完成任务 |
| devman://tasks/history | 任务历史 |
| devman://knowledge/recent | 最近知识 |
| devman://knowledge/all | 全部知识 |
| devman://knowledge/by-tag/{tag} | 按标签筛选 |
| devman://task/{task_id} | 单个任务详情 |
| devman://goal/{goal_id} | 单个目标详情 |
| devman://job/{job_id} | 异步任务状态 |
| devman://quality/report | 质量报告 |

### 参考资料

- JSON-RPC 2.0 规范：https://www.jsonrpc.org/specification
- MCP 协议规范：Anthropic 官方文档
- DevMan 项目架构：docs/ARCHITECTURE.md
