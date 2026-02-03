# DevMan MCP Server 使用指南

> 如何在 VSCode 和 Claude Code 中配置使用 DevMan MCP Server

---

## 概述

DevMan MCP Server 为 AI 助手提供工作管理、质量检查、知识检索等能力。通过 MCP 协议，AI 可以结构化地与 DevMan 系统交互。

---

## 在 Claude Code 中使用

### 方式一：直接运行（推荐）

```bash
# 启动 MCP Server（stdio 模式，Claude Code 会自动连接）
cargo run -p devman-ai

# 或使用 Unix Socket 模式
cargo run -p devman-ai -- --socket /tmp/devman.sock
```

### 方式二：配置到 MCP Servers（Claude Code 设置）

1. 打开 Claude Code 设置（`Ctrl+,` 或 `Cmd+,`）
2. 找到 **MCP Servers** 配置
3. 添加新服务器：

```json
{
  "devman": {
    "command": "cargo",
    "args": ["run", "-p", "devman-ai", "--"],
    "env": {},
    "disabled": false
  }
}
```

### 使用示例

```python
# Claude Code 会自动加载 devman 工具
# 可以直接调用：
await devman_create_goal({"title": "实现用户认证"})
await devman_get_task_guidance({"task_id": "task_xxx"})
await devman_search_knowledge({"query": "Rust 错误处理"})
```

---

## 在 VSCode 中使用

### 前提条件

1. 安装 **MCP Client** 扩展（如：Model Context Protocol）
2. 或使用 VSCode 的 **Claude Extension**（内置 MCP 支持）

### 配置步骤

#### 方法一：VSCode MCP 配置（settings.json）

打开 VSCode 设置，添加：

```json
{
  "mcp.servers": {
    "devman": {
      "command": "cargo",
      "args": ["run", "-p", "devman-ai", "--"],
      "cwd": "/path/to/DevMan"
    }
  }
}
```

#### 方法二：使用 MCP Proxy（推荐用于生产）

如果你已经安装了 `devman-ai` 二进制文件：

```bash
# 安装到 PATH
cargo install --path crates/ai --force

# 配置 VSCode
{
  "mcp.servers": {
    "devman": {
      "command": "devman-ai",
      "args": ["--socket", "/tmp/devman.sock"],
      "disabled": false
    }
  }
}
```

### 验证连接

1. 重载 VSCode 窗口（`Ctrl+Shift+P` → "Reload Window"）
2. 打开 MCP Client 面板，查看连接状态
3. 测试工具调用：

```
→ devman_get_context({})
← {"success": true, "data": {"message": "Context retrieval..."}}
```

---

## 工具调用示例

### 1. 创建目标

```json
{
  "name": "devman_create_goal",
  "arguments": {
    "title": "实现用户认证功能",
    "description": "添加 JWT 基础认证",
    "success_criteria": [
      "登录功能测试通过",
      "安全性检查通过"
    ]
  }
}
```

### 2. 获取任务引导

```json
{
  "name": "devman_get_task_guidance",
  "arguments": {
    "task_id": "task_01jhvp5q2c1f00000006"
  }
}
```

**响应示例：**
```json
{
  "success": true,
  "data": {
    "task_id": "task_01jhvp5q2c1f00000006",
    "current_state": "Created",
    "next_action": "read_context",
    "guidance_message": "请调用 devman_read_task_context() 读取任务上下文",
    "allowed_operations": ["devman_read_task_context"]
  }
}
```

### 3. 任务工作流

```json
// 1. 读取上下文
{
  "name": "devman_read_task_context",
  "arguments": {"task_id": "task_xxx"}
}

// 2. 查询相关知识
{
  "name": "devman_review_knowledge",
  "arguments": {
    "task_id": "task_xxx",
    "query": "JWT Rust authentication"
  }
}

// 3. 确认学习完成
{
  "name": "devman_confirm_knowledge_reviewed",
  "arguments": {
    "task_id": "task_xxx",
    "knowledge_ids": ["kn_xxx"]
  }
}

// 4. 开始执行
{
  "name": "devman_start_execution",
  "arguments": {"task_id": "task_xxx"}
}

// 5. 记录工作
{
  "name": "devman_log_work",
  "arguments": {
    "task_id": "task_xxx",
    "action": "modified",
    "description": "实现了 JWT token 生成和验证",
    "files": ["src/auth/jwt.rs"]
  }
}

// 6. 提交工作
{
  "name": "devman_finish_work",
  "arguments": {
    "task_id": "task_xxx",
    "description": "完成 JWT 认证模块"
  }
}

// 7. 运行质检
{
  "name": "devman_run_task_quality_check",
  "arguments": {
    "task_id": "task_xxx",
    "check_types": ["compile", "test", "lint"]
  }
}
```

---

## 完整工具列表

| 类别 | 工具 | 说明 |
|------|------|------|
| **目标** | `devman_create_goal` | 创建目标 |
| | `devman_get_goal_progress` | 获取进度 |
| **引导** | `devman_get_task_guidance` | 获取任务引导 |
| | `devman_read_task_context` | 读取上下文 |
| **知识** | `devman_review_knowledge` | 查询知识 |
| | `devman_confirm_knowledge_reviewed` | 确认学习 |
| | `devman_search_knowledge` | 搜索知识库 |
| **执行** | `devman_start_execution` | 开始执行 |
| | `devman_log_work` | 记录工作 |
| | `devman_finish_work` | 提交工作 |
| **质检** | `devman_run_task_quality_check` | 运行质检 |
| | `devman_get_quality_result` | 获取结果 |
| | `devman_confirm_quality_result` | 确认结果 |
| **控制** | `devman_complete_task` | 完成任务 |
| | `devman_pause_task` | 暂停任务 |
| | `devman_resume_task` | 恢复任务 |
| | `devman_abandon_task` | 放弃任务 |
| **其他** | `devman_list_blockers` | 列出阻塞 |
| | `devman_get_job_status` | 任务状态 |
| | `devman_cancel_job` | 取消任务 |

---

## 故障排除

### 1. 连接失败

```bash
# 检查是否有进程占用端口
lsof -i :8080

# 检查 socket 文件
ls -la /tmp/devman.sock
```

### 2. 工具未加载

```bash
# 重新构建
cargo build -p devman-ai

# 检查工具列表
cargo run -p devman-ai 2>&1 | head -20
```

### 3. 权限问题

```bash
# 确保 socket 文件权限
chmod 777 /tmp/devman.sock
```

---

## 推荐工作流

```
1. 创建目标 → devman_create_goal
2. 获取引导 → devman_get_task_guidance
3. 按引导操作 → 按 next_action 调用相应工具
4. 遇到问题 → devman_search_knowledge 查找知识
5. 完成工作 → devman_run_task_quality_check
6. 记录知识 → devman_save_knowledge
```

---

## 相关文档

- [MCP API 完整参考](./MCP_API.md)
- [项目 README](./README.md)
- [设计文档](./DESIGN.md)
- [架构指南](./ARCHITECTURE.md)

---

最后更新：2026-02-03
