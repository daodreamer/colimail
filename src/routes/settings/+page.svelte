'''<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let accountConfig = {
    email: "",
    password: "",
    imap_server: "",
    imap_port: 993,
    smtp_server: "",
    smtp_port: 465,
  };

  async function saveConfig() {
    try {
      await invoke("save_account_config", { config: accountConfig });
      alert("配置已保存！");
    } catch (error) {
      console.error("保存配置失败:", error);
      alert("保存配置失败！");
    }
  }

  // 只是为了演示，调用 fetch_emails
  async function testFetch() {
      await invoke("fetch_emails", { config: accountConfig });
  }

</script>

<div class="container">
  <h1>账户配置</h1>
  <form on:submit|preventDefault={saveConfig}>
    <div class="form-group">
      <label for="email">邮箱地址</label>
      <input id="email" type="email" bind:value={accountConfig.email} required>
    </div>
    <div class="form-group">
      <label for="password">密码</label>
      <input id="password" type="password" bind:value={accountConfig.password} required>
    </div>
    <div class="form-group">
      <label for="imap-server">IMAP 服务器</label>
      <input id="imap-server" bind:value={accountConfig.imap_server} required>
    </div>
    <div class="form-group">
      <label for="imap-port">IMAP 端口</label>
      <input id="imap-port" type="number" bind:value={accountConfig.imap_port} required>
    </div>
    <div class="form-group">
      <label for="smtp-server">SMTP 服务器</label>
      <input id="smtp-server" bind:value={accountConfig.smtp_server} required>
    </div>
    <div class="form-group">
      <label for="smtp-port">SMTP 端口</label>
      <input id="smtp-port" type="number" bind:value={accountConfig.smtp_port} required>
    </div>
    <button type="submit">保存配置</button>
  </form>

  <button on:click={testFetch} style="margin-top: 20px;">测试收取邮件</button>
</div>

<style>
  .container {
    padding: 2rem;
    max-width: 600px;
    margin: 0 auto;
  }
  .form-group {
    margin-bottom: 1rem;
  }
  label {
    display: block;
    margin-bottom: 0.5rem;
  }
  input {
    width: 100%;
    padding: 0.5rem;
    font-size: 1rem;
    border-radius: 4px;
    border: 1px solid #ccc;
  }
  button {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border-radius: 4px;
    border: none;
    background-color: #007bff;
    color: white;
    cursor: pointer;
  }
  button:hover {
    background-color: #0056b3;
  }
</style>
''