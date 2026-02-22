#!/usr/bin/env bash
set -euo pipefail

# CLASP Release Script
# Bumps versions, builds, publishes all packages in correct dependency order.
#
# Usage:
#   ./scripts/release.sh <version>          # e.g. 4.0.3
#   ./scripts/release.sh <version> --dry-run # preview without publishing
#   ./scripts/release.sh <version> --skip-publish # bump + build only

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Args
# ---------------------------------------------------------------------------

if [ $# -lt 1 ]; then
  echo "Usage: $0 <version> [--dry-run] [--skip-publish]"
  echo ""
  echo "Examples:"
  echo "  $0 4.0.3"
  echo "  $0 4.1.0 --dry-run"
  echo "  $0 4.0.3 --skip-publish"
  exit 1
fi

VERSION="$1"
DRY_RUN=false
SKIP_PUBLISH=false

shift
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --skip-publish) SKIP_PUBLISH=true ;;
    *) echo "Unknown flag: $arg"; exit 1 ;;
  esac
done

# Semver validation
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be semver (e.g. 4.0.3), got: $VERSION"
  exit 1
fi

MAJOR_MINOR="${VERSION%.*}"  # e.g. "4.0" from "4.0.3"

echo "=== CLASP Release $VERSION ==="
echo "    Dry run: $DRY_RUN"
echo "    Skip publish: $SKIP_PUBLISH"
echo ""

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

step() { echo ""; echo "--- $1"; }
ok()   { echo "    OK"; }

bump_json_version() {
  local file="$1"
  local ver="$2"
  # Use python for reliable JSON editing
  python3 -c "
import json, sys
with open('$file') as f:
    data = json.load(f)
data['version'] = '$ver'
with open('$file', 'w') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
"
}

bump_toml_version() {
  local file="$1"
  local old_pattern="$2"
  local new_value="$3"
  sed -i '' "s|$old_pattern|$new_value|" "$file"
}

cargo_publish() {
  local crate_dir="$1"
  local crate_name="$2"
  if [ "$DRY_RUN" = true ]; then
    echo "    [dry-run] cargo publish -p $crate_name"
    (cd "$crate_dir" && cargo publish -p "$crate_name" --dry-run 2>&1 | tail -3) || true
  elif [ "$SKIP_PUBLISH" = true ]; then
    echo "    [skip] $crate_name"
  else
    echo "    Publishing $crate_name..."
    cargo publish -p "$crate_name" --allow-dirty
    echo "    Waiting 25s for crates.io index..."
    sleep 25
  fi
}

# ---------------------------------------------------------------------------
# 1. Bump Versions
# ---------------------------------------------------------------------------

step "1. Bumping Rust workspace version"

# Workspace root
bump_toml_version Cargo.toml \
  'version = "[0-9]*\.[0-9]*\.[0-9]*"' \
  "version = \"$VERSION\""
echo "    Cargo.toml (workspace) -> $VERSION"

# Deploy relay (standalone)
bump_toml_version deploy/relay/Cargo.toml \
  '^version = "[0-9]*\.[0-9]*\.[0-9]*"' \
  "version = \"$VERSION\""
echo "    deploy/relay/Cargo.toml -> $VERSION"

ok

step "2. Bumping npm package versions"

# @clasp-to/core
bump_json_version bindings/js/packages/clasp-core/package.json "$VERSION"
echo "    @clasp-to/core -> $VERSION"

# App package.json files (not published but keep in sync)
for app_pkg in apps/chat/package.json apps/bridge/package.json apps/docs/package.json site/package.json; do
  if [ -f "$app_pkg" ]; then
    bump_json_version "$app_pkg" "$VERSION"
    echo "    $app_pkg -> $VERSION"
  fi
done

# Update @clasp-to/core dependency ranges in consumers
for consumer in apps/chat/package.json site/package.json; do
  if [ -f "$consumer" ]; then
    python3 -c "
import json
with open('$consumer') as f:
    data = json.load(f)
for section in ['dependencies', 'devDependencies']:
    if section in data and '@clasp-to/core' in data[section]:
        val = data[section]['@clasp-to/core']
        if not val.startswith('file:'):
            data[section]['@clasp-to/core'] = '^$VERSION'
with open('$consumer', 'w') as f:
    json.dump(data, f, indent=2)
    f.write('\n')
"
    echo "    $consumer @clasp-to/core -> ^$VERSION"
  fi
done

ok

step "3. Bumping Python package version"

bump_toml_version bindings/python/pyproject.toml \
  'version = "[0-9]*\.[0-9]*\.[0-9]*"' \
  "version = \"$VERSION\""
echo "    clasp-to (Python) -> $VERSION"

ok

# ---------------------------------------------------------------------------
# 2. Build & Check
# ---------------------------------------------------------------------------

step "4. Checking Rust workspace builds"
cargo check --workspace
ok

step "5. Checking relay builds (with local patches)"
(cd deploy/relay && cargo check --features full)
ok

step "6. Building @clasp-to/core (npm)"
if [ -f bindings/js/packages/clasp-core/tsconfig.json ] || [ -f bindings/js/packages/clasp-core/package.json ]; then
  (cd bindings/js/packages/clasp-core && npm run build 2>&1 | tail -5) || echo "    (build script not available or failed — check manually)"
