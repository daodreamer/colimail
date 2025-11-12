# CMVH客户端集成状态

**日期**: 2025-11-11
**状态**: 🔄 **进行中** - 后端完成，前端待完成

---

## ✅ 已完成的工作

### 1. Rust后端 - CMVH签名功能

**文件**: `src-tauri/src/cmvh/signer.rs`

**功能**:
- ✅ `sign_email()` - 使用私钥签名邮件
- ✅ `derive_address()` - 从私钥推导以太坊地址
- ✅ `canonicalize_email()` - 标准化邮件内容格式
- ✅ `hash_email()` - 计算keccak256哈希
- ✅ EIP-191签名支持

**测试**: 所有单元测试通过

###  2. Tauri命令

**文件**: `src-tauri/src/commands/cmvh.rs`

**可用命令**:
```rust
sign_email_with_cmvh(private_key, content) -> CMVHHeaders
derive_eth_address(private_key) -> String
parse_email_cmvh_headers(raw_headers) -> CMVHHeaders
verify_cmvh_signature(headers, content) -> VerificationResult
has_cmvh_headers(raw_headers) -> bool
hash_email_content(content) -> String
```

### 3. 邮件发送命令更新

**文件**: `src-tauri/src/commands/send.rs`

**状态**: ✅ 编译成功

**注意**: 由于lettre库对自定义头部支持有限，暂时无法在发送的邮件中添加CMVH头部。

**临时解决方案**:
- 用户可以使用 `scripts/send-cmvh-email.mjs` 发送CMVH签名的邮件
- 客户端可以完整验证接收到的CMVH邮件
- 未来可以升级到支持自定义头部的SMTP库

---

## 🔄 待完成的工作

### 1. SettingsDialog - CMVH配置

**需要添加**:
- CMVH私钥输入框（带隐藏/显示切换）
- "从私钥生成地址"按钮
- 显示派生的以太坊地址
- "启用CMVH签名"开关
- 私钥安全存储（使用加密）

**位置**: `src/routes/components/SettingsDialog.svelte`

**示例配置UI**:
```svelte
{:else if currentPage === "CMVH Verification"}
  <div class="space-y-6">
    <!-- 启用CMVH -->
    <div class="flex items-center justify-between">
      <Label>Enable CMVH Email Signing</Label>
      <input type="checkbox" bind:checked={cmvhConfig.enabled} />
    </div>

    <!-- 私钥输入 -->
    <div class="space-y-2">
      <Label>Private Key</Label>
      <Input type={showPrivateKey ? "text" : "password"}
             bind:value={privateKey}
             placeholder="0x..." />
      <Button size="sm" onclick={toggleShowPrivateKey}>
        {showPrivateKey ? "Hide" : "Show"}
      </Button>
    </div>

    <!-- 派生地址 -->
    <div class="space-y-2">
      <Label>Ethereum Address</Label>
      <Input value={derivedAddress} readonly />
      <Button onclick={deriveAddressFromKey}>Derive Address</Button>
    </div>

    <!-- 现有的验证设置 -->
    <div class="flex items-center justify-between">
      <Label>Auto-verify on Email Open</Label>
      <input type="checkbox" bind:checked={cmvhConfig.autoVerify} />
    </div>

    <Button onclick={saveCMVHSettings}>Save Settings</Button>
  </div>
{/if}
```

### 2. 前端签名工具

**需要创建**: `src/lib/cmvh/signing.ts`

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { CMVHHeaders, EmailContent } from "./types";

export async function signEmailContent(
  privateKey: string,
  content: EmailContent
): Promise<CMVHHeaders> {
  return await invoke("sign_email_with_cmvh", {
    privateKey,
    content,
  });
}

export async function deriveAddress(privateKey: string): Promise<string> {
  return await invoke("derive_eth_address", { privateKey });
}
```

### 3. EmailList组件 - 验证徽章

**位置**: `src/routes/components/EmailList.svelte`

**需要添加**:
```svelte
<script>
  import VerificationBadge from "$lib/components/cmvh/verification-badge.svelte";
  import { verifyEmail } from "$lib/cmvh";

  let verificationStates = $state(new Map());

  async function loadVerificationStatus(email: EmailHeader) {
    if (email.raw_headers) {
      const state = await verifyEmail(email.raw_headers, {
        subject: email.subject,
        from: email.from,
        to: email.to,
        body: email.body || "",
      });
      verificationStates.set(email.uid, state);
    }
  }
