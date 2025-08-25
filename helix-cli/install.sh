#!/bin/bash

# Set your repository
REPO="HelixDB/helix-db"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_error() {
    echo -e "${RED}Error: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_info() {
    echo -e "${YELLOW}$1${NC}"
}

# Function to get version from binary safely
get_binary_version() {
    local binary_path=$1
    if [[ -f "$binary_path" && -x "$binary_path" ]]; then
        # Try to run with timeout, but don't fail if it doesn't work
        if command -v timeout >/dev/null 2>&1; then
            timeout 5s "$binary_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
        else
            "$binary_path" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
        fi
    fi
}

# Fetch the latest release version from GitHub API
print_info "Fetching latest release information..."
VERSION=$(curl --silent "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

if [[ -z "$VERSION" ]]; then
    print_error "Failed to fetch the latest version. Please check your internet connection."
    exit 1
fi

# Remove 'v' prefix if present for comparison
LATEST_VERSION=${VERSION#v}

print_info "Latest available version: $VERSION"

# Detect the operating system and architecture
OS=$(uname -s)
ARCH=$(uname -m)

print_info "Detected system: $OS $ARCH"

# Set installation directory
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Add the installation directory to PATH immediately for this session
export PATH="$INSTALL_DIR:$PATH"

# Check if binary already exists and get its version
EXISTING_BINARY="$INSTALL_DIR/helix"
if [[ -f "$EXISTING_BINARY" ]]; then
    print_info "Existing binary found at $EXISTING_BINARY"
    CURRENT_VERSION=$(get_binary_version "$EXISTING_BINARY")
    if [[ -n "$CURRENT_VERSION" ]]; then
        print_info "Current installed version: $CURRENT_VERSION"
        
        if [[ "$CURRENT_VERSION" == "$LATEST_VERSION" ]]; then
            print_success "You already have the latest version ($CURRENT_VERSION) installed!"
            print_info "To force reinstall, delete $EXISTING_BINARY and run this script again."
            exit 0
        else
            print_info "Updating from version $CURRENT_VERSION to $LATEST_VERSION"
        fi
    fi
fi

# Determine the appropriate binary to download based on OS and architecture
case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)
                FILE="helix-cli-linux-amd64"
                ;;
            aarch64|arm64)
                FILE="helix-cli-linux-arm64"
                ;;
            *)
                print_error "Unsupported Linux architecture: $ARCH"
                print_info "Supported architectures: x86_64, aarch64/arm64"
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            arm64)
                FILE="helix-cli-macos-arm64"
                ;;
            x86_64)
                FILE="helix-cli-macos-amd64"
                ;;
            *)
                print_error "Unsupported macOS architecture: $ARCH"
                print_info "Supported architectures: arm64, x86_64"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
        # Windows support
        case "$ARCH" in
            x86_64|AMD64)
                FILE="helix-cli-windows-amd64.exe"
                ;;
            *)
                print_error "Unsupported Windows architecture: $ARCH"
                print_info "Supported architectures: x86_64/AMD64"
                exit 1
                ;;
        esac
        ;;
    *)
        print_error "Unsupported operating system: $OS"
        print_info "Supported systems: Linux, macOS, Windows"
        exit 1
        ;;
esac

# Download the binary
URL="https://github.com/$REPO/releases/download/$VERSION/$FILE"
print_info "Downloading from: $URL"

# Create a temporary file for download
TEMP_BINARY=$(mktemp)
if ! curl -L "$URL" -o "$TEMP_BINARY" --fail --silent --show-error; then
    print_error "Failed to download the binary from $URL"
    print_info "Please check if the release exists and try again."
    rm -f "$TEMP_BINARY"
    exit 1
fi

# Make it executable
chmod +x "$TEMP_BINARY"

# Move to installation directory
print_info "Installing to $INSTALL_DIR/helix"
mv "$TEMP_BINARY" "$INSTALL_DIR/helix"

# Ensure PATH is set up correctly
print_info "Setting up PATH..."

# Determine shell config file
SHELL_CONFIG=""
if [[ "$SHELL" == *"bash"* ]]; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [[ "$SHELL" == *"zsh"* ]]; then
    SHELL_CONFIG="$HOME/.zshrc"
elif [[ "$SHELL" == *"fish"* ]]; then
    SHELL_CONFIG="$HOME/.config/fish/config.fish"
fi

# Add to shell config if not already present
if [[ -n "$SHELL_CONFIG" ]] && [[ -f "$SHELL_CONFIG" ]]; then
    if [[ "$SHELL" == *"fish"* ]]; then
        if ! grep -q 'set -gx PATH $HOME/.local/bin $PATH' "$SHELL_CONFIG"; then
            echo 'set -gx PATH $HOME/.local/bin $PATH' >> "$SHELL_CONFIG"
            print_info "Added $HOME/.local/bin to PATH in $SHELL_CONFIG"
        fi
    else
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_CONFIG"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_CONFIG"
            print_info "Added $HOME/.local/bin to PATH in $SHELL_CONFIG"
        fi
    fi
fi

# Final success message
print_success "Installation completed successfully!"
print_info "Helix CLI has been installed to: $INSTALL_DIR/helix"

# Try to verify version (but don't fail if it doesn't work)
INSTALLED_VERSION=$(get_binary_version "$INSTALL_DIR/helix")
if [[ -n "$INSTALLED_VERSION" ]]; then
    print_success "Installed version: $INSTALLED_VERSION"
fi

print_info ""
print_info "Next steps:"
print_info "1. Restart your terminal or run: source $SHELL_CONFIG"
print_info "2. Run 'helix --version' to verify the installation"
print_info ""
print_info "If you encounter issues:"
print_info "- On Linux: You may need a newer glibc version. Try updating your system."
print_info "- On macOS: You may need to allow the binary in System Preferences > Security & Privacy"
print_info "- On Windows: Run this script in Git Bash or WSL"
print_info ""
print_info "Metrics are enabled by default. To disable them, run 'helix metrics --off'"

# Exit successfully even if we can't verify the binary runs
# This prevents the script from trying to build from source
exit 0