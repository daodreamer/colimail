<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";

  let authMode: "manual" | "oauth" = $state("manual");
  let selectedProvider: "google" | "outlook" = $state("google");
  let oauthEmail = $state("");
  let isAuthenticating = $state(false);

  let accountConfig = $state({
    email: "",
    password: "",
    imap_server: "",
    imap_port: 993,
    smtp_server: "",
    smtp_port: 465,
  });

  async function saveConfig(event: SubmitEvent) {
    event.preventDefault();
    try {
      await invoke("save_account_config", {
        config: {
          ...accountConfig,
          auth_type: "basic"
        }
      });

      // Load the saved account to get its ID
      const accounts = await invoke<any[]>("load_account_configs");
      const savedAccount = accounts.find(acc => acc.email === accountConfig.email);

      if (savedAccount) {
        // Sync folders for the new account
        try {
          await invoke("sync_folders", { config: savedAccount });
          console.log("Folders synced for new account");

          // Start IDLE monitoring for this account
          await invoke("start_idle_for_account", { config: savedAccount });
          console.log("IDLE monitoring started for new account");
        } catch (error) {
          console.error("Failed to sync folders or start IDLE:", error);
          // Don't show error to user as account is saved successfully
        }
      }

      alert("配置已保存！");
    } catch (error) {
      console.error("保存配置失败:", error);
      alert("保存配置失败！");
    }
  }

  async function startOAuth2() {
    if (!oauthEmail) {
      alert("请输入邮箱地址");
      return;
    }

    isAuthenticating = true;
    try {
      // Start listening for callback first
      const callbackPromise = invoke("listen_for_oauth_callback");

      // Get the authorization URL
      const response = await invoke<{ auth_url: string; state: string }>(
        "start_oauth2_flow",
        {
          request: {
            provider: selectedProvider,
            email: oauthEmail,
          },
        }
      );

      // Open browser for user authentication
      await open(response.auth_url);

      // Wait for callback
      const [code, state] = await callbackPromise as [string, string];

      // Complete OAuth2 flow
      await invoke("complete_oauth2_flow", {
        provider: selectedProvider,
        email: oauthEmail,
        code,
        state,
      });

      // Load the saved account to get its ID
      const accounts = await invoke<any[]>("load_account_configs");
      const savedAccount = accounts.find(acc => acc.email === oauthEmail);

      if (savedAccount) {
        // Sync folders for the new account
        try {
          await invoke("sync_folders", { config: savedAccount });
          console.log("Folders synced for new OAuth2 account");

          // Start IDLE monitoring for this account
          await invoke("start_idle_for_account", { config: savedAccount });
          console.log("IDLE monitoring started for new OAuth2 account");
        } catch (error) {
          console.error("Failed to sync folders or start IDLE:", error);
          // Don't show error to user as account is saved successfully
        }
      }

      alert(`✅ ${selectedProvider === 'google' ? 'Google' : 'Outlook'} 账户添加成功！`);
      oauthEmail = "";
    } catch (error) {
      console.error("OAuth2 认证失败:", error);
      alert(`OAuth2 认证失败: ${error}`);
    } finally {
      isAuthenticating = false;
    }
  }

  // 只是为了演示，调用 fetch_emails
  async function testFetch() {
    await invoke("fetch_emails", { config: accountConfig });
  }
</script>

