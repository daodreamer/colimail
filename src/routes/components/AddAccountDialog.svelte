<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openUrl } from "@tauri-apps/plugin-shell";
  import { toast } from "svelte-sonner";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Tabs from "$lib/components/ui/tabs";
  import { Combobox } from "$lib/components/ui/combobox";
  import { EMAIL_PROVIDER_PRESETS, type EmailProviderPreset } from "../lib/email-providers";

  interface AddAccountDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onAccountAdded?: () => void;
  }

  let { open = $bindable(), onOpenChange, onAccountAdded }: AddAccountDialogProps = $props();

  let selectedProvider: "google" | "outlook" = $state("google");
  let oauthEmail = $state("");
  let oauthEmailInput = $state(""); // User input before @ symbol
  let isAuthenticating = $state(false);
  let emailInputError = $state("");

  // Validate and construct full email when input changes
  $effect(() => {
    if (oauthEmailInput.includes("@")) {
      emailInputError = "Email prefix should not contain @ symbol";
      oauthEmail = "";
    } else if (oauthEmailInput.trim()) {
      emailInputError = "";
      const domain = selectedProvider === "google" ? "@gmail.com" : "@outlook.com";
      oauthEmail = oauthEmailInput.trim() + domain;
    } else {
      emailInputError = "";
      oauthEmail = "";
    }
  });

  // Reset input when provider changes
  $effect(() => {
    selectedProvider; // track dependency
    oauthEmailInput = "";
    oauthEmail = "";
    emailInputError = "";
  });

  let accountConfig = $state({
    email: "",
    password: "",
    imap_server: "",
    imap_port: 993,
    smtp_server: "",
    smtp_port: 465,
  });

  let selectedPreset = $state("");
  let isTesting = $state(false);
  let testResult = $state<{
    imap_success: boolean;
    imap_error?: string;
    smtp_success: boolean;
    smtp_error?: string;
  } | null>(null);

  // Computed value for button disabled state
  const isTestPassed = $derived(
    testResult !== null && 
    testResult.imap_success === true && 
    testResult.smtp_success === true
  );

  // Debug effect to track isTestPassed changes
  $effect(() => {
    console.log("isTestPassed changed:", isTestPassed);
    console.log("testResult:", testResult);
  });

  // Apply preset when selected
  function applyPreset(presetValue: string) {
    console.log("Applying preset:", presetValue);
    const preset = EMAIL_PROVIDER_PRESETS.find(p => p.value === presetValue);
    if (preset && preset.value !== "custom") {
      accountConfig.imap_server = preset.imap_server;
      accountConfig.imap_port = preset.imap_port;
      accountConfig.smtp_server = preset.smtp_server;
      accountConfig.smtp_port = preset.smtp_port;
      console.log("Preset applied:", {
        imap: `${preset.imap_server}:${preset.imap_port}`,
        smtp: `${preset.smtp_server}:${preset.smtp_port}`
      });
    }
  }

  // Watch for preset changes - use separate effect to ensure it triggers
  $effect(() => {
    const preset = selectedPreset;
    if (preset) {
      console.log("Selected preset changed to:", preset);
      applyPreset(preset);
      // Clear test results when preset changes
      testResult = null;
    }
  });

  // Note: We don't auto-clear testResult when config changes because
  // it causes the button to disable right after successful test.
  // Users should re-test manually if they change settings.

  async function testConnection() {
    if (!accountConfig.email || !accountConfig.password || !accountConfig.imap_server || !accountConfig.smtp_server) {
      toast.error("Please fill in all required fields before testing");
      return;
    }

    isTesting = true;
    testResult = null;

    try {
      const result = await invoke<{
        imap_success: boolean;
        imap_error?: string;
        smtp_success: boolean;
        smtp_error?: string;
      }>("test_connection", {
        config: {
          ...accountConfig,
          auth_type: "basic"
        }
      });

      testResult = result;
      console.log("Test result:", testResult);

      if (result.imap_success && result.smtp_success) {
        toast.success("Connection test successful! Both IMAP and SMTP are working.");
      } else {
        toast.error("Connection test failed. Please check the details below.");
      }
    } catch (error) {
      console.error("Failed to test connection:", error);
      toast.error(`Connection test failed: ${error}`);
    } finally {
      isTesting = false;
    }
  }

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

      toast.success("Account configuration saved successfully!");
      
      // Reset form
      accountConfig = {
        email: "",
        password: "",
        imap_server: "",
        imap_port: 993,
        smtp_server: "",
        smtp_port: 465,
      };

      // Close dialog and notify parent
      onOpenChange(false);
      if (onAccountAdded) {
        onAccountAdded();
      }
    } catch (error) {
      console.error("Failed to save configuration:", error);
      toast.error("Failed to save account configuration");
    }
  }

  async function startOAuth2() {
    if (!oauthEmail) {
      toast.error("Please enter your email address");
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
      await openUrl(response.auth_url);

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

      toast.success(`${selectedProvider === 'google' ? 'Google' : 'Outlook'} account added successfully!`);
      oauthEmail = "";
      oauthEmailInput = "";
      emailInputError = "";
      
      // Close dialog and notify parent
      onOpenChange(false);
      if (onAccountAdded) {
        onAccountAdded();
      }
    } catch (error) {
      console.error("OAuth2 authentication failed:", error);
      toast.error(`OAuth2 authentication failed: ${error}`);
    } finally {
      isAuthenticating = false;
    }
  }
