# Release Checklist

Complete guide for releasing a new version of Colimail to the public.

## Pre-Release Preparation

### 1. Code Quality & Testing

- [ ] All tests pass locally
- [ ] Run frontend type check: `npm run check`
- [ ] Run Rust checks: `cd src-tauri && cargo fmt && cargo check`
- [ ] No compiler warnings or errors
- [ ] Manual testing on all target platforms:
  - [ ] Windows 10/11
  - [ ] macOS (Intel & Apple Silicon)
  - [ ] Ubuntu/Debian Linux
- [ ] Test all core features:
  - [ ] Add/remove email accounts
  - [ ] Send/receive emails
  - [ ] OAuth2 authentication (Gmail, Outlook)
  - [ ] IMAP/SMTP authentication
  - [ ] Attachment handling
  - [ ] Folder sync
  - [ ] Search functionality

### 2. Documentation

- [ ] Update `README.md` with new features
- [ ] Update `CHANGELOG.md` with version changes
- [ ] Review and update `OAUTH2_SETUP.md` if OAuth flow changed
- [ ] Check all documentation links work
- [ ] Update screenshots if UI changed
- [ ] Verify installation instructions are current

### 3. Version Updates

- [ ] Update version in `package.json`
- [ ] Update version in `src-tauri/Cargo.toml`
- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Ensure all three versions match (e.g., `0.1.0` → `0.2.0`)
- [ ] Update copyright year if needed

### 4. Security Review

- [ ] No hardcoded credentials or secrets
- [ ] No debug logging of sensitive data
- [ ] OAuth2 client secrets not committed (use environment variables)
- [ ] Run security audit: `cargo audit` and `npm audit`
- [ ] Review `SECURITY.md` for accuracy

### 5. Build Configuration

- [ ] Verify `tauri.conf.json` settings:
  - [ ] Product name
  - [ ] Bundle identifier
  - [ ] Icons present (32x32, 128x128, icns, ico)
  - [ ] Windows signing (if applicable)
  - [ ] macOS signing (if applicable)
- [ ] Test production build locally:
  ```bash
  npm run tauri build
  ```

## Release Process

### Method 1: Automated Release (GitHub Actions)

#### Step 1: Create Version Tag

```bash
# Ensure you're on main branch
git checkout main
git pull origin main

# Create and push version tag
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

#### Step 2: Monitor GitHub Actions

1. Go to: `https://github.com/YOUR_USERNAME/colimail/actions`
2. Watch the "Release Build" workflow
3. Verify builds complete for all platforms:
   - Windows (x86_64)
   - macOS (Intel x86_64)
   - macOS (Apple Silicon aarch64)
   - Linux (x86_64)

#### Step 3: Review Draft Release

1. Navigate to: `https://github.com/YOUR_USERNAME/colimail/releases`
2. Find the draft release created by GitHub Actions
3. Edit release notes:
   - Add highlights and breaking changes
   - Link to issues/PRs
   - Add upgrade instructions if needed
4. Verify all artifacts uploaded:
   - `Colimail_0.2.0_x64_en-US.msi` (Windows)
   - `Colimail_0.2.0_x64.dmg` (macOS Intel)
   - `Colimail_0.2.0_aarch64.dmg` (macOS Apple Silicon)
   - `colimail_0.2.0_amd64.deb` (Linux)
   - `colimail_0.2.0_amd64.AppImage` (Linux)

#### Step 4: Publish Release

- [ ] Download and test each artifact on respective platform
- [ ] Verify installer works and app launches
- [ ] Click "Publish release" on GitHub

### Method 2: Manual Release

If not using GitHub Actions:

#### Step 1: Build for Each Platform

**Windows**:
```bash
npm install
npm run tauri build
# Output: src-tauri/target/release/bundle/msi/
```

**macOS**:
```bash
npm install
npm run tauri build
# For universal binary (both Intel & Apple Silicon):
npm run tauri build -- --target universal-apple-darwin
# Output: src-tauri/target/release/bundle/dmg/
```

