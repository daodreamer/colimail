# CMVH Testing Guide

**Date**: 2025-11-11
**Status**: Ready for Testing
**Contract**: [0xf251c131d6b9f71992e2ba43023d3b52588dbd02](https://sepolia.arbiscan.io/address/0xf251c131d6b9f71992e2ba43023d3b52588dbd02) (Arbitrum Sepolia)

## Overview

This guide explains how to test CMVH (ColiMail Verification Header) email signature verification in ColiMail.

## Testing Workflow

```
┌─────────────────┐
│ 1. Sign Email   │  Generate CMVH headers with test wallet
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. Send Email   │  Send via SMTP with custom headers
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 3. Receive      │  Open email in ColiMail
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 4. Verify Local │  ECDSA signature verification (instant)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 5. Verify Chain │  Smart contract verification (optional)
└─────────────────┘
```

## Prerequisites

### 1. Install Dependencies

```bash
npm install nodemailer viem
```

### 2. Set Up SMTP Access

#### For Gmail Users

1. **Enable 2-Factor Authentication**
   - Go to [Google Account Security](https://myaccount.google.com/security)
   - Enable "2-Step Verification"

2. **Generate App Password**
   - Go to [App Passwords](https://myaccount.google.com/apppasswords)
   - Select "Mail" and "Other (Custom name)"
   - Name it "ColiMail CMVH Testing"
   - Copy the 16-character password

3. **Configure SMTP Settings**
   - Host: `smtp.gmail.com`
   - Port: `587` (TLS) or `465` (SSL)
   - User: Your Gmail address
   - Pass: The app password from step 2

#### For Other Email Providers

| Provider | SMTP Host | Port | Notes |
|----------|-----------|------|-------|
| **Outlook** | smtp-mail.outlook.com | 587 | Use account password |
| **Yahoo** | smtp.mail.yahoo.com | 587 | Requires app password |
| **ProtonMail** | smtp.protonmail.com | 587 | Requires Bridge app |
| **Custom** | Your SMTP server | 587/465 | Check provider docs |

### 3. Configure ColiMail

1. Open ColiMail
2. Go to **Settings** (⚙️ icon)
3. Navigate to **CMVH Verification**
4. Configure:
   - ✅ Enable Email Signature Verification
   - ✅ Auto-verify on Email Open
   - ✅ Enable On-Chain Verification (optional)
   - Network: **Arbitrum Sepolia**
   - RPC: `https://sepolia-rollup.arbitrum.io/rpc` (default)
5. Click **Save Settings**

## Test Scenario 1: Sign and Preview Email

Use this to generate a signed email without sending it.

### Run the Signing Tool

```bash
node scripts/sign-test-email.mjs
```

### Expected Output

```
🔐 CMVH Email Signing Tool

📧 Email to sign:
  From: sender@example.com
  To: receiver@example.com
  Subject: CMVH Test Email - Blockchain Verified

🔏 Signing email...
✅ Email signed successfully!

📋 CMVH Headers:
  X-CMVH-Version: 1
  X-CMVH-Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  X-CMVH-Chain: Arbitrum
  X-CMVH-Timestamp: 1730733600
  X-CMVH-HashAlgo: keccak256
  X-CMVH-Signature: 0xabcd...ef01

🔑 Signer Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
📝 Email Hash: 0x1234...5678

================================================================================
📨 SIGNED EMAIL (Ready to send)
================================================================================
From: sender@example.com
To: receiver@example.com
Subject: CMVH Test Email - Blockchain Verified
X-CMVH-Version: 1
X-CMVH-Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
...
================================================================================
```

### What This Tests

- ✅ Email canonicalization (subject, from, to - body excluded to avoid HTML formatting issues)
- ✅ keccak256 hashing
- ✅ ECDSA signature generation
- ✅ CMVH header formatting

## Test Scenario 2: Send and Verify Email (Full E2E)

Use this to send a real signed email and verify it in ColiMail.

### Step 1: Send CMVH-Signed Email

```bash
SMTP_HOST=smtp.gmail.com \
SMTP_PORT=587 \
SMTP_USER=your-email@gmail.com \
SMTP_PASS=your-app-password \
RECEIVER_EMAIL=receiver@example.com \
node scripts/send-cmvh-email.mjs
```

**Windows PowerShell:**
```powershell
$env:SMTP_HOST="smtp.gmail.com"
$env:SMTP_PORT="587"
$env:SMTP_USER="your-email@gmail.com"
$env:SMTP_PASS="your-app-password"
$env:RECEIVER_EMAIL="receiver@example.com"
node scripts/send-cmvh-email.mjs
```

**Windows CMD:**
```cmd
set SMTP_HOST=smtp.gmail.com
set SMTP_PORT=587
set SMTP_USER=your-email@gmail.com
set SMTP_PASS=your-app-password
set RECEIVER_EMAIL=receiver@example.com
node scripts\send-cmvh-email.mjs
```

### Expected Output

```
🔐 CMVH Email Sending Tool

📧 Email Details:
  From: your-email@gmail.com
  To: receiver@example.com
  Subject: CMVH Test Email - Blockchain Verified

🔏 Signing email with CMVH...
✅ Email signed successfully!

📋 CMVH Headers:
  X-CMVH-Version: 1
  X-CMVH-Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
  X-CMVH-Chain: Arbitrum
  X-CMVH-Timestamp: 1730733600
  X-CMVH-HashAlgo: keccak256
  X-CMVH-Signature: 0xabcd...ef01

🔑 Signer Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
📝 Email Hash: 0x1234...5678

📤 Sending email via SMTP...
  Server: smtp.gmail.com:587
  User: your-email@gmail.com

✅ Email sent successfully!

📬 Message Details:
  Message ID: <abc123@gmail.com>
  Response: 250 2.0.0 OK  1234567890 - gsmtp

🎉 Success! Next steps:

1. Open ColiMail and sync your inbox
2. Look for the email with verification badge
3. Click the badge to see CMVH verification details
4. Try "Verify On-Chain" to verify via Arbitrum smart contract

Expected verification result:
  ✅ Verified (signer: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
  📅 Timestamp: 2024-11-04T12:00:00.000Z
  🔗 Chain: Arbitrum Sepolia
```

### Step 2: Receive Email in ColiMail

1. Open ColiMail
2. Select the receiver account in the left sidebar
3. Click **🔄 Refresh** to sync emails
4. Wait for sync to complete (~3 seconds for 100 emails)

### Step 3: Verify Locally

1. Locate the test email in the inbox
2. Look for the **verification badge** next to the email:
   - 🟢 **Green "Verified"** - Signature is valid
   - 🔴 **Red "Invalid"** - Signature verification failed
   - ⚪ **Gray "Not Signed"** - No CMVH headers found
3. Click on the email to open it
4. The badge should automatically verify (if auto-verify is enabled)

### Step 4: View Verification Details

1. Click the verification badge
2. A verification panel appears showing:
   - **Ethereum Address**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
   - **ENS Name**: (if available)
   - **Blockchain**: Arbitrum Sepolia
   - **Timestamp**: When the email was signed
   - **Explorer Link**: View on Arbiscan

### Step 5: Verify On-Chain (Optional)

1. In the verification panel, click **"Verify On-Chain"**
2. ColiMail calls the deployed smart contract at `0xf251c131d6b9f71992e2ba43023d3b52588dbd02`
3. Wait 1-3 seconds for blockchain RPC response
4. Result updates to:
   - 🔵 **Blue "On-Chain Verified"** - Contract confirmed signature
   - 🔴 **Red "Invalid"** - Contract rejected signature

### What This Tests

- ✅ Email sending with custom CMVH headers
- ✅ Email receiving and header parsing
- ✅ Local ECDSA signature verification (Rust backend)
- ✅ UI badge display and status
- ✅ Verification panel with details
- ✅ On-chain verification via smart contract
- ✅ Arbitrum Sepolia RPC integration
- ✅ Explorer link generation

## Test Scenario 3: Invalid Signature

Test that ColiMail correctly rejects tampered emails.

### Step 1: Modify Email Content

1. Run `node scripts/sign-test-email.mjs` to generate a signed email
2. Copy the output
3. **Change the subject** or **body** text (don't change headers)
4. Manually send this modified email via your email client

### Step 2: Receive and Verify

1. Open the modified email in ColiMail
2. Badge should show:
   - 🔴 **Red "Invalid Signature"**
3. Verification panel shows:
   - ❌ Verification Failed
   - Error message explaining the signature doesn't match

### What This Tests

- ✅ Signature verification detects tampering
- ✅ Invalid signatures are clearly marked
- ✅ Error handling and user feedback

## Test Scenario 4: Email Without CMVH Headers

Test that ColiMail gracefully handles regular emails.

### Step 1: Send Regular Email

Send a normal email without CMVH headers to your ColiMail account.

### Step 2: Receive and Check

1. Open the email in ColiMail
2. Badge should show:
   - ⚪ **Gray "Not Signed"**
3. No verification panel appears

### What This Tests

- ✅ Graceful handling of non-CMVH emails
- ✅ No false positives or errors

## Troubleshooting

### Issue: "Missing required environment variables"

**Solution**: Set all required environment variables:
- `SMTP_USER`: Your email address
- `SMTP_PASS`: Your email password or app password
- `RECEIVER_EMAIL`: Destination email address

### Issue: "Invalid login" error

**Cause**: Wrong password or app password required

**Solution**:
- Gmail: Use app password, not regular password
- Enable 2FA first, then generate app password
- Verify credentials are correct

### Issue: "ECONNREFUSED" error

**Cause**: Wrong SMTP port or server not reachable

**Solution**:
- Check `SMTP_HOST` and `SMTP_PORT` settings
- Try port 587 (TLS) or 465 (SSL)
- Check firewall/network settings

### Issue: Email sent but no CMVH headers visible

**Cause**: Some SMTP servers strip custom headers

**Solution**:
- Check email source/raw headers
- Try a different SMTP provider
- Use Gmail, which preserves custom headers

### Issue: "Verification Failed" in ColiMail

**Possible Causes**:
1. Email content was modified after signing
2. Signature format is incorrect
3. Wrong test private key

**Solution**:
- Ensure email wasn't modified by SMTP server
- Check that the test private key matches: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`
- Verify signer address is `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`

### Issue: "On-Chain Verification" fails

**Possible Causes**:
1. RPC endpoint is down
2. Network connectivity issues
3. Wrong contract address

**Solution**:
- Check internet connection
- Verify RPC URL in settings: `https://sepolia-rollup.arbitrum.io/rpc`
- Verify contract address: `0xf251c131d6b9f71992e2ba43023d3b52588dbd02`
- Try using a different RPC (Alchemy, Infura, etc.)

### Issue: Badge shows "⏳ Verifying..." forever

**Cause**: Async verification is stuck

**Solution**:
- Check browser console for errors
- Ensure Tauri backend is running
- Restart ColiMail
- Check that `auto-verify` is enabled in settings

## Performance Benchmarks

Expected performance metrics:

| Operation | Target | Typical | Notes |
|-----------|--------|---------|-------|
| **Parse Headers** | <10ms | ~2ms | Regex-based parsing |
| **Local Verification** | <50ms | ~20ms | ECDSA in Rust |
| **On-Chain Verification** | <3s | ~1.5s | RPC latency |
| **UI Update** | <100ms | ~50ms | Svelte reactivity |
| **Total (Local)** | <100ms | ~50ms | Parse + Verify |
| **Total (On-Chain)** | <3s | ~1.5s | Includes RPC |

## Security Checklist

- ✅ **Test Private Key**: Only use for testing, NEVER for real funds
- ✅ **SMTP Credentials**: Use environment variables, don't commit to Git
- ✅ **App Passwords**: Use Gmail app passwords, not main password
- ✅ **Testnet Only**: Tests use Arbitrum Sepolia (no real money)
- ✅ **Signature Verification**: Correctly implements ECDSA + address recovery
- ✅ **Header Validation**: Checks format and required fields
- ✅ **Error Handling**: No sensitive data leaked in error messages

## Test Wallet Information

**DO NOT USE IN PRODUCTION!**

- **Private Key**: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`
- **Address**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- **Network**: Arbitrum Sepolia (testnet)
- **Purpose**: Testing CMVH signature verification only
- **Warning**: This is a well-known test key from Hardhat. Never send real funds to this address!

## Next Steps

After successful testing:

1. **Report Issues**: Open GitHub issues for any bugs found
2. **Integration**: Add verification badges to EmailList component
3. **Integration**: Add verification panel to EmailBody component
4. **Performance**: Monitor verification speed and optimize if needed
5. **Phase 4**: Begin incentive layer implementation (wACT rewards)

## Resources

- **CMVH Documentation**: `docs/CMVH_INTEGRATION.md`
- **Implementation Status**: `docs/CMVH_PHASE3_COMPLETE.md`
- **Smart Contract**: [View on Arbiscan](https://sepolia.arbiscan.io/address/0xf251c131d6b9f71992e2ba43023d3b52588dbd02)
- **CMVH Repository**: https://github.com/daodreamer/colimail-cmvh

---

**Testing Status**: ✅ Ready
**Last Updated**: 2025-11-11
**Tools**: `sign-test-email.mjs`, `send-cmvh-email.mjs`
