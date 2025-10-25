<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as ButtonGroup from "$lib/components/ui/button-group";

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


<div class="min-h-screen bg-background p-8">
  <div class="mx-auto max-w-3xl space-y-6">
    <!-- Header -->
    <div class="flex items-center gap-4">
      <Button variant="outline" href="/">
        ← 返回
      </Button>
      <h1 class="text-3xl font-bold">添加邮箱账户</h1>
    </div>

    <!-- Authentication Mode Selector -->
    <ButtonGroup.Root class="w-full border-b">
      <Button
        variant={authMode === 'oauth' ? 'default' : 'ghost'}
        class="flex-1 rounded-b-none"
        onclick={() => (authMode = "oauth")}
      >
        OAuth2 认证 (推荐)
      </Button>
      <Button
        variant={authMode === 'manual' ? 'default' : 'ghost'}
        class="flex-1 rounded-b-none"
        onclick={() => (authMode = "manual")}
      >
        手动配置
      </Button>
    </ButtonGroup.Root>

    {#if authMode === "oauth"}
      <!-- OAuth2 Flow -->
      <Card>
        <CardHeader>
          <CardTitle>使用 OAuth2 添加账户</CardTitle>
          <CardDescription>
            选择您的邮箱服务提供商，我们将引导您完成安全认证流程。
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-6">
          <!-- Provider Selection -->
          <div class="space-y-2">
            <Label>选择邮箱服务商</Label>
            <ButtonGroup.Root class="w-full">
              <Button
                variant={selectedProvider === 'google' ? 'default' : 'outline'}
                class="h-14 flex-1 text-base"
                onclick={() => (selectedProvider = "google")}
              >
                <span class="mr-2 flex h-7 w-7 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-semibold">
                  G
                </span>
                Google
              </Button>
              <Button
                variant={selectedProvider === 'outlook' ? 'default' : 'outline'}
                class="h-14 flex-1 text-base"
                onclick={() => (selectedProvider = "outlook")}
              >
                <span class="mr-2 flex h-7 w-7 items-center justify-center rounded-full bg-primary text-primary-foreground text-sm font-semibold">
                  M
                </span>
                Outlook
              </Button>
            </ButtonGroup.Root>
          </div>

          <!-- Email Input -->
          <div class="space-y-2">
            <Label for="oauth-email">邮箱地址</Label>
            <Input
              id="oauth-email"
              type="email"
              bind:value={oauthEmail}
              placeholder="example@{selectedProvider === 'google' ? 'gmail.com' : 'outlook.com'}"
              disabled={isAuthenticating}
            />
          </div>

          <!-- Start Authentication Button -->
          <Button
            class="w-full"
            onclick={startOAuth2}
            disabled={isAuthenticating}
          >
            {isAuthenticating ? "认证中..." : "开始认证"}
          </Button>

          {#if isAuthenticating}
            <div class="rounded-lg border border-yellow-300 bg-yellow-50 p-4 text-center text-sm text-yellow-800">
              请在浏览器中完成认证，然后关闭浏览器窗口返回此处...
            </div>
          {/if}
        </CardContent>
      </Card>
    {:else}
      <!-- Manual Configuration Form -->
      <Card>
        <CardHeader>
          <CardTitle>手动配置邮箱</CardTitle>
          <CardDescription>
            适用于自建邮箱或不支持 OAuth2 的邮箱服务。
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onsubmit={saveConfig} class="space-y-4">
            <div class="space-y-2">
              <Label for="email">邮箱地址</Label>
              <Input id="email" type="email" bind:value={accountConfig.email} required />
            </div>
            <div class="space-y-2">
              <Label for="password">密码</Label>
              <Input
                id="password"
                type="password"
                bind:value={accountConfig.password}
                required
              />
            </div>
            <div class="space-y-2">
              <Label for="imap-server">IMAP 服务器</Label>
              <Input id="imap-server" bind:value={accountConfig.imap_server} required />
            </div>
            <div class="space-y-2">
              <Label for="imap-port">IMAP 端口</Label>
              <Input
                id="imap-port"
                type="number"
                bind:value={accountConfig.imap_port}
                required
              />
            </div>
            <div class="space-y-2">
              <Label for="smtp-server">SMTP 服务器</Label>
              <Input id="smtp-server" bind:value={accountConfig.smtp_server} required />
            </div>
            <div class="space-y-2">
              <Label for="smtp-port">SMTP 端口</Label>
              <Input
                id="smtp-port"
                type="number"
                bind:value={accountConfig.smtp_port}
                required
              />
            </div>
            <Button type="submit" class="w-full">保存配置</Button>
          </form>

          <Button onclick={testFetch} variant="secondary" class="mt-4 w-full">
            测试收取邮件
          </Button>
        </CardContent>
      </Card>
    {/if}
  </div>
</div>
