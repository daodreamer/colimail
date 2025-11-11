# Phase 3 Implementation Plan

**Date**: 2025-11-10
**Status**: 🚧 **IN PROGRESS**
**Estimated Duration**: 3 weeks

## Overview

Phase 3 focuses on integrating CMVH verification capabilities into the ColiMail email client. This phase will enable users to verify email signatures both locally (using SDK) and on-chain (using smart contracts).

## Prerequisites

- ✅ Phase 1: SDK completed
- ✅ Phase 2: Smart contracts completed
- 📋 ColiMail client codebase (Tauri + TypeScript + React)

## Goals

1. **Email Header Parser**: Extract and parse CMVH headers from emails
2. **Local Verification**: Verify signatures using SDK (instant, no gas cost)
3. **On-Chain Verification**: Verify signatures via smart contract (optional, requires RPC)
4. **UI Integration**: Display verification status with badges/indicators
5. **Settings Panel**: Allow users to configure verification options

## Architecture Design

### Module Structure

```
client/
├── src/
│   ├── lib/
│   │   └── cmvh/
│   │       ├── parser.ts          # Header parsing
│   │       ├── verifier.ts        # Verification logic
│   │       ├── blockchain.ts      # On-chain verification
│   │       └── types.ts           # TypeScript types
│   ├── components/
│   │   └── cmvh/
│   │       ├── VerificationBadge.tsx  # Status indicator
│   │       ├── VerificationPanel.tsx  # Detailed view
│   │       └── SettingsPanel.tsx      # Configuration
│   └── hooks/
│       └── useCMVHVerification.ts     # React hook
├── src-tauri/
│   └── src/
│       └── cmvh/
│           └── mod.rs              # Rust backend integration
└── tests/
    └── cmvh/
        ├── parser.test.ts
        ├── verifier.test.ts
        └── integration.test.ts
```

## Technical Stack

### Frontend
- **React**: UI components
- **TypeScript**: Type-safe development
- **Viem**: Blockchain interactions
- **@colimail/cmvh-js**: Email signing/verification SDK

### Backend (Tauri)
- **Rust**: Native performance
- **Tauri IPC**: Frontend-backend communication
- **IMAP/SMTP**: Email protocol handling

### Blockchain
- **Arbitrum**: L2 for low gas costs
- **Viem**: Modern Ethereum library
- **CMVHVerifier Contract**: On-chain verification

## Implementation Steps

### Step 1: CMVH Parser Module

**File**: `client/src/lib/cmvh/parser.ts`

**Features**:
- Parse X-CMVH-* headers from email
- Validate header format
- Extract signature, address, chain, etc.
- Handle missing/malformed headers

**Functions**:
```typescript
parseCMVHHeaders(emailHeaders: string): CMVHHeaders | null
extractEmailContent(email: Email): EmailContent
validateCMVHHeaders(headers: CMVHHeaders): boolean
```

### Step 2: Local Verification

**File**: `client/src/lib/cmvh/verifier.ts`

**Features**:
- Verify signatures using SDK (instant)
- Cache verification results
- Handle verification errors gracefully

**Functions**:
```typescript
verifyEmailLocally(email: Email): Promise<VerificationResult>
getCachedVerification(emailId: string): VerificationResult | null
```

### Step 3: On-Chain Verification

**File**: `client/src/lib/cmvh/blockchain.ts`

**Features**:
- Connect to Arbitrum RPC
- Call CMVHVerifier contract
- Handle network errors
- Cache blockchain results

**Functions**:
```typescript
verifyEmailOnChain(
  headers: CMVHHeaders,
  content: EmailContent
): Promise<OnChainVerificationResult>

connectToArbitrum(rpcUrl?: string): PublicClient
```

### Step 4: UI Components

#### Verification Badge

**File**: `client/src/components/cmvh/VerificationBadge.tsx`

**Features**:
- Display verification status (✓ Verified / ⚠️ Unverified / ❌ Invalid)
- Show chain icon for on-chain verification
- Clickable for detailed view
- Loading state during verification

**States**:
- `verified-local`: ✓ Verified (SDK)
- `verified-onchain`: 🔵 On-Chain Verified
- `unverified`: ⚠️ No Signature
- `invalid`: ❌ Invalid Signature
- `loading`: ⏳ Verifying...

#### Verification Panel

**File**: `client/src/components/cmvh/VerificationPanel.tsx`

**Features**:
- Detailed verification information
- Signer address and ENS name
- Timestamp and chain
- Signature details
- Option to verify on-chain

#### Settings Panel

**File**: `client/src/components/cmvh/SettingsPanel.tsx`

**Features**:
- Enable/disable blockchain verification
- Configure RPC endpoint
- Set verification preferences
- Gas cost estimates

### Step 5: React Hook

**File**: `client/src/hooks/useCMVHVerification.ts`