<div class="container">
  <div class="header">
    <a href="/" class="back-button">← 返回</a>
    <h1>添加邮箱账户</h1>
  </div>

  <!-- Authentication Mode Selector -->
  <div class="mode-selector">
    <button
      class="mode-button {authMode === 'oauth' ? 'active' : ''}"
      onclick={() => (authMode = "oauth")}
      type="button"
    >
      OAuth2 认证 (推荐)
    </button>
    <button
      class="mode-button {authMode === 'manual' ? 'active' : ''}"
      onclick={() => (authMode = "manual")}
      type="button"
    >
      手动配置
    </button>
  </div>

  {#if authMode === "oauth"}
    <!-- OAuth2 Flow -->
    <div class="oauth-section">
      <h2>使用 OAuth2 添加账户</h2>
      <p class="info-text">
        选择您的邮箱服务提供商，我们将引导您完成安全认证流程。
      </p>

      <div class="form-group">
        <span class="label-text">选择邮箱服务商</span>
        <div class="provider-buttons">
          <button
            type="button"
            class="provider-button {selectedProvider === 'google' ? 'selected' : ''}"
            onclick={() => (selectedProvider = "google")}
          >
            <span class="provider-icon">G</span>
            Google
          </button>
          <button
            type="button"
            class="provider-button {selectedProvider === 'outlook' ? 'selected' : ''}"
            onclick={() => (selectedProvider = "outlook")}
          >
            <span class="provider-icon">M</span>
            Outlook
          </button>
        </div>
      </div>

      <div class="form-group">
        <label for="oauth-email">邮箱地址</label>
        <input
          id="oauth-email"
          type="email"
          bind:value={oauthEmail}
          placeholder="example@{selectedProvider === 'google' ? 'gmail.com' : 'outlook.com'}"
          required
          disabled={isAuthenticating}
        />
      </div>

      <button
        class="primary-button"
        onclick={startOAuth2}
        disabled={isAuthenticating}
        type="button"
      >
        {isAuthenticating ? "认证中..." : "开始认证"}
      </button>

      {#if isAuthenticating}
        <p class="auth-hint">
          请在浏览器中完成认证，然后关闭浏览器窗口返回此处...
        </p>
      {/if}
    </div>
  {:else}
    <!-- Manual Configuration Form -->
    <div class="manual-section">
      <h2>手动配置邮箱</h2>
      <p class="info-text">
        适用于自建邮箱或不支持 OAuth2 的邮箱服务。
      </p>

      <form onsubmit={saveConfig}>
        <div class="form-group">
          <label for="email">邮箱地址</label>
          <input id="email" type="email" bind:value={accountConfig.email} required />
        </div>
        <div class="form-group">
          <label for="password">密码</label>
          <input
            id="password"
            type="password"
            bind:value={accountConfig.password}
            required
          />
        </div>
        <div class="form-group">
          <label for="imap-server">IMAP 服务器</label>
          <input id="imap-server" bind:value={accountConfig.imap_server} required />
        </div>
        <div class="form-group">
          <label for="imap-port">IMAP 端口</label>
          <input
            id="imap-port"
            type="number"
            bind:value={accountConfig.imap_port}
            required
          />
        </div>
        <div class="form-group">
          <label for="smtp-server">SMTP 服务器</label>
          <input id="smtp-server" bind:value={accountConfig.smtp_server} required />
        </div>
        <div class="form-group">
          <label for="smtp-port">SMTP 端口</label>
          <input
            id="smtp-port"
            type="number"
            bind:value={accountConfig.smtp_port}
            required
          />
        </div>
        <button type="submit" class="primary-button">保存配置</button>
      </form>

      <button onclick={testFetch} class="secondary-button" type="button">
        测试收取邮件
      </button>
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    overflow: auto !important;
    position: static !important;
    height: auto !important;
  }

  .container {
    padding: 2rem;
    max-width: 700px;
    margin: 0 auto;
    min-height: 100vh;
    overflow-y: auto;
    box-sizing: border-box;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .back-button {
    padding: 0.5rem 1rem;
    background-color: #6c757d;
    color: white;
    text-decoration: none;
    border-radius: 6px;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .back-button:hover {
    background-color: #5a6268;
  }

  h1 {
    margin: 0;
    color: #333;
  }

  h2 {
    font-size: 1.3rem;
    margin-bottom: 0.5rem;
    color: #444;
  }

  .info-text {
    color: #666;
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
  }

  /* Mode Selector */
  .mode-selector {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
    border-bottom: 2px solid #e0e0e0;
  }

  .mode-button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border: none;
    background: transparent;
    color: #666;
    cursor: pointer;
    border-bottom: 3px solid transparent;
    transition: all 0.2s;
    margin-bottom: -2px;
  }

  .mode-button.active {
    color: #007bff;
    border-bottom-color: #007bff;
  }

  .mode-button:hover:not(.active) {
    color: #333;
  }

  /* OAuth Section */
  .oauth-section,
  .manual-section {
    animation: fadeIn 0.3s;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* Provider Buttons */
  .provider-buttons {
    display: flex;
    gap: 1rem;
    margin-top: 0.5rem;
  }

  .provider-button {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1rem;
    font-size: 1rem;
    border: 2px solid #ddd;
    border-radius: 8px;
    background: white;
    color: #333;
    cursor: pointer;
    transition: all 0.2s;
  }

  .provider-button:hover {
    border-color: #007bff;
    background: #f8f9fa;
  }

  .provider-button.selected {
    border-color: #007bff;
    background: #e7f3ff;
    color: #007bff;
    font-weight: 600;
  }

  .provider-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #007bff;
    color: white;
    font-weight: bold;
    font-size: 0.9rem;
  }

  .provider-button.selected .provider-icon {
    background: #0056b3;
  }

  /* Form Elements */
  .form-group {
    margin-bottom: 1.5rem;
  }

  label,
  .label-text {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: #333;
  }

  input {
    width: 100%;
    padding: 0.75rem;
    font-size: 1rem;
    border-radius: 6px;
    border: 1px solid #ccc;
    box-sizing: border-box;
    transition: border-color 0.2s;
  }

  input:focus {
    outline: none;
    border-color: #007bff;
  }

  input:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  /* Buttons */
  .primary-button,
  .secondary-button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border-radius: 6px;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
  }

  .primary-button {
    background-color: #007bff;
    color: white;
    width: 100%;
  }

  .primary-button:hover:not(:disabled) {
    background-color: #0056b3;
  }

  .primary-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .secondary-button {
    background-color: #6c757d;
    color: white;
    margin-top: 1rem;
  }

  .secondary-button:hover {
    background-color: #5a6268;
  }

  .auth-hint {
    margin-top: 1rem;
    padding: 1rem;
    background: #fff3cd;
    border: 1px solid #ffc107;
    border-radius: 6px;
    color: #856404;
    text-align: center;
  }
</style>
