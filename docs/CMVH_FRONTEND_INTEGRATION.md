# CMVH Frontend Integration - User Guide

## Overview

The CMVH (ColiMail Verification Header) frontend integration allows users to sign their outgoing emails with blockchain-verifiable signatures. This document explains how to configure and use CMVH email signing.

## Features Implemented

### 1. Settings Configuration (CMVH Verification Page)

**Location**: Settings → CMVH Verification

**Email Signature Creation Section**:
- ✅ Enable/disable CMVH signing globally
- ✅ Private key input with show/hide toggle
- ✅ Derive Ethereum address from private key
- ✅ Address display (read-only)
- ✅ Security warning about private key storage

**Email Signature Verification Section** (existing):
- ✅ Enable/disable CMVH verification
- ✅ Auto-verify on email open
- ✅ On-chain verification toggle
- ✅ Blockchain network selection (Arbitrum Sepolia/Mainnet)
- ✅ RPC endpoint configuration
- ✅ Contract address display

### 2. Compose Dialog Integration

**Location**: Compose Email Dialog

**CMVH Signing Toggle**:
- ✅ Checkbox to enable CMVH signing for current email
- ✅ Only visible when CMVH signing is configured in Settings
- ✅ Shows signing address when enabled
- ✅ Visual indicator (🔐 badge) when signing is active

### 3. Send Email Handler

**Location**: `src/routes/handlers/compose-send.ts`

**CMVH Signing Flow**:
1. ✅ Check if CMVH signing is enabled for current email
2. ✅ Validate CMVH configuration (private key exists)
3. ✅ Call `sign_email_with_cmvh` backend command
4. ✅ Call `send_email_with_cmvh` with generated headers
5. ✅ Show success toast with CMVH confirmation
6. ✅ Fallback to regular sending if CMVH disabled

## User Workflow

### Step 1: Configure CMVH Signing (One-time Setup)

1. Open Settings (⚙️ icon)
2. Navigate to "CMVH Verification" section
3. Scroll to "Email Signature Creation"
4. Check "Enable CMVH Signing"
5. Enter your Ethereum private key (64 hex characters)
   - You can include or omit the `0x` prefix
   - Example: `abcd1234...` or `0xabcd1234...`
6. Click "Derive Address" button
7. Verify your Ethereum address appears correctly
8. Click "Save Settings"

**Security Note**: Your private key is stored in browser localStorage. Consider using a dedicated key for email signing, separate from your main wallet.

### Step 2: Send CMVH-Signed Email

1. Click "Compose" button
2. Fill in recipient, subject, and body
3. Check the "Sign this email with CMVH (blockchain verification)" checkbox
   - This checkbox only appears if CMVH signing is configured
4. Verify your signing address is displayed below the checkbox
5. Click "Send"

**What Happens**:
- Email is signed with your private key
- CMVH headers are generated (X-CMVH-Version, X-CMVH-Signature, etc.)
- Email is sent with CMVH headers injected
- Success toast confirms "Email signed with CMVH and sent successfully!"

### Step 3: Verify Received CMVH Emails (Future)

*This feature is planned but not yet implemented in the UI. See "Future Work" section.*

## Technical Details

### Configuration Storage

**CMVH Config** (`localStorage` → `cmvh_config` key):
```typescript
{
  enabled: boolean;              // Enable CMVH verification
  autoVerify: boolean;           // Auto-verify on email open
  verifyOnChain: boolean;        // Use blockchain verification
  rpcUrl: string;                // RPC endpoint
  network: "arbitrum-sepolia" | "arbitrum";
  contractAddress: string;       // CMVHVerifier contract
  enableSigning: boolean;        // Enable CMVH signing
  privateKey: string;            // Hex-encoded private key (no 0x)
  derivedAddress: string;        // Ethereum address
}
```

**Application State** (`src/routes/lib/state.svelte.ts`):
```typescript
enableCMVHSigning: boolean;  // Per-email signing toggle
```

### Backend Commands Used

1. **`derive_eth_address(privateKeyHex: string) -> string`**
   - Derives Ethereum address from private key
   - Called when user clicks "Derive Address" button

2. **`sign_email_with_cmvh(...) -> CMVHHeaders`**
   - Signs email content with private key
   - Returns CMVH headers structure
   - Called before sending if CMVH signing enabled

3. **`send_email_with_cmvh(..., cmvhHeaders: CMVHHeaders) -> string`**
   - Sends email with CMVH headers injected
   - Called instead of regular `send_email` when signing enabled

### CMVH Headers Structure

```typescript
interface CMVHHeaders {
  version: string;       // CMVH protocol version
  address: string;       // Signer's Ethereum address
  chain: string;         // Blockchain network
  timestamp: string;     // Unix timestamp
  hash_algo: string;     // Hashing algorithm (keccak256)
  signature: string;     // ECDSA signature (hex)
  ens?: string;          // ENS name (optional)
  reward?: string;       // Reward amount (optional)
  proof_url?: string;    // Proof URL (optional)
}
```

## Code Changes Summary

### Files Modified

1. **`src/lib/cmvh/types.ts`**
   - Added `enableSigning`, `privateKey`, `derivedAddress` to `CMVHConfig`
   - Updated `DEFAULT_CMVH_CONFIG` with new fields

