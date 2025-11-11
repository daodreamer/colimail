# CMVH Integration in ColiMail

**Date**: 2025-11-11
**Status**: ✅ Phase 3 Implementation Complete
**Contract**: [0xf251c131d6b9f71992e2ba43023d3b52588dbd02](https://sepolia.arbiscan.io/address/0xf251c131d6b9f71992e2ba43023d3b52588dbd02) (Arbitrum Sepolia)

## Overview

ColiMail now includes **CMVH (ColiMail Verification Header)** support, enabling blockchain-based email signature verification. This integration allows users to:

- ✅ **Verify email signatures** cryptographically
- ✅ **Authenticate sender identity** using Ethereum addresses
- ✅ **View ENS names** for verified senders
- ✅ **On-chain verification** via Arbitrum smart contracts (optional)

## Architecture

### Backend (Rust)

**Location**: `src-tauri/src/cmvh/`

```
cmvh/
├── mod.rs           # Module exports
├── types.rs         # Data structures (CMVHHeaders, EmailContent, VerificationResult)
├── parser.rs        # Parse X-CMVH-* headers from email
└── verifier.rs      # ECDSA signature verification (secp256k1 + keccak256)
```

**Key Features**:
- **Parser**: Extracts CMVH headers from raw email headers
- **Verifier**: Performs local ECDSA signature verification
- **Recovery**: Recovers Ethereum address from signature
- **Validation**: Ensures header format and values are correct

### Frontend (TypeScript + Svelte)

**Location**: `src/lib/cmvh/`

```
cmvh/
├── index.ts          # Main exports
├── types.ts          # TypeScript interfaces
├── verifier.ts       # Verification logic
├── blockchain.ts     # On-chain verification (viem)
└── config.ts         # Configuration management
```

**Components**: `src/lib/components/cmvh/`

```
cmvh/
├── verification-badge.svelte    # Status badge for email list
└── verification-panel.svelte    # Detailed verification info panel
```

**Settings Page**: `src/routes/settings/cmvh/+page.svelte`

## Usage

### 1. Enable CMVH Verification

Navigate to **Settings → CMVH Verification** to configure:

- **Enable Email Signature Verification**: Turn on/off CMVH
- **Auto-verify on Email Open**: Automatically verify when opening emails
- **Enable On-Chain Verification**: Verify signatures using smart contracts
- **Network**: Choose Arbitrum Sepolia (testnet) or Arbitrum One (mainnet)
- **RPC Endpoint**: Configure custom RPC URL (optional)

### 2. Reading Verified Emails

When you receive an email with CMVH headers:

1. **Email List**: A verification badge appears next to the email
   - ✅ **Green "Verified"**: Signature verified locally
   - 🔵 **Blue "On-Chain Verified"**: Verified via smart contract
   - ❌ **Red "Invalid"**: Signature verification failed
   - ⚪ **Gray "Not Signed"**: No CMVH headers found

2. **Email Detail**: Click the badge to see verification details:
   - Signer's Ethereum address
   - ENS name (if available)
   - Blockchain network
   - Timestamp
   - Link to Arbiscan explorer

3. **On-Chain Verification**: Click "Verify On-Chain" to verify via smart contract (requires RPC access)

### 3. For Developers: Integration Example

#### Verify Email in Your Code

```typescript
import { verifyEmail } from "$lib/cmvh";

// Verify email with CMVH headers
const verificationState = await verifyEmail(rawHeaders, {
  subject: "Hello World",
  from: "alice@example.com",
  to: "bob@example.com",
  body: "Email content...",
});

console.log(verificationState.status); // "verified-local" | "invalid" | "error" | "idle"
console.log(verificationState.result?.signer_address); // "0x1234...5678"
```

#### Display Verification Badge

```svelte
<script>
  import VerificationBadge from "$lib/components/cmvh/verification-badge.svelte";

  let verification = $state({ status: "verified-local", result: {...} });
</script>

<VerificationBadge {verification} onclick={() => showDetails()} />
```

#### On-Chain Verification

```typescript
import { verifyOnChain } from "$lib/cmvh/blockchain";
import { loadConfig } from "$lib/cmvh/config";

const config = loadConfig();
const result = await verifyOnChain(headers, content, config);

if (result.isValid) {
  console.log("✅ On-chain verification passed!");
} else {
  console.error("❌ Verification failed:", result.error);
}
```

## Technical Details

### CMVH Header Format

Emails with CMVH signatures include these headers:

```
X-CMVH-Version: 1
X-CMVH-Address: 0x1234567890123456789012345678901234567890
X-CMVH-Chain: Arbitrum
X-CMVH-Timestamp: 1730733600
X-CMVH-HashAlgo: keccak256
X-CMVH-Signature: 0xabcd...ef01
X-CMVH-ENS: alice.eth (optional)
X-CMVH-Reward: 0.05 wACT (optional)
X-CMVH-ProofURL: ipfs://... (optional)
```

### Signature Verification Process

1. **Canonicalization**: Email content is normalized to:
   ```
   {subject}\n{from}\n{to}\n{body}
   ```

2. **Hashing**: Content is hashed using keccak256

3. **Signature Recovery**: ECDSA signature is used to recover the public key

4. **Address Derivation**: Ethereum address is derived from public key

5. **Verification**: Recovered address is compared with claimed address

### Deployed Contracts

| Network | Contract Address | Explorer |
|---------|-----------------|----------|
| **Arbitrum Sepolia** (testnet) | `0xf251c131d6b9f71992e2ba43023d3b52588dbd02` | [View on Arbiscan](https://sepolia.arbiscan.io/address/0xf251c131d6b9f71992e2ba43023d3b52588dbd02) |
| **Arbitrum One** (mainnet) | *Not deployed yet* | TBD |

### Smart Contract Interface

```solidity
interface ICMVHVerifier {
  function verifyEmail(
    address signer,
    string calldata subject,
    string calldata from,
    string calldata to,
    string calldata body,
    bytes calldata signature
  ) external view returns (bool isValid);
}
```

## Configuration

### Default Settings

```typescript
{
  enabled: true,
  autoVerify: true,
  verifyOnChain: false,
  rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
  network: "arbitrum-sepolia",
  contractAddress: "0xf251c131d6b9f71992e2ba43023d3b52588dbd02"
}
```

### Local Storage

Configuration is stored in `localStorage` under key `cmvh_config`.

## Performance

| Operation | Average Time | Notes |
|-----------|--------------|-------|
| **Parse Headers** | <10ms | Instant |
| **Local Verification** | <50ms | ECDSA verification in Rust |
| **On-Chain Verification** | 1-3s | Depends on RPC latency |
| **Gas Cost** | ~31,462 gas | ~$0.001-0.01 on Arbitrum |

## Security Considerations

### Implemented

- ✅ **ECDSA Signature Verification**: secp256k1 curve
- ✅ **Address Recovery**: From signature + hash
- ✅ **Header Validation**: Format and content checks
- ✅ **Error Handling**: Graceful failures without exposing internals

### Known Limitations (MVP)

- ⚠️ **No Replay Protection**: Same content produces same signature
- ⚠️ **No Timestamp Validation**: Accepts future/past timestamps
- ⚠️ **EOA Only**: No support for EIP-1271 smart contract wallets
- ⚠️ **No Forward Detection**: Original signature remains valid after forwarding

These limitations are by design for the MVP and will be addressed in future phases.

## Troubleshooting

### Email shows "Invalid Signature"

**Possible Causes**:
- Email content was modified after signing
- Signature format is incorrect
- Wrong network selected in settings

**Solutions**:
1. Verify sender used correct CMVH signing tool
2. Check email wasn't modified by email server
3. Ensure correct network is selected in settings

### On-Chain Verification Fails

**Possible Causes**:
- RPC endpoint is down or rate-limited
- Network connectivity issues
- Wrong contract address

**Solutions**:
1. Check internet connection
2. Try using a different RPC URL
3. Verify contract address matches network

### Settings Don't Save

**Possible Causes**:
- LocalStorage is disabled or full
- Browser privacy settings

**Solutions**:
1. Check browser localStorage permissions
2. Clear browser cache and try again
3. Try different browser

## Future Enhancements (Phase 4+)

- 🔜 **Reward System**: wACT token integration for incentivized emails
- 🔜 **Replay Protection**: Nonce-based signature verification
- 🔜 **Timestamp Validation**: Enforce time-to-live for signatures
- 🔜 **EIP-1271 Support**: Smart contract wallet signatures
- 🔜 **Forward Detection**: Track email forwarding chains
- 🔜 **Attachment Verification**: IPFS hash-based integrity checks

## Resources

### Documentation
- **CMVH Specification**: `docs/CMVH_DEV.md`
- **Phase 3 Plan**: `docs/PHASE3_PLAN.md`
- **Phase 3 Implementation**: `docs/PHASE3_IMPLEMENTATION.md`
- **Deployment Guide**: `docs/DEPLOYMENT_GUIDE.md`

### External Links
- **CMVH Repository**: https://github.com/daodreamer/colimail-cmvh
- **Contract on Arbiscan**: https://sepolia.arbiscan.io/address/0xf251c131d6b9f71992e2ba43023d3b52588dbd02
- **ColiMail Repository**: https://github.com/daodreamer/colimail

### Support
- **GitHub Issues**: https://github.com/daodreamer/colimail/issues
- **CMVH Issues**: https://github.com/daodreamer/colimail-cmvh/issues

---

**Implementation Status**: ✅ Complete
**Last Updated**: 2025-11-11
**Next Steps**: Phase 4 - Incentive Layer