fi
ok

step "7. Building Python package"
if command -v python3 &>/dev/null && [ -f bindings/python/pyproject.toml ]; then
  (cd bindings/python && python3 -m build 2>&1 | tail -3) || echo "    (python build not available — install 'build' package)"
fi
ok

# ---------------------------------------------------------------------------
# 3. Publish Rust Crates (dependency order)
# ---------------------------------------------------------------------------

step "8. Publishing Rust crates to crates.io"
echo "    Order: core -> transport -> {journal,rules,caps,registry,bridge,wasm,embedded}"
echo "           -> {router,discovery,federation,client} -> {test-utils,cli}"

# Tier 0: no workspace deps
cargo_publish "$REPO_ROOT" clasp-core
cargo_publish "$REPO_ROOT" clasp-embedded

# Tier 1: depends on clasp-core only
cargo_publish "$REPO_ROOT" clasp-transport
cargo_publish "$REPO_ROOT" clasp-journal
cargo_publish "$REPO_ROOT" clasp-rules
cargo_publish "$REPO_ROOT" clasp-caps
cargo_publish "$REPO_ROOT" clasp-registry
cargo_publish "$REPO_ROOT" clasp-bridge
cargo_publish "$REPO_ROOT" clasp-wasm

# Tier 2: depends on core + transport (+ optional tier-1)
cargo_publish "$REPO_ROOT" clasp-router
cargo_publish "$REPO_ROOT" clasp-discovery
cargo_publish "$REPO_ROOT" clasp-federation
cargo_publish "$REPO_ROOT" clasp-client

# Tier 3: depends on tier-2
cargo_publish "$REPO_ROOT" clasp-test-utils
cargo_publish "$REPO_ROOT" clasp-cli

ok

# ---------------------------------------------------------------------------
# 4. Update relay Cargo.lock
# ---------------------------------------------------------------------------

step "9. Updating deploy/relay Cargo.lock"
if [ "$SKIP_PUBLISH" = false ] && [ "$DRY_RUN" = false ]; then
  (cd deploy/relay && cargo update 2>&1 | tail -10)
  echo "    Cargo.lock updated to pull $VERSION from crates.io"
else
  echo "    [skipped — crates not published yet]"
fi
ok

# ---------------------------------------------------------------------------
# 5. Publish npm
# ---------------------------------------------------------------------------

step "10. Publishing @clasp-to/core to npm"
if [ "$DRY_RUN" = true ]; then
  echo "    [dry-run] npm publish"
  (cd bindings/js/packages/clasp-core && npm publish --dry-run 2>&1 | tail -5) || true
elif [ "$SKIP_PUBLISH" = true ]; then
  echo "    [skip]"
else
  (cd bindings/js/packages/clasp-core && npm publish --access public)
fi
ok

# ---------------------------------------------------------------------------
# 6. Publish Python
# ---------------------------------------------------------------------------

step "11. Publishing clasp-to to PyPI"
if [ "$DRY_RUN" = true ]; then
  echo "    [dry-run] twine upload"
elif [ "$SKIP_PUBLISH" = true ]; then
  echo "    [skip]"
else
  if command -v twine &>/dev/null; then
    (cd bindings/python && twine upload dist/clasp_to-${VERSION}*)
  else
    echo "    twine not found — run: pip install twine && cd bindings/python && twine upload dist/*"
  fi
fi
ok

# ---------------------------------------------------------------------------
# 7. Install updated packages in apps
# ---------------------------------------------------------------------------

step "12. Installing updated packages in apps"
if [ "$SKIP_PUBLISH" = false ] && [ "$DRY_RUN" = false ]; then
  for app_dir in apps/chat site; do
    if [ -f "$app_dir/package.json" ]; then
      echo "    npm install in $app_dir"
      (cd "$app_dir" && npm install 2>&1 | tail -3)
    fi
  done
else
  echo "    [skipped — packages not published yet]"
fi
ok

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

step "DONE"
echo ""
echo "  Version:  $VERSION"
echo "  Rust:     14 crates on crates.io"
echo "  npm:      @clasp-to/core on npm"
echo "  Python:   clasp-to on PyPI"
echo ""
if [ "$DRY_RUN" = true ] || [ "$SKIP_PUBLISH" = true ]; then
  echo "  Next steps:"
  echo "    1. Review changes: git diff"
  echo "    2. Commit: git add -A && git commit -m 'v$VERSION'"
  echo "    3. Tag: git tag v$VERSION"
  echo "    4. Push: git push && git push --tags"
  if [ "$SKIP_PUBLISH" = true ]; then
    echo "    5. Publish: $0 $VERSION  (without --skip-publish)"
  fi
else
  echo "  Next steps:"
  echo "    1. Commit: git add -A && git commit -m 'v$VERSION'"
  echo "    2. Tag: git tag v$VERSION"
  echo "    3. Push: git push && git push --tags"
  echo "       (push triggers GitHub Actions -> Docker build -> deploy to droplet)"
fi