**Features**:
- Encapsulate verification logic
- Manage verification state
- Provide loading/error states
- Auto-verify on email load

**API**:
```typescript
const {
  verification,
  isLoading,
  error,
  verifyLocally,
  verifyOnChain,
  refreshVerification
} = useCMVHVerification(email);
```

### Step 6: Tauri Backend Integration

**File**: `client/src-tauri/src/cmvh/mod.rs`

**Features**:
- Expose CMVH functions to frontend
- Email parsing in Rust
- Performance optimizations
- Error handling

**Commands**:
```rust
#[tauri::command]
async fn verify_email_signature(
  email_id: String
) -> Result<VerificationResult, String>

#[tauri::command]
async fn get_cmvh_headers(
  email_id: String
) -> Result<CMVHHeaders, String>
```

## User Experience Flow

### Email List View

```
┌────────────────────────────────────┐
│ From: alice@example.com            │
│ Subject: Partnership Proposal      │
│ [🔵 On-Chain Verified]            │ ← Badge
└────────────────────────────────────┘
```

### Email Detail View

```
┌────────────────────────────────────────────┐
│ From: alice@example.com                    │
│ To: bob@example.com                        │
│ Subject: Partnership Proposal              │
│                                            │
│ ┌────────────────────────────────────────┐ │
│ │ 🔵 On-Chain Verified                   │ │
│ │ ───────────────────────────────────    │ │
│ │ Signer: alice.eth                      │ │
│ │         (0x1234...5678)                │ │
│ │ Chain: Arbitrum                        │ │
│ │ Signed: 2025-11-10 10:30 AM           │ │
│ │                                        │ │
│ │ [View on Arbiscan] [Details]          │ │
│ └────────────────────────────────────────┘ │
│                                            │
│ Email content here...                      │
└────────────────────────────────────────────┘
```

### Settings Panel

```
┌────────────────────────────────────────────┐
│ CMVH Settings                              │
│ ──────────────────────────────────────────│
│                                            │
│ [✓] Enable Email Signature Verification   │
│                                            │
│ Verification Method:                       │
│ [✓] Local Verification (Instant, Free)    │
│ [ ] On-Chain Verification (Slower, Secure)│
│                                            │
│ Blockchain Settings:                       │
│ Network: Arbitrum One ▼                    │
│ RPC URL: [https://arb1.arbitrum.io/rpc]   │
│                                            │
│ Display Settings:                          │
│ [✓] Show verification badge in email list │
│ [✓] Auto-verify on email open             │
│ [ ] Alert on invalid signatures            │
│                                            │
│ [Save Settings]                            │
└────────────────────────────────────────────┘
```

## Performance Requirements

| Operation | Target | Notes |
|-----------|--------|-------|
| Parse headers | <10ms | Instant UI feedback |
| Local verification | <50ms | SDK verification |
| On-chain verification | <3s | Network-dependent |
| UI update | <100ms | Smooth user experience |

## Testing Strategy

### Unit Tests
- Header parsing with various formats
- Signature verification (valid/invalid)
- Error handling and edge cases

### Integration Tests
- Full verification flow
- SDK-Contract interoperability
- UI component rendering

### E2E Tests (Playwright)
- Sign email → Send → Receive → Verify
- Settings configuration
- Error scenarios

## Security Considerations

1. **RPC Endpoint Security**
   - Validate RPC responses
   - Handle malicious RPC providers
   - Rate limiting

2. **Cache Security**
   - Invalidate cache on email changes
   - Prevent cache poisoning

3. **Error Messages**
   - Don't expose sensitive info in errors
   - Clear user-facing messages

4. **Gas Cost Protection**
   - Warn users about on-chain verification costs
   - Batch verification when possible

## Deployment Checklist

- [ ] Module implementation complete
- [ ] Unit tests passing (>90% coverage)
- [ ] Integration tests passing
- [ ] E2E tests passing
- [ ] UI/UX review completed
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] Documentation complete
- [ ] User guide written

## Dependencies to Install

### Frontend
```bash
npm install viem @colimail/cmvh-js
npm install -D @testing-library/react vitest
```

### Backend (Rust)
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tauri = { version = "2.0", features = ["protocol-asset"] }
```

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| RPC provider downtime | High | Fallback RPC URLs, graceful degradation |
| Gas price spikes | Medium | Cache results, warn users |
| UI performance issues | Medium | Lazy loading, virtualization |
| IMAP parsing errors | High | Robust error handling, fallback parsing |

## Success Metrics

- [ ] 95%+ verification accuracy
- [ ] <3s average verification time
- [ ] <5% error rate
- [ ] Positive user feedback
- [ ] Smooth UI performance

## Next Steps After Phase 3

- Phase 4: Incentive Layer (reward pool)
- Phase 5: Browser extension
- Security audit
- Public beta testing

---

**Plan Status**: 📋 **READY TO IMPLEMENT**
**Last Updated**: 2025-11-10
