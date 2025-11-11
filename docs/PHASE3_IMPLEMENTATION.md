# Phase 3 Implementation Guide

**Date**: 2025-11-10
**Status**: 🚧 **IN PROGRESS**
**Target**: ColiMail Client Integration

## Project Structure

Based on the ColiMail repository (https://github.com/daodreamer/colimail), we'll integrate CMVH into the existing Tauri + Svelte architecture.

### New Files to Create

```
colimail/
├── src/                              # Svelte frontend
│   ├── lib/
│   │   ├── cmvh/
│   │   │   ├── parser.ts            # Parse CMVH headers
│   │   │   ├── verifier.ts          # Local verification
│   │   │   ├── blockchain.ts        # On-chain verification
│   │   │   └── types.ts             # TypeScript types
│   │   └── components/
│   │       └── cmvh/
│   │           ├── verification-badge.svelte
│   │           ├── verification-panel.svelte
│   │           └── verification-settings.svelte
│   └── routes/
│       └── settings/
│           └── cmvh/
│               └── +page.svelte     # CMVH settings page
│
├── src-tauri/
│   └── src/
│       ├── cmvh/
│       │   ├── mod.rs               # Module definition
│       │   ├── parser.rs            # Rust header parser
│       │   ├── verifier.rs          # Rust verification
│       │   └── types.rs             # Rust types
│       └── commands/
│           └── cmvh.rs              # Tauri commands
│
└── package.json                      # Add CMVH dependencies
```

## Implementation Steps

### Step 1: Add Dependencies

#### Frontend (package.json)
```json
{
  "dependencies": {
    "@colimail/cmvh-js": "file:../colimail-cmvh/sdk/cmvh-js",
    "viem": "^2.21.0"
  }
}
```

#### Backend (Cargo.toml)
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
hex = "0.4"
sha3 = "0.10"  # For keccak256
secp256k1 = "0.28"
```

### Step 2: Rust Backend - CMVH Module

#### File: `src-tauri/src/cmvh/types.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CMVHHeaders {
    pub version: String,
    pub address: String,
    pub chain: String,
    pub timestamp: String,
    pub hash_algo: String,
    pub signature: String,
    pub ens: Option<String>,
    pub reward: Option<String>,
    pub proof_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContent {
    pub subject: String,
    pub from: String,
    pub to: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub signer_address: Option<String>,
    pub ens_name: Option<String>,
    pub timestamp: Option<String>,
    pub chain: Option<String>,
    pub error: Option<String>,
}
```

#### File: `src-tauri/src/cmvh/parser.rs`
```rust
use super::types::CMVHHeaders;
use std::collections::HashMap;

pub fn parse_cmvh_headers(raw_headers: &str) -> Result<CMVHHeaders, String> {
    let mut headers_map: HashMap<String, String> = HashMap::new();

    // Parse header lines
    for line in raw_headers.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            if key.starts_with("X-CMVH-") {
                headers_map.insert(key.to_string(), value.trim().to_string());
            }
        }
    }

    // Extract required fields
    let version = headers_map.get("X-CMVH-Version")
        .ok_or("Missing X-CMVH-Version")?
        .clone();
    let address = headers_map.get("X-CMVH-Address")
        .ok_or("Missing X-CMVH-Address")?
        .clone();
    let chain = headers_map.get("X-CMVH-Chain")
        .ok_or("Missing X-CMVH-Chain")?
        .clone();
    let timestamp = headers_map.get("X-CMVH-Timestamp")
        .ok_or("Missing X-CMVH-Timestamp")?
        .clone();
    let hash_algo = headers_map.get("X-CMVH-HashAlgo")
        .ok_or("Missing X-CMVH-HashAlgo")?
        .clone();
    let signature = headers_map.get("X-CMVH-Signature")
        .ok_or("Missing X-CMVH-Signature")?
        .clone();

    // Extract optional fields
    let ens = headers_map.get("X-CMVH-ENS").cloned();
    let reward = headers_map.get("X-CMVH-Reward").cloned();
    let proof_url = headers_map.get("X-CMVH-ProofURL").cloned();

    Ok(CMVHHeaders {
        version,
        address,
        chain,
        timestamp,
        hash_algo,
        signature,
        ens,
        reward,
        proof_url,
    })
}

