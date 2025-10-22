# OAuth2 认证配置指南

本文档说明如何配置 Google 和 Outlook 的 OAuth2 认证,以便用户可以通过更安全的 OAuth2 流程添加邮箱账户。

## 功能概述

Colimail 现在支持两种邮箱账户添加方式:

1. **OAuth2 认证** (推荐) - 适用于 Google 和 Outlook 等支持 OAuth2 的邮箱服务
2. **手动配置** - 适用于自建邮箱或不支持 OAuth2 的邮箱服务

## OAuth2 实现架构

### 核心组件

1. **后端 (Rust)**
   - `src-tauri/src/oauth2_config.rs` - OAuth2 提供商配置
   - `src-tauri/src/commands/oauth2.rs` - OAuth2 认证命令
   - `src-tauri/src/models.rs` - 数据模型 (支持 OAuth2 令牌)

2. **前端 (SvelteKit)**
   - `src/routes/settings/+page.svelte` - 账户设置页面,支持 OAuth2 和手动配置切换

3. **数据库**
   - accounts 表扩展了以下字段:
     - `auth_type` - 认证类型 (basic/oauth2)
     - `access_token` - OAuth2 访问令牌
     - `refresh_token` - OAuth2 刷新令牌
     - `token_expires_at` - 令牌过期时间

### OAuth2 流程

1. 用户在前端选择邮箱提供商 (Google/Outlook)
2. 后端生成授权 URL (使用 PKCE 增强安全性)
3. 应用打开系统浏览器,用户进行认证
4. 本地 HTTP 服务器 (localhost:8765) 接收回调
5. 后端交换授权码获取访问令牌
6. 令牌存储到数据库,账户添加完成

## Google OAuth2 配置

### 1. 创建 Google Cloud Project

