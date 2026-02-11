# Release

Create a new release. Analyzes git history and determines version according to semver rules.

## Steps

### Step 1: Check Current State

Run the following commands **simultaneously**:

```bash
# Check current version
grep -m1 'version' src-tauri/Cargo.toml

# Check commits since last tag
git log $(git describe --tags --abbrev=0 2>/dev/null || echo "HEAD~20")..HEAD --oneline

# Check working directory status
git status --short
```

### Step 2: Run CI Checks

Run all CI checks before proceeding. **All must pass** — if any fail, fix the issue and re-run before continuing.

```bash
# Rust format check
cd src-tauri && cargo fmt --check

# Rust lint check
cd src-tauri && cargo clippy --all-targets -- -D warnings

# Rust tests
cd src-tauri && cargo test

# TypeScript type check
pnpm vue-tsc --noEmit
```

If any check fails, stop and report the error. Do not proceed to the next step.

### Step 3: Determine Version

Analyze commit history and determine new version according to semver:

- **MAJOR (x.0.0)**: breaking changes, incompatible API changes
- **MINOR (0.x.0)**: new features (`feat:`)
- **PATCH (0.0.x)**: bug fixes, refactoring, docs (`fix:`, `chore:`, `docs:`, `refactor:`)

Bump version based on the highest change type found in commits.

### Step 4: Update Version Files

Update version in these 3 files:

1. `src-tauri/Cargo.toml` - line 3: `version = "x.x.x"`
2. `src-tauri/tauri.conf.json` - line 4: `"version": "x.x.x"`
3. `package.json` - line 4: `"version": "x.x.x"`

### Step 5: Commit and Tag

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json
git commit -m "chore: release v{new_version}"
git tag v{new_version}
```

### Step 6: Push

```bash
git push && git push origin v{new_version}
```

## Notes

- If there are uncommitted changes, ask user how to handle them first.
- Confirm version with user before proceeding.
- For first release (no tags), use current version as baseline.
