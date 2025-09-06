#!/usr/bin/env bash

# Realm installer script
# This script downloads and installs the latest version of Realm

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="wess/realm"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case "$ARCH" in
    x86_64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

# Map OS names
case "$OS" in
    linux)
        PLATFORM="linux"
        ;;
    darwin)
        PLATFORM="macos"
        ;;
    mingw*|msys*|cygwin*)
        PLATFORM="windows"
        echo -e "${RED}Windows detected. Please download the binary manually from:${NC}"
        echo "https://github.com/$REPO/releases"
        exit 1
        ;;
    *)
        echo -e "${RED}Unsupported operating system: $OS${NC}"
        exit 1
        ;;
esac

# Construct asset name
ASSET_NAME="realm-${PLATFORM}-${ARCH}"
if [ "$PLATFORM" = "windows" ]; then
    ASSET_NAME="${ASSET_NAME}.exe"
fi

echo -e "${BLUE}Installing Realm...${NC}"
echo -e "Platform: ${PLATFORM}-${ARCH}"
echo ""

# Get latest release URL
echo -e "${YELLOW}→${NC} Fetching latest release..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest")
if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to fetch latest release${NC}"
    exit 1
fi

# Extract version and download URL
VERSION=$(echo "$LATEST_RELEASE" | grep '"tag_name":' | sed -E 's/.*"v?([^"]+)".*/\1/')
DOWNLOAD_URL=$(echo "$LATEST_RELEASE" | grep "browser_download_url.*$ASSET_NAME\"" | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Could not find binary for ${PLATFORM}-${ARCH}${NC}"
    echo "Available releases: https://github.com/$REPO/releases"
    exit 1
fi

echo -e "${GREEN}✓${NC} Found version $VERSION"

# Create install directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Download binary
TEMP_FILE=$(mktemp)
echo -e "${YELLOW}→${NC} Downloading realm..."
curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"
if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to download realm${NC}"
    rm -f "$TEMP_FILE"
    exit 1
fi

# Make executable and move to install directory
chmod +x "$TEMP_FILE"
mv "$TEMP_FILE" "$INSTALL_DIR/realm"

echo -e "${GREEN}✓${NC} Installed realm to $INSTALL_DIR/realm"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo -e "${YELLOW}⚠${NC} $INSTALL_DIR is not in your PATH"
    echo ""
    echo "Add it to your PATH by adding this line to your shell config file:"
    echo ""
    
    # Detect shell
    SHELL_NAME=$(basename "$SHELL")
    case "$SHELL_NAME" in
        bash)
            CONFIG_FILE="$HOME/.bashrc"
            ;;
        zsh)
            CONFIG_FILE="$HOME/.zshrc"
            ;;
        fish)
            CONFIG_FILE="$HOME/.config/fish/config.fish"
            echo "  set -gx PATH $INSTALL_DIR \$PATH"
            echo ""
            echo "to $CONFIG_FILE"
            ;;
        *)
            CONFIG_FILE="your shell config file"
            ;;
    esac
    
    if [ "$SHELL_NAME" != "fish" ]; then
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        echo "to $CONFIG_FILE"
    fi
    
    echo ""
    echo "Then reload your shell or run:"
    echo "  source $CONFIG_FILE"
else
    # Verify installation
    if command -v realm &> /dev/null; then
        INSTALLED_VERSION=$(realm --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
        echo ""
        echo -e "${GREEN}✓${NC} Realm $INSTALLED_VERSION is installed and ready to use!"
        echo ""
        echo "Get started with:"
        echo "  realm init my-project --template react"
        echo "  cd my-project"
        echo "  source .venv/bin/activate"
        echo "  realm start"
    fi
fi

echo ""
echo -e "${GREEN}✓${NC} Installation complete!"
echo ""
echo "Documentation: https://github.com/$REPO"
echo "Report issues: https://github.com/$REPO/issues"