2. **`src/routes/components/SettingsDialog.svelte`**
   - Added private key input with show/hide toggle
   - Added "Derive Address" button and handler
   - Added derived address display
   - Added validation for signing configuration
   - Added security warning

3. **`src/routes/lib/state.svelte.ts`**
   - Added `enableCMVHSigning` boolean state
   - Reset `enableCMVHSigning` in `resetComposeState()`

4. **`src/routes/components/ComposeDialog.svelte`**
   - Added CMVH signing toggle checkbox
   - Added signing address display
   - Added visual indicator badge
   - Conditional rendering based on CMVH config

5. **`src/routes/+page.svelte`**
   - Added `bind:enableCMVHSigning` to ComposeDialog

6. **`src/routes/lib/types.ts`**
   - Added `CMVHHeaders` interface

7. **`src/routes/handlers/compose-send.ts`**
   - Imported CMVH types and config
   - Added CMVH signing logic in `handleSendEmail`
   - Call `sign_email_with_cmvh` if enabled
   - Call `send_email_with_cmvh` with headers
   - Show CMVH-specific success toast

### Lines of Code Added

- **Settings UI**: ~100 lines (private key management)
- **Compose Dialog**: ~30 lines (signing toggle)
- **Send Handler**: ~50 lines (signing logic)
- **Types**: ~20 lines (CMVHConfig updates)
- **Total**: ~200 lines of frontend code

## Testing Checklist

### Settings Configuration
- [ ] Open Settings → CMVH Verification
- [ ] Enable CMVH Signing checkbox
- [ ] Enter valid private key (64 hex chars)
- [ ] Click "Derive Address" - should show Ethereum address
- [ ] Toggle "Show/Hide" button - should mask/unmask key
- [ ] Save settings - should show success toast
- [ ] Reload app - settings should persist

### Compose Email
- [ ] Click "Compose" button
- [ ] CMVH signing checkbox should appear (if configured)
- [ ] Check CMVH signing checkbox
- [ ] Signing address should display below checkbox
- [ ] Badge should show "🔐 Signing enabled"

### Send Email
- [ ] Fill in recipient, subject, body
- [ ] Enable CMVH signing
- [ ] Click "Send"
- [ ] Should show "Email signed with CMVH and sent successfully!" toast
- [ ] Email should be sent (check server logs for CMVH headers)

### Error Handling
- [ ] Try to send CMVH email without configuring private key - should show error
- [ ] Try invalid private key format - should show validation error
- [ ] Network failure during signing - should show error toast

## Known Limitations

### Current Implementation
1. **Reply/Forward**: CMVH signing only works for new emails, not replies or forwards (can be extended in future).

2. **Verification UI**: Receiving and verifying CMVH-signed emails is not yet implemented in the UI.

3. **BCC Support**: BCC recipients are not currently supported for CMVH-signed emails.

### Security Considerations
1. **Private Key Storage**: Keys are stored in browser localStorage (not encrypted). Users should use dedicated signing keys.

2. **Key Exposure**: Showing private key in UI (even with toggle) poses security risk. Consider warning users.

3. **No Key Derivation**: No support for mnemonic phrases or hardware wallets.

## Recent Updates

### ✅ SMTP Sending Completed (2025-11-12)
- Implemented raw SMTP sending using lettre's `send_raw()` API
- CMVH-signed emails are now **actually sent** via SMTP
- Full OAuth2 and Basic Auth support
- See `docs/CMVH_SMTP_IMPLEMENTATION.md` for technical details

## Future Work

### Short-term
1. **Reply/Forward Support**: Extend CMVH signing to reply and forward modes
2. **Better Key Management**: Add key import/export, encryption at rest
3. **BCC Support**: Add BCC recipient handling

### Medium-term
1. **Verification UI**: Add CMVH verification badges in email list
2. **Verification Panel**: Show detailed verification info in email body view
3. **On-chain Verification**: Implement blockchain verification UI

### Long-term
1. **Hardware Wallet Support**: Integrate with MetaMask or hardware wallets
2. **ENS Integration**: Auto-resolve ENS names for signer addresses
3. **Multi-key Support**: Allow multiple signing keys per account
4. **Signature History**: Track all signed emails with timestamps

## Troubleshooting

### "CMVH signing enabled but private key not configured"
- Go to Settings → CMVH Verification
- Ensure "Enable CMVH Signing" is checked
- Enter private key and derive address
- Save settings

### "Invalid private key format"
- Private key must be exactly 64 hexadecimal characters
- Can start with `0x` or not
- No spaces or special characters

### "Failed to derive address"
- Check private key is valid hex
- Check backend logs for detailed error
- Ensure Rust backend is running

### CMVH checkbox not showing in Compose Dialog
- Check Settings → CMVH Verification
- Ensure "Enable CMVH Signing" is enabled
- Ensure private key is configured and address derived
- Ensure settings were saved

## References

- **Backend Documentation**: `docs/CMVH_BACKEND_COMPLETE.md`
- **CMVH Specification**: See main project README
- **Rust Commands**: `src-tauri/src/commands/cmvh.rs`, `send_cmvh.rs`
- **Frontend State**: `src/routes/lib/state.svelte.ts`