pub fn validate_cmvh_headers(headers: &CMVHHeaders) -> Result<(), String> {
    // Validate version
    if headers.version != "1" {
        return Err(format!("Unsupported version: {}", headers.version));
    }

    // Validate hash algorithm
    if headers.hash_algo != "keccak256" {
        return Err(format!("Unsupported hash algorithm: {}", headers.hash_algo));
    }

    // Validate address format (0x + 40 hex chars)
    if !headers.address.starts_with("0x") || headers.address.len() != 42 {
        return Err("Invalid address format".to_string());
    }

    // Validate signature format (0x + 130 hex chars)
    if !headers.signature.starts_with("0x") || headers.signature.len() != 132 {
        return Err("Invalid signature format".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_headers() {
        let raw = r#"
From: alice@example.com
To: bob@example.com
Subject: Test
X-CMVH-Version: 1
X-CMVH-Address: 0x1234567890123456789012345678901234567890
X-CMVH-Chain: Arbitrum
X-CMVH-Timestamp: 1730733600
X-CMVH-HashAlgo: keccak256
X-CMVH-Signature: 0x123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012
        "#;

        let result = parse_cmvh_headers(raw);
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert_eq!(headers.version, "1");
        assert_eq!(headers.chain, "Arbitrum");
    }
}
```

#### File: `src-tauri/src/cmvh/verifier.rs`
```rust
use super::types::{CMVHHeaders, EmailContent, VerificationResult};
use sha3::{Digest, Keccak256};
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1, PublicKey};

pub fn canonicalize_email(content: &EmailContent) -> String {
    format!("{}\n{}\n{}\n{}",
        content.subject,
        content.from,
        content.to,
        content.body
    )
}

pub fn hash_email(content: &EmailContent) -> Vec<u8> {
    let canonical = canonicalize_email(content);
    let mut hasher = Keccak256::new();
    hasher.update(canonical.as_bytes());
    hasher.finalize().to_vec()
}

pub fn verify_signature(
    headers: &CMVHHeaders,
    content: &EmailContent,
) -> VerificationResult {
    // Compute email hash
    let email_hash = hash_email(content);

    // Parse signature
    let signature_hex = headers.signature.trim_start_matches("0x");
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(e) => return VerificationResult {
            is_valid: false,
            signer_address: None,
            ens_name: None,
            timestamp: None,
            chain: None,
            error: Some(format!("Invalid signature hex: {}", e)),
        },
    };

    // Verify signature and recover address
    // Note: This is a simplified version. Full implementation would:
    // 1. Parse recoverable signature (65 bytes)
    // 2. Recover public key from signature + hash
    // 3. Derive Ethereum address from public key
    // 4. Compare with claimed address

    // For now, return success if format is valid
    // TODO: Implement full ECDSA verification

    VerificationResult {
        is_valid: true,
        signer_address: Some(headers.address.clone()),
        ens_name: headers.ens.clone(),
        timestamp: Some(headers.timestamp.clone()),
        chain: Some(headers.chain.clone()),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_email() {
        let content = EmailContent {
            subject: "Test".to_string(),
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            body: "Hello".to_string(),
        };

        let canonical = canonicalize_email(&content);
        assert_eq!(canonical, "Test\nalice@example.com\nbob@example.com\nHello");
    }

    #[test]
    fn test_hash_email() {
        let content = EmailContent {
            subject: "Test".to_string(),
            from: "alice@example.com".to_string(),
            to: "bob@example.com".to_string(),
            body: "Hello".to_string(),
        };

        let hash = hash_email(&content);
        assert_eq!(hash.len(), 32); // keccak256 produces 32 bytes
    }
}
```

#### File: `src-tauri/src/commands/cmvh.rs`
```rust
use crate::cmvh::{parser, verifier, types::{CMVHHeaders, EmailContent, VerificationResult}};
use tauri::{command, Runtime, AppHandle};

#[command]
pub async fn parse_cmvh_headers<R: Runtime>(
    _app: AppHandle<R>,
    raw_headers: String,
) -> Result<CMVHHeaders, String> {
    parser::parse_cmvh_headers(&raw_headers)
}

#[command]
pub async fn verify_cmvh_signature<R: Runtime>(
    _app: AppHandle<R>,
    headers: CMVHHeaders,
    content: EmailContent,
) -> Result<VerificationResult, String> {
    // Validate headers first
    parser::validate_cmvh_headers(&headers)?;

    // Verify signature
    Ok(verifier::verify_signature(&headers, &content))
}

#[command]
pub async fn hash_email_content<R: Runtime>(
    _app: AppHandle<R>,
    content: EmailContent,
) -> Result<String, String> {
    let hash = verifier::hash_email(&content);
    Ok(format!("0x{}", hex::encode(hash)))
}
```

### Step 3: Svelte Frontend - Components

#### File: `src/lib/cmvh/types.ts`
```typescript
export interface CMVHHeaders {
  "X-CMVH-Version": string;
  "X-CMVH-Address": string;
  "X-CMVH-Chain": string;
  "X-CMVH-Timestamp": string;
  "X-CMVH-HashAlgo": string;
  "X-CMVH-Signature": string;
  "X-CMVH-ENS"?: string;
  "X-CMVH-Reward"?: string;
  "X-CMVH-ProofURL"?: string;
}

export interface VerificationResult {
  isValid: boolean;
  signerAddress?: string;
  ensName?: string;
  timestamp?: string;
  chain?: string;
  error?: string;
}

export interface VerificationState {
  status: "idle" | "verifying" | "verified" | "invalid" | "error";
  result?: VerificationResult;
  error?: string;
}
```

#### File: `src/lib/components/cmvh/verification-badge.svelte`
```svelte
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import type { VerificationState } from "$lib/cmvh/types";

  interface Props {
    verification: VerificationState;
    onclick?: () => void;
  }

  let { verification, onclick }: Props = $props();

  const statusConfig = {
    idle: { label: "Not Verified", variant: "secondary" as const, icon: "⚪" },
    verifying: { label: "Verifying...", variant: "secondary" as const, icon: "⏳" },
    verified: { label: "Verified", variant: "default" as const, icon: "✓" },
    invalid: { label: "Invalid", variant: "destructive" as const, icon: "❌" },
    error: { label: "Error", variant: "destructive" as const, icon: "⚠️" }
  };

  const config = $derived(statusConfig[verification.status]);
</script>

<Badge
  variant={config.variant}
  class="cursor-pointer hover:opacity-80 transition-opacity"
  onclick={onclick}
>
  <span class="mr-1">{config.icon}</span>
  {config.label}
  {#if verification.result?.chain}
    <span class="ml-1 opacity-70">({verification.result.chain})</span>
  {/if}
</Badge>
```

#### File: `src/lib/components/cmvh/verification-panel.svelte`
```svelte
<script lang="ts">
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import type { VerificationResult } from "$lib/cmvh/types";

  interface Props {
    result: VerificationResult;
    onVerifyOnChain?: () => void;
  }

  let { result, onVerifyOnChain }: Props = $props();

  function formatAddress(address: string): string {
    if (!address) return "";
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }

  function formatTimestamp(timestamp: string): string {
    const date = new Date(parseInt(timestamp) * 1000);
    return date.toLocaleString();
  }
</script>

<Card.Root class="w-full max-w-md">
  <Card.Header>
    <Card.Title>Email Verification</Card.Title>
    <Card.Description>
      This email has been signed with blockchain verification
    </Card.Description>
  </Card.Header>

  <Card.Content class="space-y-4">
    {#if result.isValid}
      <div class="flex items-center gap-2">
        <span class="text-2xl">✓</span>
        <div>
          <p class="font-semibold">Signature Verified</p>
          <p class="text-sm text-muted-foreground">
            This email's authenticity has been confirmed
          </p>
        </div>
      </div>

      <div class="space-y-2">
        {#if result.ensName}
          <div>
            <p class="text-sm font-medium">Signer</p>
            <p class="text-sm">{result.ensName}</p>
          </div>
        {/if}

        {#if result.signerAddress}
          <div>
            <p class="text-sm font-medium">Address</p>
            <p class="text-sm font-mono">{formatAddress(result.signerAddress)}</p>
          </div>
        {/if}

        {#if result.chain}
          <div>
            <p class="text-sm font-medium">Chain</p>
            <Badge variant="outline">{result.chain}</Badge>
          </div>
        {/if}

        {#if result.timestamp}
          <div>
            <p class="text-sm font-medium">Signed</p>
            <p class="text-sm">{formatTimestamp(result.timestamp)}</p>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex items-center gap-2 text-destructive">
        <span class="text-2xl">❌</span>
        <div>
          <p class="font-semibold">Verification Failed</p>
          <p class="text-sm">{result.error || "Unknown error"}</p>
        </div>
      </div>
    {/if}
  </Card.Content>

  <Card.Footer class="flex gap-2">
    {#if onVerifyOnChain && result.isValid}
      <Button variant="outline" onclick={onVerifyOnChain}>
        Verify On-Chain
      </Button>
    {/if}
    <Button variant="ghost" onclick={() => window.open(`https://arbiscan.io/address/${result.signerAddress}`, '_blank')}>
      View on Arbiscan
    </Button>
  </Card.Footer>
</Card.Root>
```

## Next Steps

1. **Deploy Contract to Arbitrum Sepolia**
2. **Implement on-chain verification using viem**
3. **Add CMVH verification to email viewer**
4. **Create settings page for CMVH configuration**
5. **Write integration tests**

## Testing Checklist

- [ ] Rust parser tests pass
- [ ] Rust verifier tests pass
- [ ] Tauri commands work correctly
- [ ] Svelte components render properly
- [ ] Local verification works
- [ ] On-chain verification works
- [ ] Settings persist correctly

---

**Status**: Ready for implementation
**Next**: Deploy contract and integrate with client