1. 访问 [Google Cloud Console](https://console.cloud.google.com/)
2. 创建新项目或选择现有项目
3. 在左侧菜单选择 "APIs & Services" > "Credentials"

### 2. 配置 OAuth 同意屏幕

1. 点击 "OAuth consent screen"
2. 选择用户类型 (External 用于个人使用)
3. 填写应用信息:
   - App name: `MailDesk`
   - User support email: 您的邮箱
   - Developer contact information: 您的邮箱
4. 添加 Scopes:
   - `https://mail.google.com/` (完整的邮件访问)
   - `https://www.googleapis.com/auth/userinfo.email` (基本信息)
5. 添加测试用户 (如果是测试模式)

### 3. 创建 OAuth 2.0 Client ID

1. 在 "Credentials" 页面,点击 "Create Credentials" > "OAuth client ID"
2. Application type: **Desktop app**
3. Name: `MailDesk Desktop Client`
4. 点击 "Create"
5. 记录 **Client ID** 和 **Client Secret**

### 4. 设置环境变量

在系统中设置以下环境变量:

```bash
# Windows (PowerShell)
$env:GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
$env:GOOGLE_CLIENT_SECRET="your-client-secret"

# Windows (命令提示符)
set GOOGLE_CLIENT_ID=your-client-id.apps.googleusercontent.com
set GOOGLE_CLIENT_SECRET=your-client-secret

# Linux/Mac
export GOOGLE_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_CLIENT_SECRET="your-client-secret"
```

或者直接修改 `src-tauri/src/oauth2_config.rs` 文件中的 `google()` 方法,替换默认值。

## Outlook (Microsoft 365) OAuth2 配置

### 1. 注册应用到 Azure AD

1. 访问 [Azure Portal](https://portal.azure.com/)
2. 进入 "Azure Active Directory" > "App registrations"
3. 点击 "New registration"

### 2. 配置应用

1. 填写应用信息:
   - Name: `MailDesk`
   - Supported account types: "Accounts in any organizational directory and personal Microsoft accounts"
   - Redirect URI: 选择 "Public client/native (mobile & desktop)",填写 `http://localhost:8765/callback`
2. 点击 "Register"

### 3. 配置 API 权限

1. 在左侧菜单选择 "API permissions"
2. 点击 "Add a permission" > "Microsoft Graph"
3. 选择 "Delegated permissions",添加:
   - `IMAP.AccessAsUser.All`
   - `SMTP.Send`
   - `offline_access` (用于刷新令牌)
4. 点击 "Grant admin consent" (如果需要)

### 4. 创建 Client Secret

1. 在左侧菜单选择 "Certificates & secrets"
2. 点击 "New client secret"
3. 添加描述,选择过期时间
4. 记录 **Value** (这是您的 client secret,只显示一次!)

### 5. 获取 Client ID

1. 在 "Overview" 页面找到 "Application (client) ID"
2. 这就是您的 Client ID

### 6. 设置环境变量

```bash
# Windows (PowerShell)
$env:OUTLOOK_CLIENT_ID="your-application-id"
$env:OUTLOOK_CLIENT_SECRET="your-client-secret-value"

# Windows (命令提示符)
set OUTLOOK_CLIENT_ID=your-application-id
set OUTLOOK_CLIENT_SECRET=your-client-secret-value

# Linux/Mac
export OUTLOOK_CLIENT_ID="your-application-id"
export OUTLOOK_CLIENT_SECRET="your-client-secret-value"
```

## 使用方法

1. 启动应用: `npm run tauri dev`
2. 进入设置页面
3. 选择 "OAuth2 认证 (推荐)"
4. 选择邮箱服务商 (Google 或 Outlook)
5. 输入邮箱地址
6. 点击 "开始认证"
7. 在浏览器中完成登录和授权
8. 关闭浏览器窗口,返回应用
9. 账户添加成功!

## 安全性说明

- 使用 PKCE (Proof Key for Code Exchange) 增强安全性
- 访问令牌和刷新令牌加密存储在本地数据库
- 回调服务器仅在认证过程中短暂运行
- CSRF 令牌验证防止跨站请求伪造攻击

## 注意事项

1. **端口占用**: OAuth2 回调服务器使用 8765 端口,请确保该端口未被占用
2. **防火墙**: 如果遇到回调问题,请检查防火墙设置
3. **令牌刷新**: 当前实现不包含自动令牌刷新,需要在未来版本中添加
4. **密钥安全**: 不要将 Client Secret 提交到版本控制系统,建议使用环境变量

## 开发者指南

### 添加新的 OAuth2 提供商

1. 在 `oauth2_config.rs` 中添加新的提供商方法:
```rust
pub fn provider_name() -> Self {
    Self {
        client_id: std::env::var("PROVIDER_CLIENT_ID").unwrap_or_else(|_| "default".to_string()),
        client_secret: std::env::var("PROVIDER_CLIENT_SECRET").unwrap_or_else(|_| "default".to_string()),
        auth_url: "https://provider.com/oauth2/authorize".to_string(),
        token_url: "https://provider.com/oauth2/token".to_string(),
        scopes: vec!["mail.read".to_string(), "mail.send".to_string()],
        imap_server: "imap.provider.com".to_string(),
        imap_port: 993,
        smtp_server: "smtp.provider.com".to_string(),
        smtp_port: 587,
    }
}
```

2. 在 `get_provider()` 方法中添加匹配:
```rust
"provider_name" => Ok(Self::provider_name()),
```

3. 在前端添加新的提供商按钮

## 故障排查

### 问题: "Invalid or expired OAuth state"
**解决**: 认证流程超时或被中断,请重新开始认证

### 问题: "Failed to bind to port 8765"
**解决**: 端口被占用,关闭占用该端口的程序或更改回调 URL 中的端口

### 问题: 浏览器打不开
**解决**: 检查是否安装了默认浏览器,或手动复制 URL 到浏览器

### 问题: "Failed to exchange authorization code"
**解决**:
- 检查 Client ID 和 Client Secret 是否正确
- 确认 redirect URI 配置为 `http://localhost:8765/callback`
- 查看控制台错误日志获取详细信息

## 未来改进

- [ ] 自动令牌刷新机制
- [ ] 令牌加密存储
- [ ] 支持更多邮箱服务商 (Yahoo, ProtonMail 等)
- [ ] OAuth2 令牌撤销功能
- [ ] 更友好的错误提示
