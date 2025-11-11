# CMVHVerifier Deployment Guide

**Version**: 1.0.0
**Target Networks**: Arbitrum Sepolia (testnet), Arbitrum One (mainnet)

## Prerequisites

### 1. Set Up Wallet
- Create a dedicated deployment wallet
- Fund with Arbitrum Sepolia ETH (get from faucet)
- Export private key

### 2. Get RPC Access
- **Free Options**:
  - Arbitrum Public RPC: `https://sepolia-rollup.arbitrum.io/rpc`
  - Alchemy: https://www.alchemy.com/
  - Infura: https://infura.io/

### 3. Environment Setup

Create `.env` file in `contracts/` directory:

```bash
# Deployment wallet private key (NEVER commit this!)
PRIVATE_KEY=your_private_key_here

# RPC URLs (optional, defaults to public RPCs)
ARBITRUM_SEPOLIA_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# Arbiscan API key for contract verification (optional)
ARBISCAN_API_KEY=your_arbiscan_api_key
```

## Deployment Steps

### Step 1: Compile Contract

```bash
cd contracts
npx hardhat compile
```

**Expected Output**:
```
Compiled 2 Solidity files with solc 0.8.28
```

### Step 2: Run Tests

```bash
npx hardhat test
```

**Verify**:
- ✅ 27 tests passing
- ✅ Gas estimates < 100k

### Step 3: Deploy to Arbitrum Sepolia (Testnet)

#### Option A: Using Deployment Script

```bash
npx hardhat run scripts/deploy.ts --network arbitrumSepolia
```

#### Option B: Using Hardhat Ignition

```bash
npx hardhat ignition deploy ignition/modules/CMVHVerifier.ts --network arbitrumSepolia
```

**Expected Output**:
```
🚀 Deploying CMVHVerifier Contract...

📍 Network: arbitrumSepolia (Chain ID: 421614)

⏳ Deploying contract...
✅ CMVHVerifier deployed at: 0x1234...5678

🔍 Verifying deployment...
   Name: CMVHVerifier
   Version: 1.0.0

============================================================
📋 DEPLOYMENT SUMMARY
============================================================
Contract:        CMVHVerifier v1.0.0
Address:         0x1234...5678
Network:         arbitrumSepolia
Chain ID:        421614
============================================================

✨ Deployment complete!
```

### Step 4: Verify Contract on Arbiscan

#### Automatic Verification

```bash
npx hardhat verify --network arbitrumSepolia <CONTRACT_ADDRESS>
```

#### Manual Verification

1. Go to https://sepolia.arbiscan.io/
2. Navigate to your contract address
3. Click "Contract" tab → "Verify and Publish"
4. Fill in:
   - Compiler: 0.8.28
   - Optimization: Yes (200 runs)
   - Source code: Copy from `contracts/CMVHVerifier.sol`

### Step 5: Test Deployed Contract

Create test script: `contracts/scripts/test-deployment.ts`

```typescript
import { network } from "hardhat";
import { keccak256, type Hex } from "viem";
import { privateKeyToAccount } from "viem/accounts";

async function main() {
  const { viem } = await network.connect();
  const publicClient = await viem.getPublicClient();

  // Contract address from deployment
  const VERIFIER_ADDRESS = "0x..."; // Replace with your deployed address

  // Test account
  const TEST_PRIVATE_KEY: Hex = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const testAccount = privateKeyToAccount(TEST_PRIVATE_KEY);

  // Test email
  const testEmail = {
    subject: "Test Email",
    from: "alice@example.com",
    to: "bob@example.com",
    body: "Hello world"
  };

  // Canonicalize and hash
  const canonical = `${testEmail.subject}\n${testEmail.from}\n${testEmail.to}\n${testEmail.body}`;
  const emailHash = keccak256(new TextEncoder().encode(canonical));

  // Sign
  const signature = await testAccount.sign({ hash: emailHash });

  // Verify on-chain
  const isValid = await publicClient.readContract({
    address: VERIFIER_ADDRESS,
    abi: [{
      name: "verifySignature",
      type: "function",
      stateMutability: "view",
      inputs: [
        { name: "signer", type: "address" },
        { name: "emailHash", type: "bytes32" },
        { name: "signature", type: "bytes" }
      ],
      outputs: [{ name: "isValid", type: "bool" }]
    }],
    functionName: "verifySignature",
    args: [testAccount.address, emailHash, signature]
  });

  console.log("\n🧪 Test Result:");
  console.log(`   Email Hash: ${emailHash}`);
  console.log(`   Signer: ${testAccount.address}`);
  console.log(`   Valid: ${isValid ? "✅ YES" : "❌ NO"}`);

  if (isValid) {
    console.log("\n✅ Contract is working correctly!");
  } else {
    console.log("\n❌ Verification failed!");
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
```

