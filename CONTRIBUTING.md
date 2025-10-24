# Contributing to Colimail

Thank you for your interest in contributing to Colimail! We welcome contributions from the community to help make this project better.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)

## Code of Conduct

By participating in this project, you agree to:
- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

Before contributing, ensure you have:
- **Rust** 1.70+ ([Install](https://rustup.rs/))
- **Node.js** 18+ ([Install](https://nodejs.org/))
- **Git** for version control
- Platform-specific build tools (see README)

### Setup Development Environment

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/colimail.git
cd colimail

# Add upstream remote
git remote add upstream https://github.com/daodreamer/colimail.git

# Install dependencies
npm install

# Run development server
npm run tauri dev
```

## Development Workflow

### 1. Create a Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name
# or for bug fixes
git checkout -b fix/bug-description
```

### 2. Make Changes

- Write clear, concise code
- Follow existing code style
- Add tests if applicable
- Update documentation as needed

### 3. Test Your Changes

```bash
# Frontend type checking
npm run check

# Rust formatting and checks
cd src-tauri
cargo fmt
cargo check

# Run the application
npm run tauri dev
```

### 4. Commit Changes

Use clear, descriptive commit messages:

```bash
git add .
git commit -m "feat: add email search functionality"
# or
git commit -m "fix: resolve IMAP connection timeout issue"
```

**Commit message format**:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub:
1. Go to your fork on GitHub
2. Click "Compare & pull request"
3. Fill in the PR template with details
4. Submit the PR

## Coding Standards

### Rust Code

- **Format code** with `cargo fmt` before committing
- **Run checks** with `cargo check` to ensure no compilation errors
- **Use meaningful names** for variables and functions
- **Add comments** for complex logic
- **Handle errors** properly (no unwrap in production code)
- **Write tests** for new functionality

Example:
```rust
/// Fetches email headers from IMAP server
///
/// # Arguments
/// * `account_id` - The database ID of the account
/// * `folder` - IMAP folder name (e.g., "INBOX")
///
/// # Returns
/// Result containing vector of EmailHeader or error string
#[tauri::command]
async fn fetch_emails(account_id: i64, folder: String) -> Result<Vec<EmailHeader>, String> {
    // Implementation
}
```

### TypeScript/Svelte Code

- **Run type checking** with `npm run check`
- **Use TypeScript** for type safety
- **Follow Svelte 5 runes API** (`$state`, `$derived`, `$effect`)
- **Keep components small** and focused
- **Extract reusable logic** into separate files

Example:
```typescript
import { invoke } from '@tauri-apps/api/core';
import type { EmailHeader } from '$lib/types';

async function fetchEmails(accountId: number, folder: string): Promise<EmailHeader[]> {
    return await invoke<EmailHeader[]>('fetch_emails', { accountId, folder });
}
```

### All Code

- **All code and comments must be in English** (per project conventions)
- **Keep lines under 100 characters** when possible
- **Use consistent indentation** (tabs for Rust, 2 spaces for TS/Svelte)
- **Avoid hard-coded values** - use constants or configuration

## Submitting Changes

### Pull Request Guidelines

- **One feature/fix per PR** - keep changes focused
- **Update documentation** if your changes affect user-facing features
- **Add tests** for new functionality
- **Ensure all checks pass** (formatting, type checking, builds)
- **Write a clear PR description**:
  - What changes were made
  - Why these changes were necessary
  - How to test the changes
  - Related issue numbers (if any)

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Testing
How to test these changes:
1. Step one
2. Step two

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings generated
- [ ] Tests added (if applicable)
```

### Review Process

1. Maintainers will review your PR
2. Address any requested changes
3. Once approved, your PR will be merged
4. Your contribution will be included in the next release

## Reporting Bugs

### Before Submitting a Bug Report

- **Check existing issues** to avoid duplicates
- **Test with the latest version** to see if the bug is already fixed
- **Gather information**:
  - OS and version
  - Colimail version
  - Steps to reproduce
  - Expected vs actual behavior
  - Error messages or logs

### Bug Report Template

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Go to '...'
2. Click on '...'
3. See error

## Expected Behavior
What you expected to happen

## Actual Behavior
What actually happened

## Environment
- OS: [e.g., Windows 11, macOS 14.2]
- Colimail Version: [e.g., 0.1.0]
- Email Provider: [e.g., Gmail, Outlook]

## Additional Context
Screenshots, error logs, etc.
```

## Feature Requests

We welcome feature suggestions! When requesting a feature:

1. **Check existing issues** to see if already requested
2. **Describe the problem** your feature would solve
3. **Propose a solution** or implementation approach
4. **Consider alternatives** and trade-offs
5. **Provide use cases** - how would this benefit users?

### Feature Request Template

```markdown
## Problem Statement
What problem does this feature solve?

## Proposed Solution
How should this feature work?

## Alternatives Considered
Other approaches you've thought about

## Use Cases
Who would benefit from this feature and how?
```

## Areas for Contribution

Looking for where to start? Here are some areas that need help:

### High Priority
- Password encryption (currently stored in plaintext)
- OAuth2 token secure storage
- Email search functionality
- Performance optimizations
- Cross-platform testing

### Good First Issues
- UI/UX improvements
- Documentation enhancements
- Additional email provider configurations
- Keyboard shortcuts
- Accessibility improvements

### Advanced
- Plugin/extension system
- PGP encryption support
- Calendar integration
- Custom themes
- Multi-language support

## Questions?

If you have questions about contributing:
- Open a GitHub Discussion
- Comment on relevant issues
- Reach out to maintainers

Thank you for contributing to Colimail! Your efforts help make email better for everyone.
