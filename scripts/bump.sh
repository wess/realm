#!/usr/bin/env bash

# bump.sh - Synchronize version across the project
# Usage: ./scripts/bump.sh [VERSION]
#        If VERSION is not provided, uses the VERSION file

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the version
if [ -n "$1" ]; then
    VERSION="$1"
    echo "$VERSION" > VERSION
    echo -e "${GREEN}✓${NC} Updated VERSION file to $VERSION"
else
    if [ ! -f VERSION ]; then
        echo -e "${RED}✗${NC} VERSION file not found"
        echo "Usage: $0 [VERSION]"
        exit 1
    fi
    VERSION=$(cat VERSION)
fi

echo -e "${YELLOW}→${NC} Syncing version $VERSION across the project..."

# Update Cargo.toml
if [ -f Cargo.toml ]; then
    # Use sed to update the version line
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS sed requires -i ''
        sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    else
        # Linux sed
        sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    fi
    echo -e "${GREEN}✓${NC} Updated Cargo.toml"
else
    echo -e "${RED}✗${NC} Cargo.toml not found"
    exit 1
fi

# CLI version is now dynamically read from VERSION file at compile time via build.rs
echo -e "${GREEN}✓${NC} CLI version will be updated at compile time from VERSION file"

# Update version in man pages
for man_file in docs/man/*.1; do
    if [ -f "$man_file" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/realm [0-9]\+\.[0-9]\+\.[0-9]\+/realm $VERSION/" "$man_file"
        else
            sed -i "s/realm [0-9]\+\.[0-9]\+\.[0-9]\+/realm $VERSION/" "$man_file"
        fi
    fi
done
if [ -d "docs/man" ]; then
    echo -e "${GREEN}✓${NC} Updated man pages"
fi

# Update version in realm.yml.5
if [ -f "docs/realm.yml.5" ]; then
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/realm [0-9]\+\.[0-9]\+\.[0-9]\+/realm $VERSION/" "docs/realm.yml.5"
    else
        sed -i "s/realm [0-9]\+\.[0-9]\+\.[0-9]\+/realm $VERSION/" "docs/realm.yml.5"
    fi
    echo -e "${GREEN}✓${NC} Updated realm.yml.5"
fi

# Create/Update CHANGELOG entry (if CHANGELOG.md exists)
if [ -f CHANGELOG.md ]; then
    # Check if this version already exists in changelog
    if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
        echo -e "${YELLOW}→${NC} Add entry for v$VERSION to CHANGELOG.md manually"
    fi
fi

# Verify the changes
echo ""
echo -e "${YELLOW}→${NC} Verification:"

# Check Cargo.toml
CARGO_VERSION=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
if [ "$CARGO_VERSION" = "$VERSION" ]; then
    echo -e "${GREEN}✓${NC} Cargo.toml version: $CARGO_VERSION"
else
    echo -e "${RED}✗${NC} Cargo.toml version mismatch: $CARGO_VERSION (expected $VERSION)"
fi

# Check VERSION file
FILE_VERSION=$(cat VERSION)
if [ "$FILE_VERSION" = "$VERSION" ]; then
    echo -e "${GREEN}✓${NC} VERSION file: $FILE_VERSION"
else
    echo -e "${RED}✗${NC} VERSION file mismatch: $FILE_VERSION (expected $VERSION)"
fi

# CLI version is dynamically set from VERSION file at compile time
echo -e "${GREEN}✓${NC} CLI version: Set from VERSION file at compile time"

echo ""
echo -e "${GREEN}✓${NC} Version sync complete!"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git diff"
echo "  2. Commit the changes: git commit -am \"Bump version to $VERSION\""
echo "  3. Create a tag: git tag -a v$VERSION -m \"Release v$VERSION\""
echo "  4. Push changes: git push && git push --tags"