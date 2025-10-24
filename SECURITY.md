# Security Policy

## Reporting Security Issues

We take security seriously. If you discover a security vulnerability in Colimail, please report it responsibly.

### How to Report

**DO NOT** create a public GitHub issue for security vulnerabilities.

Instead, please email security details to:
- **Email**: [colimail.colibri@gmail.com] (⚠️ Replace with actual security contact)
- **Subject**: "Colimail Security Vulnerability Report"

Include the following information:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (optional)

We will acknowledge receipt within 48 hours and aim to provide a fix within 14 days for critical issues.

## Security Considerations

### Current Security Status

Colimail is in **beta/testing phase**. The following security considerations apply:

#### ⚠️ Known Security Limitations

1. **Password Storage**
   - **Status**: Passwords are currently stored in **plaintext** in the SQLite database
   - **Risk**: If an attacker gains access to the database file, they can read passwords
   - **Planned**: Encryption using platform-specific secure storage (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
   - **Recommendation**: Use OAuth2 authentication when possible (Gmail, Outlook)

2. **OAuth2 Token Storage**
   - **Status**: OAuth2 tokens stored in database without additional encryption
   - **Risk**: Token exposure if database is compromised
   - **Planned**: Secure token storage using platform keychains

3. **Database Location**
   - **Windows**: `%APPDATA%\com.jiong.colimail\`
   - **macOS**: `~/Library/Application Support/com.jiong.colimail/`
   - **Linux**: `~/.local/share/com.jiong.colimail/`
   - **Risk**: Accessible to any process running under the user account
   - **Mitigation**: Ensure your system has proper user permissions and full-disk encryption

### Security Best Practices for Users

1. **Use OAuth2 When Available**
   - Prefer OAuth2 (Google, Microsoft) over password-based authentication
   - OAuth2 tokens can be revoked without changing your email password
   - Provides better audit trail and access control

2. **Enable Disk Encryption**
   - Windows: Use BitLocker
   - macOS: Use FileVault
   - Linux: Use LUKS/dm-crypt
   - This protects the database file if your device is lost or stolen

3. **Keep Software Updated**
   - Update Colimail regularly to receive security patches
   - Keep your operating system and dependencies updated

4. **Use Strong System Passwords**
   - Protect your user account with a strong password
   - Enable screen lock when away from computer

5. **Be Cautious with Attachments**
   - Don't open attachments from unknown senders
   - Colimail does not scan for malware - use antivirus software

## Data Privacy

### What Data is Collected

Colimail does **NOT** collect or transmit any user data:
- ✅ All emails stored locally on your device
- ✅ No telemetry or analytics
- ✅ No crash reporting to external servers
- ✅ No tracking or advertising
- ✅ Open source - you can verify the code

### Data Stored Locally

The following data is stored in the local SQLite database:
- Email account credentials (IMAP/SMTP servers, username, password)
- OAuth2 tokens (if using OAuth2 authentication)
- Email messages (headers and bodies)
- Email attachments
- Application settings

### Third-Party Services

Colimail connects to:
1. **Your Email Provider** (Gmail, Outlook, etc.)
   - IMAP connection to fetch emails
   - SMTP connection to send emails
   - Subject to your provider's privacy policy

2. **OAuth2 Providers** (if using OAuth2)
   - Google OAuth2 API (for Gmail)
   - Microsoft OAuth2 API (for Outlook)
   - Only for authentication purposes

## Network Security

### Connections

- **IMAP**: Uses TLS/SSL (port 993 for secure IMAP)
- **SMTP**: Uses STARTTLS (port 587) or TLS/SSL (port 465)
- **OAuth2**: HTTPS connections to provider APIs

### Certificate Validation

- Uses system certificate store for TLS verification
- Validates server certificates using `native-tls` library
- Rejects invalid or self-signed certificates (can't be disabled)

## Dependency Security

We monitor dependencies for known vulnerabilities:
- **Rust crates**: Regularly updated, security advisories checked
- **npm packages**: Regular `npm audit` runs
- **Tauri framework**: Following Tauri security best practices

## Roadmap: Planned Security Improvements

### Short-term (v0.2.x)
- [ ] Encrypt passwords using platform keychain
- [ ] Secure OAuth2 token storage
- [ ] Add password strength indicator
- [ ] Implement session timeout

### Medium-term (v0.3.x)
- [ ] Two-factor authentication support
- [ ] Master password for database encryption
- [ ] Automatic security updates
- [ ] Security audit log

### Long-term (v1.0+)
- [ ] PGP/GPG email encryption
- [ ] S/MIME support
- [ ] End-to-end encryption for local storage
- [ ] Security hardening (sandboxing, code signing)

## Security Recommendations for Developers

If you're contributing to Colimail:

1. **Never log sensitive data** (passwords, tokens, email content)
2. **Sanitize user input** to prevent injection attacks
3. **Use parameterized queries** for database operations
4. **Validate all external data** (email headers, attachments)
5. **Follow Rust security guidelines** (avoid `unsafe`, handle errors properly)
6. **Keep dependencies updated** (`cargo update`, `npm update`)
7. **Run security checks** (`cargo audit`, `npm audit`)

## Responsible Disclosure Timeline

1. **T+0**: Vulnerability reported to security contact
2. **T+48h**: Acknowledgment sent to reporter
3. **T+7d**: Initial assessment and severity classification
4. **T+14d**: Fix developed and tested (for critical issues)
5. **T+30d**: Patch released and public disclosure (coordinated with reporter)

## Security Contact

For security-related questions or concerns:
- **Email**: [colimail.colibri@gmail.com] (⚠️ Replace with actual contact)
- **PGP Key**: [Optional: Add PGP public key for encrypted communication]

## Credits

We thank the following security researchers for responsible disclosure:
- [No reports yet]

---

**Last Updated**: 2025-01-24
**Version**: 0.1.0
