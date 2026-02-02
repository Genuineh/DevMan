# DevMan 贡献指南

> 感谢您考虑为 DevMan 做出贡献！

## 目录

- [欢迎](#欢迎)
- [行为准则](#行为准则)
- [开始贡献](#开始贡献)
- [开发环境设置](#开发环境设置)
- [开发流程](#开发流程)
- [代码风格](#代码风格)
- [测试要求](#测试要求)
- [提交规范](#提交规范)
- [Pull Request 流程](#pull-request-流程)
- [文档贡献](#文档贡献)
- [问题反馈](#问题反馈)

---

## 欢迎

DevMan 是一个开源项目，欢迎各种形式的贡献：

- 🐛 **Bug 修复**
- ✨ **新功能**
- 📝 **文档改进**
- 🎨 **代码优化**
- 💡 **设计建议**
- 🧪 **测试补充**

---

## 行为准则

请遵循以下行为准则：

1. **尊重** - 尊重他人的观点和贡献
2. **包容** - 欢迎新人和不同背景的贡献者
3. **建设性** - 提供有帮助的反馈
4. **专注** - 关注项目目标和愿景

---

## 开始贡献

### 选择任务

1. 查看 [Issues](https://github.com/jerryg/DevMan/issues) 标签
2. 查找 `good first issue` 标签（适合新手）
3. 查找 `help wanted` 标签（需要帮助）

### 声明任务

在开始工作前：

1. 在 Issue 下留言说明您要处理
2. 或者创建新的 Issue 描述您要解决的问题
3. 等待维护者确认后开始

---

## 开发环境设置

### 前置条件

```bash
# Rust (使用 rustup 安装)
rustc --version  # >= 1.70
cargo --version  # >= 1.70

# Git
git --version

# (可选) 其他工具
# - fd (文件查找)
# - ripgrep (代码搜索)
```

### 克隆仓库

```bash
# 克隆仓库
git clone https://github.com/jerryg/DevMan.git
cd DevMan

# 添加上游仓库（用于同步）
git remote add upstream https://github.com/jerryg/DevMan.git
```

### 构建项目

```bash
# 构建所有 crate
cargo build --workspace

# 运行测试
cargo test --workspace

# 运行 linter
cargo clippy --workspace

# 格式化代码
cargo fmt --workspace
```

### 本地开发

```bash
# 构建 CLI
cargo build -p devman-cli

# 运行 CLI
cargo run -p devman-cli -- --help

# 运行 MCP Server
cargo run -p devman-ai -- --help
```

---

## 开发流程

### 1. 创建分支

```bash
# 确保主分支是最新的
git checkout main
git fetch upstream
git merge upstream/main

# 创建功能分支
git checkout -b feature/your-feature-name
# 或
git checkout -b fix/bug-description
```

### 2. 开发

```bash
# 编写代码
# ... 编辑文件 ...

# 检查代码
cargo clippy
cargo fmt -- --check

# 运行测试
cargo test -p <affected-crate>
```

### 3. 提交

```bash
# 查看更改
git status
git diff

# 添加更改
git add <files>

# 提交（遵循提交规范）
git commit -m "type(scope): description"
```

### 4. 推送

```bash
# 推送到您的 fork
git push origin feature/your-feature-name
```

### 5. 创建 PR

在 GitHub 上创建 Pull Request。

---

## 代码风格

### 格式化

使用 `cargo fmt` 自动格式化：

```bash
# 格式化所有文件
cargo fmt --workspace

# 检查格式
cargo fmt --workspace -- --check
```

### Lint

使用 `cargo clippy` 检查：

```bash
# 检查所有 crate
cargo clippy --workspace -- -D warnings

# 修复 clippy 建议
cargo clippy --workspace --fix
```

### 命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `QualityCheck` |
| 枚举 | PascalCase | `QualityCategory` |
| 函数 | snake_case | `run_check` |
| 变量 | snake_case | `check_id` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_TIMEOUT` |
| 模块 | snake_case | `quality_engine` |
| Trait | PascalCase | `QualityEngine` |

### 文档要求

```rust
/// 单行文档注释
pub struct MyStruct {
    /// 字段文档
    pub field: Type,
}

/// 多行文档
///
/// # Examples
///
/// ```
/// let s = MyStruct::new();
/// ```
```

---

## 测试要求

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(expected, actual);
    }
}
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p devman-quality

# 运行 doc 测试
cargo test --doc

# 运行带输出的测试
cargo test -- --nocapture
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率
cargo tarpaulin -o html
```

### 测试要求

- 所有新功能必须有测试
- Bug 修复必须有回归测试
- 保持测试快速（< 1秒）

---

## 提交规范

### 格式

```
type(scope): description

body (可选)

footer (可选)
```

### 类型 (Type)

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档修改 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构（既不是新功能也不是 Bug 修复） |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建工具或辅助工具修改 |

### 作用域 (Scope)

| crate | 作用域 |
|-------|--------|
| `crates/core` | core |
| `crates/storage` | storage |
| `crates/quality` | quality |
| `crates/knowledge` | knowledge |
| `crates/progress` | progress |
| `crates/work` | work |
| `crates/tools` | tools |
| `crates/ai` | ai |
| `crates/cli` | cli |
| `docs` | docs |

### 示例

```
feat(quality): add security scan checker

Implement cargo-audit integration for security scanning.

Closes #123
```

```
fix(core): resolve GoalId parsing issue

Handle invalid ULID format gracefully.

Co-Authored-By: Name <email@example.com>
```

---

## Pull Request 流程

### 创建 PR

1. 访问 [Pull Requests](https://github.com/jerryg/DevMan/pulls)
2. 点击 "New Pull Request"
3. 选择您的分支
4. 填写模板（会自动填充）

### PR 模板

```markdown
## 描述
<!-- 描述您做了什么 -->

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 破坏性变更
- [ ] 文档更新

## 测试
- [ ] 我添加了测试
- [ ] 所有测试通过
- [ ] 测试覆盖率没有下降

## 检查清单
- [ ] 代码遵循项目风格
- [ ] 代码已格式化
- [ ] Linter 检查通过
- [ ] 文档已更新（如果需要）
```

### Review 流程

1. **自动化检查** - CI 会运行测试和 linter
2. **人工 Review** - 维护者会审查代码
3. **反馈** - 可能会收到修改建议
4. **修改** - 根据反馈更新代码
5. **合并** - 通过后合并到主分支

### 加速 Review

- ✅ 保持 PR 小而专注
- ✅ 添加清晰的描述
- ✅ 包含测试
- ✅ 遵循代码风格
- ❌ 不要一次提交太多无关更改

---

## 文档贡献

### 文档类型

| 文档 | 位置 | 说明 |
|------|------|------|
| API 文档 | `docs/API.md` | API 参考 |
| 设计文档 | `docs/DESIGN.md` | 架构设计 |
| 质检指南 | `docs/QUALITY_GUIDE.md` | 质检扩展 |
| 知识指南 | `docs/KNOWLEDGE.md` | 知识管理 |
| 架构文档 | `docs/ARCHITECTURE.md` | 架构详解 |
| 贡献指南 | `docs/CONTRIBUTING.md` | 本文档 |
| TODO | `docs/TODO.md` | 开发路线图 |

### 文档风格

```markdown
# 使用标题层级

## 二级标题
### 三级标题

- 使用列表
- 保持简洁

```rust
// 代码示例
fn example() {
    // ...
}
```
```

### 更新文档

```bash
# 编辑文档
vim docs/API.md

# 本地预览（使用 markdown 预览工具）
# 或直接在 GitHub 上预览
```

---

## 问题反馈

### Bug 报告

使用 Issue 模板：

```markdown
## 描述
<!-- 描述 Bug -->

## 复现步骤
1. 步骤一
2. 步骤二
3. 错误发生

## 预期行为
<!-- 应该发生什么 -->

## 实际行为
<!-- 实际发生了什么 -->

## 环境
- OS: [e.g., Linux]
- Rust 版本: [e.g., 1.70]
- DevMan 版本: [e.g., 0.1.0]

## 日志
<!-- 相关的错误日志 -->
```

### 功能建议

```markdown
## 功能描述
<!-- 描述您想要的功能 -->

## 使用场景
<!-- 为什么需要这个功能 -->

## 可能的实现
<!-- 您的想法（可选） -->

## 备选方案
<!-- 其他解决方案（可选） -->
```

---

## 常见问题

### Q: 如何运行单个测试？

```bash
cargo test -p devman-quality test_function_name
```

### Q: 如何查看编译警告？

```bash
cargo build --message-format=short
```

### Q: 代码风格有问题怎么办？

```bash
cargo fmt --workspace
cargo clippy --workspace --fix
```

### Q: 如何同步上游更改？

```bash
git fetch upstream
git merge upstream/main
```

### Q: 需要帮助怎么办？

1. 查看文档（docs/ 目录）
2. 查看 [API.md](API.md)
3. 在 Issue 中提问

---

## 资源链接

- [GitHub 仓库](https://github.com/jerryg/DevMan)
- [Issues](https://github.com/jerryg/DevMan/issues)
- [Pull Requests](https://github.com/jerryg/DevMan/pulls)
- [Rust 文档](https://doc.rust-lang.org/)
- [Tokio 教程](https://tokio.rs/tokio/tutorial)

---

## 维护者

- 项目负责人: Jerry

---

感谢您的贡献！

*最后更新: 2026-02-02*
