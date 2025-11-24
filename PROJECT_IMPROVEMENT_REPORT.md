# 项目改进与优化报告 (Project Improvement & Optimization Report)

## 1. 项目优化与改进建议

### 后端架构 (Rust/Tauri)

1.  **代码解耦与模块化 (Decoupling & Modularization)**
    *   **现状**: `src-tauri/src/main.rs` 文件过大，包含了应用初始化、插件配置、系统托盘逻辑、深度链接处理等过多职责，属于典型的 "God Component"。
    *   **建议**: 将 `main.rs` 中的初始化逻辑拆分到独立的模块（如 `setup.rs`），将系统托盘逻辑拆分到 `tray.rs`。保持 `main.rs` 作为纯粹的入口点，仅负责组装各模块。
    *   **理由**: 提高代码可读性和可维护性，降低多人协作时的冲突概率。

2.  **数据库并发性能 (Database Concurrency)**
    *   **现状**: SQLite 默认模式下，写操作会锁住整个数据库，可能导致前端在后台同步邮件时出现界面卡顿。
    *   **建议**: 显式开启 SQLite 的 **WAL (Write-Ahead Logging)** 模式 (`PRAGMA journal_mode=WAL;`) 和 `synchronous=NORMAL`。
    *   **理由**: WAL 模式允许读写并发执行，显著提升桌面应用的响应速度，避免界面冻结。

3.  **健壮性与错误处理 (Robustness)**
    *   **现状**: 代码中存在部分 `unwrap()` 调用（例如获取窗口图标、获取窗口句柄时）。
    *   **建议**: 全面替换 `unwrap()` 为 `expect()` 或更优雅的 `match/if let` 错误处理。对于非致命错误，应记录日志而不是让应用崩溃。
    *   **理由**: 防止生产环境下因边缘情况（如窗口未找到）导致应用意外崩溃 (Panic)。

### 前端架构 (SvelteKit)

1.  **UI 组件拆分 (Component Decomposition)**
    *   **现状**: `src/routes/+page.svelte` 承担了过多的 UI 布局和业务逻辑，包含了侧边栏、邮件列表、详情页等所有内容。
    *   **建议**: 将页面拆分为独立的细粒度组件，例如 `AppSidebar.svelte`, `AppHeader.svelte`, `EmailList.svelte`, `EmailDetail.svelte`。
    *   **理由**: 降低单个文件的复杂度，提高组件复用性，利用 Svelte 的编译优化提升渲染性能。

2.  **业务逻辑抽离 (Logic Extraction)**
    *   **现状**: 部分业务逻辑（如事件监听、定时器管理）直接写在 Svelte 组件的 `<script>` 标签中。
    *   **建议**: 引入 "控制器" 模式（如 `app-controller.ts`）或使用 Svelte 5 的 `runes` (`*.svelte.ts`) 将状态管理和业务逻辑与 UI 彻底分离。
    *   **理由**: 逻辑代码更容易进行单元测试，UI 组件更专注于视图渲染。

3.  **类型安全 (Type Safety)**
    *   **现状**: 项目中使用了 TypeScript，但部分地方可能存在隐式的 `any` 或类型定义不完善。
    *   **建议**: 开启 `strict: true` 并在 CI 流程中强制运行 `npm run check`。
    *   **理由**: 在编译阶段捕获潜在的运行时错误，提高代码可靠性。

---

## 2. 测试覆盖率分析

### 后端 (Rust)
*   **估算覆盖率**: **~0%**
*   **分析**:
    *   在 `src-tauri/src` 目录下未发现明显的 `#[test]` 单元测试代码。
    *   `cargo test` 运行结果显示没有执行任何业务逻辑测试。
    *   核心业务逻辑（如 IMAP 解析、数据库操作、加密逻辑）完全缺乏自动化测试保护。

### 前端 (TypeScript/Svelte)
*   **估算覆盖率**: **< 1%**
*   **分析**:
    *   仅发现 `src/lib/services/ens-resolver.test.ts` 一个测试文件（包含 14 个测试用例）。
    *   核心交互逻辑（邮件列表渲染、点击事件、状态管理）和 UI 组件均无测试。

---

## 3. 测试增强建议

为了保证项目的长期稳定性和可维护性，建议按以下优先级补充测试：

### 优先级 P0 (高危/核心逻辑)

1.  **后端：核心命令单元测试**
    *   **位置**: `src-tauri/src/commands/*.rs`
    *   **内容**: 测试 `utils.rs` 中的辅助函数、`models.rs` 的序列化逻辑、以及不依赖数据库的纯函数逻辑。
    *   **理由**: 成本低，收益高，确保基础逻辑正确。

2.  **前端：核心业务逻辑测试**
    *   **位置**: `src/routes/handlers/*.ts`
    *   **内容**: 测试 `email-operations.ts` (邮件操作), `sync-idle.ts` (同步逻辑)。
    *   **理由**: 这些文件包含了前端最复杂的业务规则，最容易出 Bug。

### 优先级 P1 (集成与组件)

3.  **后端：数据库集成测试**
    *   **内容**: 使用内存数据库 (`:memory:`) 测试 `db.rs` 中的 CRUD 操作。
    *   **理由**: 确保 SQL 语句正确，且数据模型与数据库表结构匹配。

4.  **前端：关键组件测试**
    *   **内容**: 使用 `@testing-library/svelte` 测试 `EmailListSidebar.svelte` 和 `EmailBody.svelte`。
    *   **理由**: 确保 UI 能正确响应数据变化（如加载状态、空数据状态）。

### 优先级 P2 (端到端)

5.  **E2E 冒烟测试**
    *   **内容**: 使用 Playwright 或 Tauri WebDriver 编写一条核心路径测试（启动应用 -> 加载账户 -> 显示收件箱）。
    *   **理由**: 确保应用整体链路通畅，防止“白屏”级别的严重回归。
