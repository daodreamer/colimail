# 快速发布指南

## 🚀 立即发布测试版的步骤

### 准备工作(5分钟)

1. **检查文档配置**
   - 所有仓库链接已配置为 `daodreamer/colimail`
   - 安全联系邮箱已配置

2. **更新 SECURITY.md 中的联系方式(如需修改)**
   - 替换 `[your-email@example.com]` 为真实的安全联系邮箱
   - 文件位置: `SECURITY.md` 第10行, 第174行

3. **确保所有文件已提交到 Git**
   ```bash
   git add .
   git commit -m "docs: prepare for first public release"
   git push origin dev
   ```

### 方式一: GitHub Actions 自动构建(推荐)

#### 第1步: 推送到 GitHub

```bash
# 如果还没有推送到 GitHub,创建仓库并推送
git remote add origin https://github.com/daodreamer/colimail.git
git branch -M main
git push -u origin main
```

#### 第2步: 创建发布标签

```bash
# 创建 v0.1.0 标签
git tag -a v0.1.0 -m "First public beta release"
git push origin v0.1.0
```

#### 第3步: 等待自动构建(约15-30分钟)

1. 访问: `https://github.com/daodreamer/colimail/actions`
2. 查看 "Release Build" 工作流运行状态
3. 等待所有平台构建完成(约10-15分钟):
   - ✅ Windows (x86_64)
   - ✅ macOS Apple Silicon (aarch64)

#### 第4步: 发布 Release

1. 访问: `https://github.com/daodreamer/colimail/releases`
2. 找到自动创建的草稿 Release
3. 编辑发布说明(可选,已有默认内容)
4. 点击 "Publish release"

**完成!** 用户现在可以从 Releases 页面下载安装包了。

---

### 方式二: 手动本地构建

如果不使用 GitHub Actions,可以手动构建:

#### Windows 构建

```bash
npm install
npm run tauri build
# 输出: src-tauri/target/release/bundle/msi/Colimail_0.1.0_x64_en-US.msi
```

#### macOS Apple Silicon 构建(需要在 M1/M2/M3 Mac 上)

```bash
npm install
npm run tauri build --target aarch64-apple-darwin
# 输出: src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Colimail_0.1.0_aarch64.dmg
```

然后手动上传到 GitHub Releases。

---

## 📢 发布后的推广

### 1. 更新项目主页

在 GitHub 仓库页面添加:
- 项目描述
- 主题标签: `email-client`, `tauri`, `svelte`, `rust`, `desktop-app`
- 网站链接(如有)

### 2. 社区推广

分享到:
- **Reddit**: r/rust, r/sveltejs, r/opensource
- **Twitter/X**: 使用标签 #Rust #Tauri #EmailClient
- **Hacker News**: news.ycombinator.com
- **V2EX**: v2ex.com (国内社区)
- **掘金**: juejin.cn (中文技术社区)

### 3. 撰写推广文案(示例)

```
🎉 Colimail v0.1.0 公开测试版发布!

一款轻量级、高性能的跨平台邮箱客户端:
✅ 启动仅需1.5秒,内存占用<80MB
✅ 支持 Gmail/Outlook OAuth2 认证
✅ 使用 Rust + Tauri 2 + Svelte 5 构建
✅ 完全开源,本地存储,无数据收集

下载试用: https://github.com/daodreamer/colimail/releases
欢迎反馈和贡献! ⭐

#EmailClient #Rust #Tauri #OpenSource
```

### 4. 收集反馈

- 在 GitHub Issues 中回复用户问题
- 创建 Discussions 用于功能讨论
- 建立用户社群(Discord/Telegram,可选)

---

## 📊 监控发布效果

### GitHub 统计

查看:
- Stars/Forks 增长
- Issues 数量
- 下载次数(Release 页面)
- Traffic 统计(Insights → Traffic)

### 用户反馈渠道

- GitHub Issues: 用于 bug 报告
- GitHub Discussions: 用于功能讨论
- Email: 用于安全问题和私密反馈

---

## 🔧 常见问题

### Q: macOS 提示"无法打开,因为来自未知开发者"

**A**: 用户需要右键点击 → "打开",或在系统设置中允许。

**解决方案**(未来):
- 申请 Apple Developer 账号($99/年)
- 对应用进行代码签名和公证(notarization)

### Q: Intel Mac 用户能用吗?

**A**: 当前版本仅支持 Apple Silicon (M1/M2/M3)。如果有足够需求,可以在未来版本添加 Intel 支持,或者用户可以在本地自行构建 Intel 版本。

### Q: Windows SmartScreen 警告

**A**: 新应用会触发 SmartScreen,用户点击"仍要运行"即可。

**解决方案**(未来):
- 购买代码签名证书($200-500/年)
- 积累安装量后 SmartScreen 会自动信任

### Q: 构建失败

**A**: 检查:
1. Rust 和 Node.js 版本是否符合要求
2. 是否安装了平台特定依赖(Linux)
3. 查看 GitHub Actions 日志定位问题

---

## ✅ 发布前最后检查

- [ ] README 中的用户名已更新
- [ ] SECURITY.md 中的邮箱已更新
- [ ] 所有代码已提交并推送
- [ ] 版本号一致(package.json, Cargo.toml, tauri.conf.json)
- [ ] 本地测试构建成功
- [ ] GitHub Actions 已启用
- [ ] 准备好发布推广文案

**准备就绪?** 执行 `git tag -a v0.1.0 -m "First release" && git push origin v0.1.0`

---

## 📞 需要帮助?

- 查看 [RELEASE_CHECKLIST.md](./RELEASE_CHECKLIST.md) 获取详细步骤
- 阅读 [Tauri 发布文档](https://v2.tauri.app/distribute/)
- 在 GitHub Discussions 寻求社区帮助

祝发布顺利! 🚀