**Linux (Ubuntu/Debian)**:
```bash
npm install
npm run tauri build
# Output: src-tauri/target/release/bundle/deb/ and bundle/appimage/
```

#### Step 2: Create GitHub Release

1. Go to: `https://github.com/YOUR_USERNAME/colimail/releases/new`
2. Choose tag: `v0.2.0` (create new tag)
3. Release title: `Colimail v0.2.0`
4. Add release notes (see template below)
5. Upload all build artifacts
6. Publish release

## Post-Release Tasks

### 1. Announcements

- [ ] Update project website (if applicable)
- [ ] Post on social media (Twitter, Reddit, etc.)
- [ ] Notify beta testers via email/Discord
- [ ] Update any third-party listings (if applicable)

### 2. Documentation Sites

- [ ] Update download links in README
- [ ] Update version badge in README
- [ ] Push documentation changes

### 3. Monitoring

- [ ] Monitor GitHub Issues for bug reports
- [ ] Check download statistics
- [ ] Review user feedback
- [ ] Prepare hotfix if critical bugs found

### 4. Version Bump (Post-Release)

```bash
# After release, bump to next dev version
# e.g., 0.2.0 → 0.3.0-dev

# Update package.json
# Update Cargo.toml
# Update tauri.conf.json

git add .
git commit -m "chore: bump version to 0.3.0-dev"
git push origin main
```

## Release Notes Template

```markdown
## Colimail v0.2.0

Released: 2025-01-24

### 🎉 Highlights

- Brief description of major features

### ✨ New Features

- Feature 1: Description (#123)
- Feature 2: Description (#124)

### 🐛 Bug Fixes

- Fix 1: Description (#125)
- Fix 2: Description (#126)

### 🔧 Improvements

- Improvement 1: Description
- Improvement 2: Description

### ⚠️ Breaking Changes

- Breaking change description and migration guide

### 📦 Installation

**Windows**:
Download `Colimail_0.2.0_x64_en-US.msi` and run the installer.

**macOS**:
- Apple Silicon: `Colimail_0.2.0_aarch64.dmg`
- Intel: `Colimail_0.2.0_x64.dmg`

**Linux**:
- Debian/Ubuntu: `colimail_0.2.0_amd64.deb`
- Universal: `colimail_0.2.0_amd64.AppImage`

### 📝 Full Changelog

See [CHANGELOG.md](https://github.com/YOUR_USERNAME/colimail/blob/main/CHANGELOG.md)

### 🙏 Contributors

Thanks to all contributors for this release!
```

## Emergency Rollback

If critical issues are discovered post-release:

1. **Mark release as pre-release** on GitHub
2. **Add warning notice** to release description
3. **Prepare hotfix**:
   ```bash
   git checkout -b hotfix/v0.2.1
   # Make fixes
   git commit -m "hotfix: critical bug fix"
   git tag -a v0.2.1 -m "Hotfix release 0.2.1"
   git push origin v0.2.1
   ```
4. **Release v0.2.1** following standard process

## Versioning Strategy

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.2.0): New features, backwards compatible
- **PATCH** (0.2.1): Bug fixes, backwards compatible

### Pre-1.0 Releases

For v0.x.x releases:
- Minor version bumps may include breaking changes
- Clearly document breaking changes in release notes
- Provide migration guides

## Checklist Summary

Quick checklist for release day:

```
[ ] All tests pass
[ ] Documentation updated
[ ] Version bumped in all files
[ ] Changelog updated
[ ] Security audit clean
[ ] Local build successful
[ ] Tag created and pushed
[ ] GitHub Actions builds complete
[ ] Artifacts tested on target platforms
[ ] Release notes written
[ ] Release published
[ ] Announcements posted
```

## Questions?

If unsure about any step, consult:
- Project maintainers
- `CONTRIBUTING.md`
- GitHub Discussions

---

**Template Version**: 1.0
**Last Updated**: 2025-01-24