Run test:
```bash
npx hardhat run scripts/test-deployment.ts --network arbitrumSepolia
```

## Deployment to Mainnet (Arbitrum One)

### Additional Prerequisites

1. **Security Audit**: Complete smart contract audit
2. **Sufficient ETH**: Ensure enough ETH for deployment gas (~$5-10)
3. **Backup**: Save deployment keys and addresses securely

### Deployment Command

```bash
npx hardhat run scripts/deploy.ts --network arbitrum
```

### Post-Deployment Checklist

- [ ] Contract verified on Arbiscan
- [ ] Deployment address documented
- [ ] Test transaction successful
- [ ] Update client configuration
- [ ] Announce deployment

## Deployment Addresses

### Testnet (Arbitrum Sepolia)

| Date | Address | Deployer | Status |
|------|---------|----------|--------|
| TBD | `0x...` | TBD | 🚧 Pending |

### Mainnet (Arbitrum One)

| Date | Address | Deployer | Status |
|------|---------|----------|--------|
| TBD | `0x...` | TBD | ⏳ Phase 3+ |

## Troubleshooting

### Error: Insufficient Funds

**Solution**: Get Arbitrum Sepolia ETH from faucet
- https://faucet.triangleplatform.com/arbitrum/sepolia
- https://www.alchemy.com/faucets/arbitrum-sepolia

### Error: Nonce Too Low

**Solution**: Reset account nonce
```bash
npx hardhat clean
# Try deployment again
```

### Error: Contract Already Deployed

**Solution**: Contract is already at this address
- Check if deployment was successful
- Use existing contract address

### Error: Verification Failed

**Solution**: Try manual verification on Arbiscan
- Copy contract source code
- Ensure compiler version matches (0.8.28)
- Enable optimization (200 runs)

## Gas Costs Estimate

| Network | Deployment | Verification (per call) |
|---------|------------|------------------------|
| Arbitrum Sepolia | ~$0 (testnet) | ~$0.0001 |
| Arbitrum One | ~$1-5 | ~$0.001-0.01 |

## Security Best Practices

1. **Never Commit Private Keys**: Use `.gitignore` for `.env`
2. **Use Hardware Wallet**: For mainnet deployments
3. **Test First**: Always deploy to testnet first
4. **Verify Contracts**: Make contracts publicly verifiable
5. **Document Everything**: Keep deployment records

## Integration with Client

After deployment, update client configuration:

```typescript
// client/src/lib/cmvh/config.ts
export const CMVH_CONFIG = {
  arbitrumSepolia: {
    chainId: 421614,
    rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
    verifierAddress: "0x...", // Your deployed address
  },
  arbitrum: {
    chainId: 42161,
    rpcUrl: "https://arb1.arbitrum.io/rpc",
    verifierAddress: "0x...", // Mainnet address (Phase 3+)
  }
};
```

## Next Steps

1. Deploy to Arbitrum Sepolia
2. Test with real emails
3. Integrate with ColiMail client
4. Security audit (before mainnet)
5. Deploy to Arbitrum One

---

**Last Updated**: 2025-11-10
**Status**: Ready for deployment