</script>

<Dialog.Root bind:open {onOpenChange}>
  <Dialog.Content class="max-w-[500px] p-0" trapFocus={false}>
    <Dialog.Title class="sr-only">Add an email account</Dialog.Title>
    <Dialog.Description class="sr-only">Connect your email using OAuth2 or manual configuration</Dialog.Description>
    
    <Card class="border-0 shadow-none">
      <CardHeader class="text-center">
        <CardTitle class="text-xl">Add an email account</CardTitle>
        <CardDescription>
          Connect your email using OAuth2 or manual configuration
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Tabs.Root value="oauth" class="w-full">
          <Tabs.List class="grid w-full grid-cols-2">
            <Tabs.Trigger value="oauth">OAuth2 (Recommended)</Tabs.Trigger>
            <Tabs.Trigger value="manual">Manual</Tabs.Trigger>
          </Tabs.List>
          
          <Tabs.Content value="oauth" class="space-y-4 mt-4">
            <div class="space-y-2">
              <Label>Email Provider</Label>
              <div class="grid grid-cols-2 gap-4">
                <Button
                  variant={selectedProvider === 'google' ? 'default' : 'outline'}
                  class="h-auto py-3"
                  onclick={() => (selectedProvider = "google")}
                >
                  <div class="flex flex-col items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-6 w-6">
                      <path fill="currentColor" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                      <path fill="currentColor" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                      <path fill="currentColor" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                      <path fill="currentColor" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                    </svg>
                    <span class="text-sm font-medium">Google</span>
                  </div>
                </Button>
                <Button
                  variant={selectedProvider === 'outlook' ? 'default' : 'outline'}
                  class="h-auto py-3"
                  onclick={() => (selectedProvider = "outlook")}
                >
                  <div class="flex flex-col items-center gap-2">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-6 w-6">
                      <path fill="currentColor" d="M24 7.875v8.25A3.375 3.375 0 0 1 20.625 19.5h-1.875v-9.75L12 13.5 5.25 9.75v9.75H3.375A3.375 3.375 0 0 1 0 16.125v-8.25A3.375 3.375 0 0 1 3.375 4.5h17.25A3.375 3.375 0 0 1 24 7.875z"/>
                    </svg>
                    <span class="text-sm font-medium">Outlook</span>
                  </div>
                </Button>
              </div>
            </div>
            
            <div class="space-y-2">
              <Label for="oauth-email">Email</Label>
              <div class="flex items-center gap-2">
                <Input
                  id="oauth-email"
                  type="text"
                  bind:value={oauthEmailInput}
                  placeholder={selectedProvider === "google" ? "username" : "username"}
                  disabled={isAuthenticating}
                  class={emailInputError ? "border-red-500" : ""}
                />
                <span class="text-sm text-muted-foreground whitespace-nowrap">
                  {selectedProvider === "google" ? "@gmail.com" : "@outlook.com"}
                </span>
              </div>
              {#if emailInputError}
                <p class="text-xs text-red-500">{emailInputError}</p>
              {:else}
                <p class="text-xs text-muted-foreground">
                  Enter only the part before {selectedProvider === "google" ? "@gmail.com" : "@outlook.com"}
                </p>
              {/if}
            </div>

            <Button
              class="w-full"
              onclick={startOAuth2}
              disabled={isAuthenticating || !oauthEmail}
            >
              {isAuthenticating ? "Authenticating..." : "Continue with " + (selectedProvider === 'google' ? 'Google' : 'Outlook')}
            </Button>

            {#if isAuthenticating}
              <div class="rounded-lg border border-blue-200 bg-blue-50 p-3 text-sm text-blue-900 dark:border-blue-800 dark:bg-blue-950/30 dark:text-blue-200">
                Please complete authentication in your browser, then close the browser window to return here...
              </div>
            {/if}
          </Tabs.Content>

          <Tabs.Content value="manual" class="space-y-4 mt-4">
            <form onsubmit={saveConfig} class="space-y-4">
              <!-- Provider Preset -->
              <div class="space-y-2">
                <Label for="preset">Email Provider Preset</Label>
                <Combobox
                  options={EMAIL_PROVIDER_PRESETS.map(p => ({ value: p.value, label: p.name }))}
                  bind:value={selectedPreset}
                  placeholder="Select a provider..."
                  searchPlaceholder="Search providers..."
                  emptyText="No provider found."
                />
                {#if selectedPreset && EMAIL_PROVIDER_PRESETS.find(p => p.value === selectedPreset)?.description}
                  <p class="text-xs text-muted-foreground">
                    {EMAIL_PROVIDER_PRESETS.find(p => p.value === selectedPreset)?.description}
                  </p>
                {/if}
              </div>

              <!-- Email/Password (Left) and IMAP/SMTP Settings (Right) in Two Columns -->
              <div class="grid grid-cols-2 gap-4">
                <!-- Left Column: Email and Password -->
                <div class="space-y-4">
                  <div class="space-y-2">
                    <Label for="email">Email</Label>
                    <Input 
                      id="email" 
                      type="email" 
                      bind:value={accountConfig.email} 
                      placeholder="m@example.com"
                      required 
                    />
                  </div>
                  
                  <div class="space-y-2">
                    <Label for="password">Password</Label>
                    <Input
                      id="password"
                      type="password"
                      bind:value={accountConfig.password}
                      placeholder="Use app-specific password if available"
                      required
                    />
                  </div>
                </div>

                <!-- Right Column: IMAP and SMTP Settings -->
                <div class="space-y-4">
                  <!-- IMAP Settings -->
                  <div class="space-y-2">
                    <Label>IMAP Settings</Label>
                    <div class="flex gap-2">
                      <Input 
                        id="imap-server" 
                        bind:value={accountConfig.imap_server} 
                        placeholder="imap.example.com"
                        class="flex-1"
                        required 
                      />
                      <Input
                        id="imap-port"
                        type="number"
                        bind:value={accountConfig.imap_port}
                        placeholder="993"
                        class="w-20"
                        required
                      />
                    </div>
                  </div>
                  
                  <!-- SMTP Settings -->
                  <div class="space-y-2">
                    <Label>SMTP Settings</Label>
                    <div class="flex gap-2">
                      <Input 
                        id="smtp-server" 
                        bind:value={accountConfig.smtp_server}
                        placeholder="smtp.example.com"
                        class="flex-1"
                        required 
                      />
                      <Input
                        id="smtp-port"
                        type="number"
                        bind:value={accountConfig.smtp_port}
                        placeholder="465"
                        class="w-20"
                        required
                      />
                    </div>
                  </div>
                </div>
              </div>

              <!-- Test Connection Button -->
              <Button
                type="button"
                variant="outline"
                class="w-full"
                onclick={testConnection}
                disabled={isTesting || !accountConfig.email || !accountConfig.password || !accountConfig.imap_server || !accountConfig.smtp_server}
              >
                {isTesting ? "Testing Connection..." : "Test Connection"}
              </Button>
              
              <!-- Test Results -->
              {#if testResult}
                <div class="space-y-2 rounded-lg border border-border p-3 text-sm bg-card">
                  <div class="flex items-center gap-2">
                    <span class="text-xl font-bold" style:color={testResult.imap_success ? '#16a34a' : '#dc2626'}>
                      {testResult.imap_success ? "✓" : "✗"}
                    </span>
                    <span class="font-medium">IMAP:</span>
                    <span class="font-semibold" style:color={testResult.imap_success ? '#16a34a' : '#dc2626'}>
                      {testResult.imap_success ? "Connected" : "Failed"}
                    </span>
                  </div>
                  {#if testResult.imap_error}
                    <p class="text-xs ml-8" style:color="#dc2626">{testResult.imap_error}</p>
                  {/if}
                  
                  <div class="flex items-center gap-2">
                    <span class="text-xl font-bold" style:color={testResult.smtp_success ? '#16a34a' : '#dc2626'}>
                      {testResult.smtp_success ? "✓" : "✗"}
                    </span>
                    <span class="font-medium">SMTP:</span>
                    <span class="font-semibold" style:color={testResult.smtp_success ? '#16a34a' : '#dc2626'}>
                      {testResult.smtp_success ? "Connected" : "Failed"}
                    </span>
                  </div>
                  {#if testResult.smtp_error}
                    <p class="text-xs ml-8" style:color="#dc2626">{testResult.smtp_error}</p>
                  {/if}
                </div>
              {/if}
              
              <!-- Create Account Button - Only enabled after successful test -->
              <Button 
                type="submit" 
                class="w-full"
                disabled={!isTestPassed}
              >
                Create Account
              </Button>

              {#if !testResult}
                <p class="text-center text-xs text-muted-foreground">
                  Please test connection before creating account
                </p>
              {:else if !isTestPassed}
                <p class="text-center text-xs font-medium" style:color="#dc2626">
                  ⚠ Connection test must pass before creating account
                </p>
              {:else}
                <p class="text-center text-xs font-medium" style:color="#16a34a">
                  ✓ Connection test passed! You can now create the account.
                </p>
              {/if}
            </form>
          </Tabs.Content>
        </Tabs.Root>
      </CardContent>
    </Card>
  </Dialog.Content>
</Dialog.Root>
