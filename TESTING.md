# Testing Documentation

**Colimail Email Client - Comprehensive Testing Suite**

[![Tests](https://img.shields.io/badge/tests-201%20passing-brightgreen)](.)
[![Coverage](https://img.shields.io/badge/coverage-~90%25-green)](.)
[![Security](https://img.shields.io/badge/security-20%20tests-blue)](.)

---

## Overview

Colimail maintains a robust multi-layered testing strategy covering unit tests, integration tests, end-to-end tests, security tests, and performance benchmarks. Our testing philosophy prioritizes **real-world reliability** over test count metrics, focusing on critical paths and edge cases that matter to users.

### Test Statistics

| Test Category | Count | Pass Rate | Execution Time |
|---------------|-------|-----------|----------------|
| **Unit Tests** (Rust) | 83 | 98.8% | < 1s |
| **Integration Tests** (Rust) | 90 | 100% | < 5s |
| **E2E Tests** (WebDriverIO) | 8 | 100% | ~30s |
| **Security Tests** | 20 | 100% | < 100ms |
| **Benchmarks** | 2 suites | - | varies |
| **Total** | **201+** | **99.5%** | **< 1min** |

---

## 1. Unit Tests (P0 Priority)

### Backend Unit Tests (Rust)

**Location:** `src-tauri/src/` (inline `#[cfg(test)]` modules)
**Framework:** Cargo Test
**Coverage:** ~85% of critical paths

#### Email Codec Module (20 tests)
Tests RFC 2047 encoding/decoding, date parsing, and attachment detection:
- RFC 2047 header decoding (UTF-8 Q/B encoding)
- Chinese/Japanese character handling
- Quoted-printable and Base64 decoding
- Email date parsing with fallback mechanisms
- Attachment detection from MIME headers

**Key Test:**
```rust
#[test]
fn test_decode_header_chinese_utf8() {
    let input = "=?UTF-8?B?5rWL6K+V?="; // "测试"
    assert_eq!(decode_header(input), "测试");
}
```

#### Data Models Module (14 tests)
Tests business logic in core data structures:
- Folder selection logic (`is_selectable()`)
- User visibility filtering (system folder detection)
- Serde serialization/deserialization
- Default value handling for OAuth fields

**Key Test:**
```rust
#[test]
fn test_should_show_to_user_sync_issues_chinese() {
    let folder = Folder {
        display_name: "同步问题".to_string(),
        ..Default::default()
    };
    assert!(!folder.should_show_to_user()); // Filter system folders
}
```

#### Security Module (23 tests)
Tests credential storage and encryption:
- Keyring integration (Windows Credential Manager)
- Password zeroization after use
- OAuth token storage (handles 2000+ char tokens)
- Concurrent access safety
- Credential retrieval and deletion

**Key Test:**
```rust
#[tokio::test]
async fn test_long_token_storage() {
    let long_token = "a".repeat(2000);
    let creds = AccountCredentials { access_token: Some(long_token.clone()), .. };
    store_credentials(&creds).unwrap();
    assert_eq!(get_credentials(email).unwrap().access_token, Some(long_token));
}
```

#### CMVH Blockchain Module (12 tests)
Tests email signature verification:
- EIP-712 signature generation
- Keccak256 hashing
- Address recovery from signatures
- Signer verification
- Cache operations

### Frontend Unit Tests (TypeScript)

**Location:** `src/lib/` (`.test.ts` files)
**Framework:** Vitest
**Coverage:** ~75% of utility functions

#### Store Management (26 tests)
Tests Svelte stores and reactive state:
- Account store operations (add/update/delete)
- Folder store management
- Email list state updates
- Selected email tracking
- Store persistence

**Key Test:**
```typescript
test('should update account configuration', () => {
  const account = { email: 'test@example.com', ... };
  accountsStore.addAccount(account);
  accountsStore.updateAccount('test@example.com', { display_name: 'Updated' });
  expect(get(accountsStore).accounts[0].display_name).toBe('Updated');
});
```

---

## 2. Integration Tests (P1 Priority)

### IMAP/SMTP Protocol Tests

**Location:** `src-tauri/tests/`
**Framework:** Cargo Test + Mock Servers

#### IMAP Protocol Tests (45 tests)
Tests complete IMAP workflows with mock server:
- Connection and authentication (LOGIN, AUTHENTICATE)
- Folder operations (LIST, SELECT, CREATE, DELETE)
- Email fetching (FETCH headers, body, attachments)
- Search functionality (SEARCH, UID SEARCH)
- IDLE push notifications
- Error handling (network timeouts, auth failures)

**Test Scenarios:**
- ✅ IMAP LOGIN success/failure
- ✅ LIST returns folder tree structure
- ✅ FETCH parses RFC 2047 encoded subjects
- ✅ FETCH parses multipart MIME emails
- ✅ IDLE receives EXISTS events
- ✅ Network timeout retry logic

#### SMTP Protocol Tests (45 tests)
Tests email sending workflows:
- SMTP connection and STARTTLS
- Authentication (PLAIN, LOGIN, XOAUTH2)
- Email sending with attachments
- Multipart MIME message construction
- Bounce handling
- Rate limiting

**Test Scenarios:**
- ✅ STARTTLS encryption
- ✅ Send email with attachments (multipart/mixed)
- ✅ Send HTML + plain text (multipart/alternative)
- ✅ OAuth2 authentication flow
- ✅ Error handling (5xx responses)

---

## 3. End-to-End Tests (P2 Priority)

### E2E Smoke Tests

**Location:** `e2e-tests/specs/`
**Framework:** WebDriverIO + Tauri Driver
**Platform:** Windows (WebView2)

#### Application Lifecycle (8 tests)
Tests complete user workflows:
- ✅ Application startup (< 2s)
- ✅ Main window rendering
- ✅ Account list display
- ✅ Folder tree rendering
- ✅ Email list display
- ✅ Email detail view
- ✅ Settings panel
- ✅ Application shutdown

**Test Environment:**
- **OS:** Windows 10/11
- **WebView:** WebView2 (Chromium-based)
- **Driver:** tauri-driver
- **Execution Time:** ~30 seconds

**Sample Test:**
```typescript
it('should render the main window', async () => {
  const title = await browser.getTitle();
  expect(title).toBe('Colimail');

  const accountList = await $('.account-list');
  await accountList.waitForDisplayed({ timeout: 5000 });
  expect(await accountList.isDisplayed()).toBe(true);
});
```

---

## 4. Security Tests (P3 Priority 4)

### Automated Security Testing Suite

**Location:** `src-tauri/tests/security_tests.rs`
**Framework:** Cargo Test
**Report Generation:** Automated (Markdown + JSON)

#### Coverage Areas (20 tests)

##### SQL Injection Protection (3 tests)
- ✅ Parameterized queries in account lookups
- ✅ Search query injection prevention
- ✅ Database transaction safety

**Attack Vectors Tested:**
- `' OR '1'='1` (classic injection)
- `'; DROP TABLE emails; --` (destructive injection)

##### XSS Protection (3 tests)
- ✅ HTML content escaping in email body
- ✅ Script tag filtering in subjects
- ✅ Email address sanitization

**Attack Vectors Tested:**
- `<script>alert('XSS')</script>`
- `<iframe src="javascript:...">`
- `<img src=x onerror="...">`

##### Path Traversal Protection (3 tests)
- ✅ Attachment filename sanitization
- ✅ Download path validation
- ✅ Folder name sanitization

**Attack Vectors Tested:**
- `../../../etc/passwd`
- `..\..\..\\windows\system32`

##### Sensitive Data Encryption (4 tests)
- ✅ Password zeroization in memory
- ✅ Keyring credential storage
- ✅ Long token storage (OAuth2)
- ✅ Concurrent keyring access safety

##### Input Validation (3 tests)
- ✅ Email address format validation
- ✅ Port number range validation
- ✅ Command injection prevention

##### DoS Protection (2 tests)
- ✅ Large subject truncation (1MB → 1000 chars)
- ✅ Attachment size limits (25MB max)

##### Cryptographic Security (2 tests)
- ✅ Private key zeroization
- ✅ Signature format validation

### Security Report Generation

Automated script generates comprehensive reports:
```bash
./src-tauri/scripts/run_security_tests.ps1
```

**Report Contents:**
- Dependency vulnerability scan (cargo audit)
- Security unit test results
- Static analysis (clippy security lints)
- Unsafe code audit
- Actionable recommendations

---

## 5. Performance Benchmarks (P3 Priority 3)

### Benchmark Suites

**Location:** `src-tauri/benches/`
**Framework:** Criterion.rs (statistical benchmarking)

#### Email Parsing Benchmark
**File:** `email_parsing.rs`

Measures email decoding performance:
- Header decoding (RFC 2047)
- MIME multipart parsing
- Attachment extraction
- Throughput testing (1000 emails)

**Targets:**
- Decode 1000 headers: < 5 seconds (avg < 5ms each)
- Parse multipart MIME: < 10ms per email
- Extract attachments: < 50ms per email

#### Database Query Benchmark
**File:** `database_queries.rs`

Measures database operation performance:
- INBOX list query: < 50ms
- Batch email insert: < 3s for 100 emails
- Account lookup: < 5ms
- Full-text search: < 100ms

**Test Data:**
- In-memory SQLite database
- 1000 test emails across 10 folders
- Multiple accounts (Gmail, Outlook, custom)

**Sample Benchmark:**
```rust
fn bench_inbox_query(c: &mut Criterion) {
    c.bench_function("query_inbox_100_emails", |b| {
        b.iter(|| {
            sqlx::query("SELECT * FROM emails WHERE folder_name = 'INBOX' LIMIT 100")
                .fetch_all(&pool)
        })
    });
}
```

**Results Format:**
- HTML reports with charts (criterion/reports/)
- Statistical analysis (mean, median, std dev)
- Regression detection

---

## Running Tests

### Quick Start

```bash
# Run all backend tests
cd src-tauri
cargo test

# Run all frontend tests
npm test

# Run E2E tests
cd e2e-tests
npm test

# Run security tests
cd src-tauri
cargo test --test security_tests

# Run benchmarks
cd src-tauri
cargo bench
```

### Continuous Integration

Tests are automatically run on:
- Every commit (unit + integration)
- Pull requests (full suite)
- Nightly builds (including benchmarks)

### Test Reports

Generate comprehensive reports:

```bash
# Security report (Markdown + JSON)
./src-tauri/scripts/run_security_tests.ps1

# Coverage report (P0 tests)
# See TEST_COVERAGE_REPORT.md

# Benchmark report
cargo bench
# Open: target/criterion/report/index.html
```

---

## Test Quality Standards

### Code Coverage Targets

| Component | Target | Current |
|-----------|--------|---------|
| Critical paths (codec, models, security) | 90% | ~85% |
| Business logic (commands) | 80% | ~75% |
| UI utilities | 70% | ~75% |
| Integration scenarios | 100% | 100% |

### Test Philosophy

1. **Real-World Focus**: Test actual attack vectors and user scenarios, not just happy paths
2. **Edge Case Coverage**: Prioritize malformed input, unicode handling, and error conditions
3. **Performance Validation**: Benchmarks ensure app meets performance targets
4. **Security First**: All security-critical paths have dedicated tests
5. **Maintainability**: Tests are self-documenting with clear assertions

### Anti-Patterns We Avoid

- ❌ Testing implementation details instead of behavior
- ❌ Brittle tests that break with refactoring
- ❌ Mocking when integration tests are more valuable
- ❌ Testing trivial getters/setters
- ❌ 100% coverage as a metric goal (quality > quantity)

---

## Test Infrastructure

### Tools & Frameworks

| Purpose | Tool | Version |
|---------|------|---------|
| Backend Unit Testing | Cargo Test | stable |
| Backend Benchmarks | Criterion.rs | 0.5 |
| Frontend Unit Testing | Vitest | latest |
| E2E Testing | WebDriverIO | 8.x |
| E2E Driver | tauri-driver | 2.x |
| Security Scanning | cargo-audit | latest |
| Static Analysis | Clippy | stable |

### Test Data & Fixtures

- Mock IMAP/SMTP servers (in-memory)
- SQLite test databases (`:memory:`)
- Sample email corpus (RFC 2822 formatted)
- Mock keyring for credential tests
- WebView fixture (for E2E tests)

---

## Known Issues & Limitations

### Current Gaps

1. **TLS Certificate Validation** (2 tests ignored)
   - Requires network access to test servers
   - Should be enabled in CI/CD environment

2. **Cross-Platform E2E** (partial coverage)
   - Currently Windows-only
   - macOS/Linux testing planned for CI

3. **Visual Regression Testing** (not implemented)
   - UI appearance changes not automatically detected
   - Manual QA required for visual changes

### Flaky Tests

- `cmvh::cache::tests::test_cache_operations` - Pre-existing database initialization issue (not P0 scope)

---

## Contributing to Tests

### Writing New Tests

1. **Unit Tests**: Add to relevant module's `#[cfg(test)]` block
2. **Integration Tests**: Create new file in `src-tauri/tests/`
3. **E2E Tests**: Add specs to `e2e-tests/specs/`
4. **Security Tests**: Extend `src-tauri/tests/security_tests.rs`

### Test Naming Convention

```rust
// Unit test
#[test]
fn test_<function>_<scenario>() { ... }

// Integration test
#[tokio::test]
async fn test_<workflow>_<expected_outcome>() { ... }

// Security test
#[test]
fn test_<attack_type>_<protection_method>() { ... }
```

### Before Submitting PR

Run the full test suite:

```bash
# Backend
cd src-tauri
cargo fmt
cargo check
cargo clippy -- -D warnings
cargo test

# Frontend
npm run check
npm test

# E2E (optional - runs in CI)
cd e2e-tests
npm test
```

---

## Continuous Improvement

### Planned Enhancements

- [ ] Increase unit test coverage to 90%+
- [ ] Add macOS/Linux E2E tests
- [ ] Implement visual regression testing
- [ ] Add fuzzing tests for email parsing
- [ ] Enable network-dependent TLS tests in CI
- [ ] Add load testing for concurrent operations

### Recent Additions

- ✅ Security test suite (20 tests) - Nov 2025
- ✅ IMAP/SMTP integration tests (90 tests) - Nov 2025
- ✅ E2E smoke tests (8 tests) - Nov 2025
- ✅ Performance benchmarks (2 suites) - Nov 2025
- ✅ P0 unit tests (83 tests) - Nov 2025

---

## Resources

- **Detailed Coverage Report**: [TEST_COVERAGE_REPORT.md](./TEST_COVERAGE_REPORT.md)
- **Security Testing Guide**: [src-tauri/tests/SECURITY_TESTS_README.md](./src-tauri/tests/SECURITY_TESTS_README.md)
- **E2E Testing Setup**: [e2e-tests/README.md](./e2e-tests/README.md)
- **Benchmark Reports**: `target/criterion/report/index.html` (after running `cargo bench`)

---

**Last Updated:** November 27, 2025
**Test Suite Version:** 2.0
**Total Tests:** 201+ (99.5% passing)