</script>

<!-- 在邮件列表项中添加 -->
{#if verificationStates.has(email.uid)}
  <VerificationBadge
    verification={verificationStates.get(email.uid)}
    onclick={() => showVerificationPanel(email.uid)}
  />
{/if}
```

### 4. EmailBody组件 - 验证面板

**位置**: `src/routes/components/EmailBody.svelte`

**需要添加**:
```svelte
<script>
  import VerificationPanel from "$lib/components/cmvh/verification-panel.svelte";
  import { verifyEmail, verifyOnChain } from "$lib/cmvh";

  let verificationState = $state(null);
  let showVerificationPanel = $state(false);

  async function verifyCurrentEmail() {
    if (email && body) {
      verificationState = await verifyEmail(email.raw_headers, {
        subject: email.subject,
        from: email.from,
        to: email.to,
        body: body,
      });
      showVerificationPanel = true;
    }
  }

  async function verifyOnChainHandler() {
    // 调用链上验证
    const result = await verifyOnChain(verificationState.result.headers, {
      subject: email.subject,
      from: email.from,
      to: email.to,
      body: body,
    });
    // 更新状态
  }
</script>

<!-- 在邮件头部信息后添加 -->
{#if verificationState}
  <VerificationPanel
    {verificationState}
    onVerifyOnChain={verifyOnChainHandler}
    onClose={() => showVerificationPanel = false}
  />
{/if}
```

### 5. Toast通知

**需要添加**: 使用现有的toast系统

```typescript
import { toast } from "svelte-sonner";

// 签名成功
toast.success("Email signed with CMVH successfully", {
  description: `Signer: ${address.slice(0, 10)}...`,
});

// 验证成功
toast.success("CMVH signature verified", {
  description: "Email is cryptographically authentic",
});

// 验证失败
toast.error("CMVH verification failed", {
  description: "Signature does not match email content",
});

// 链上验证
toast.info("Verifying on-chain...", {
  description: "Calling Arbitrum smart contract",
});
```

---

## 📋 实现步骤建议

1. **SettingsDialog配置** (30分钟)
   - 添加CMVH配置UI
   - 实现私钥输入和地址派生
   - 添加保存/加载配置逻辑

2. **前端签名工具** (15分钟)
   - 创建 `src/lib/cmvh/signing.ts`
   - 封装Tauri命令调用

3. **EmailList集成** (30分钟)
   - 添加验证徽章显示
   - 实现自动验证逻辑
   - 添加验证状态缓存

4. **EmailBody集成** (30分钟)
   - 添加验证面板
   - 实现链上验证按钮
   - 添加验证详情展示

5. **Toast通知** (15分钟)
   - 添加各类操作反馈
   - 成功/失败/进行中状态

6. **测试** (30分钟)
   - 使用 `scripts/send-cmvh-email.mjs` 发送测试邮件
   - 在客户端接收和验证
   - 测试链上验证功能

---

## 🔧 已知限制

1. **无法通过客户端发送CMVH邮件**
   - 原因: lettre库不支持自定义头部
   - 临时方案: 使用 `scripts/send-cmvh-email.mjs`
   - 未来: 升级SMTP库或使用Rust email库

2. **私钥存储安全性**
   - 当前: 需要实现加密存储
   - 建议: 使用tauri的安全存储API
   - 或: 集成硬件钱包支持

3. **性能优化**
   - 邮件列表的批量验证可能较慢
   - 建议: 实现后台验证 + 缓存
   - 或: 仅验证可见邮件

---

## 📚 相关文档

- **CMVH标准**: `docs/CMVH_DEV.md`
- **Phase 3计划**: `docs/PHASE3_PLAN.md`
- **测试指南**: `docs/CMVH_TESTING.md`
- **测试脚本**: `scripts/send-cmvh-email.mjs`

---

## 🎯 下一步行动

**优先级1**: 完成前端集成
- [x] 后端签名功能
- [ ] SettingsDialog配置UI
- [ ] EmailList徽章显示
- [ ] EmailBody验证面板
- [ ] Toast通知

**优先级2**: 测试和优化
- [ ] 端到端测试
- [ ] 性能优化
- [ ] 用户体验改进

**优先级3**: 增强功能
- [ ] 私钥加密存储
- [ ] 批量验证优化
- [ ] 发送CMVH邮件支持（需要SMTP库升级）

---

**更新日期**: 2025-11-11
**实现进度**: 40% (后端完成，前端待完成)
