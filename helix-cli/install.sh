#!/bin/bash

# Set your repository
REPO="HelixDB/helix-db"

# Function to run command with timeout
run_with_timeout() {
    local timeout_duration=$1
    shift
    timeout "$timeout_duration" "$@"
}

# Function to get version from binary safely
get_binary_version() {
    local binary_path=$1
    if [[ -f "$binary_path" && -x "$binary_path" ]]; then
        local version_output
        version_output=$(run_with_timeout 10s "$binary_path" --version 2>/dev/null)
        if [[ $? -eq 0 && -n "$version_output" ]]; then
            echo "$version_output" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1
        fi
    fi
}

# Fetch the latest release version from GitHub API
VERSION=$(curl --silent "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

if [[ -z "$VERSION" ]]; then
    echo "Failed to fetch the latest version. Please check your internet connection or the repository."
    exit 1
fi

# Remove 'v' prefix if present for comparison
LATEST_VERSION=${VERSION#v}

echo "Latest available version: $VERSION"
echo "User home directory: $HOME"

# Detect the operating system
OS=$(uname -s)
ARCH=$(uname -m)

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# Add the installation directory to PATH immediately for this session
export PATH="$INSTALL_DIR:$PATH"

# Check if binary already exists and get its version
EXISTING_BINARY="$INSTALL_DIR/helix"
CURRENT_VERSION=""
if [[ -f "$EXISTING_BINARY" ]]; then
    echo "Existing binary found at $EXISTING_BINARY"
    CURRENT_VERSION=$(get_binary_version "$EXISTING_BINARY")
    if [[ -n "$CURRENT_VERSION" ]]; then
        echo "Current installed version: $CURRENT_VERSION"
        
        # Compare versions (simple string comparison should work for semver)
        if [[ "$CURRENT_VERSION" == "$LATEST_VERSION" ]]; then
            echo "You already have the latest version ($CURRENT_VERSION) installed."
            echo "To force reinstall, delete $EXISTING_BINARY and run this script again."
            
            # Still verify it works
            echo "Verifying current installation..."
            if run_with_timeout 10s helix --version >/dev/null 2>&1; then
                echo "Helix CLI is working correctly!"
                exit 0
            else
                echo "Current installation appears to be broken. Proceeding with reinstall..."
            fi
        else
            echo "Updating from version $CURRENT_VERSION to $LATEST_VERSION"
        fi
    else
        echo "Existing binary found but version check failed. Proceeding with reinstall..."
    fi
fi

# Ensure that $HOME/.local/bin is in the PATH permanently
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "Adding $HOME/.local/bin to PATH permanently"

    # Determine shell config file
    if [[ "$SHELL" == *"bash"* ]]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_CONFIG="$HOME/.zshrc"
    fi

    # Add to shell config if not already present
    if [[ -f "$SHELL_CONFIG" ]]; then
        if ! grep -q 'export PATH="$HOME/.local/bin:$PATH"' "$SHELL_CONFIG"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_CONFIG"
        fi
    fi
fi

# Determine the appropriate binary to download
if [[ "$OS" == "Linux" && "$ARCH" == "x86_64" ]]; then
    FILE="helix-cli-linux-amd64"
elif [[ "$OS" == "Linux" && "$ARCH" == "aarch64" ]]; then
    FILE="helix-cli-linux-arm64"
elif [[ "$OS" == "Darwin" && "$ARCH" == "arm64" ]]; then
    FILE="helix-cli-macos-arm64"
elif [[ "$OS" == "Darwin" && "$ARCH" == "x86_64" ]]; then
    FILE="helix-cli-macos-amd64"
else
    echo "Unsupported system: This installer only works on Linux AMD64 and macOS ARM64"
    echo "Your system is: $OS $ARCH"
    exit 1
fi

# Download the binary
URL="https://github.com/$REPO/releases/download/$VERSION/$FILE"
echo "Downloading from $URL"

# Create a temporary file for download
TEMP_BINARY=$(mktemp)
curl -L "$URL" -o "$TEMP_BINARY"
if [[ $? -ne 0 ]]; then
    echo "Failed to download the binary"
    rm -f "$TEMP_BINARY"
    exit 1
fi

# Make it executable
chmod +x "$TEMP_BINARY"

# Test the downloaded binary with timeout
echo "Testing downloaded binary..."
if run_with_timeout 10s "$TEMP_BINARY" --version &> /dev/null; then
    echo "Downloaded binary is working. Installing..."
    mv "$TEMP_BINARY" "$INSTALL_DIR/helix"
else
    echo "Downloaded binary is incompatible with system or has issues. Falling back to building from source..."
    rm -f "$TEMP_BINARY"

    # Ensure Rust is installed
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust first..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Clone and build from source
    TMP_DIR=$(mktemp -d)
    echo "Building from source in $TMP_DIR..."
    git clone "https://github.com/$REPO.git" "$TMP_DIR"
    if [[ $? -ne 0 ]]; then
        echo "Failed to clone repository"
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    cd "$TMP_DIR"
    git checkout "$VERSION"
    if [[ $? -ne 0 ]]; then
        echo "Failed to checkout version $VERSION"
        cd - > /dev/null
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    echo "Building helix-cli (this may take a while)..."
    cargo build --release --bin helix
    if [[ $? -ne 0 ]]; then
        echo "Failed to build from source"
        cd - > /dev/null
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    # Test the built binary
    if run_with_timeout 10s "./target/release/helix" --version &> /dev/null; then
        mv "target/release/helix" "$INSTALL_DIR/helix"
        echo "Successfully built and installed from source"
    else
        echo "Built binary failed version check"
        cd - > /dev/null
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    cd - > /dev/null
    rm -rf "$TMP_DIR"
fi

# Verify installation and ensure command is available
echo "Verifying installation..."
if run_with_timeout 10s helix --version >/dev/null 2>&1; then
    INSTALLED_VERSION=$(get_binary_version "$INSTALL_DIR/helix")
    echo "Installation successful!"
    echo "Helix CLI version $INSTALLED_VERSION has been installed to $INSTALL_DIR/helix"
    echo "The 'helix' command is now available in your current shell"
    if [[ -n "$SHELL_CONFIG" ]]; then
        echo "For permanent installation, please restart your shell or run:"
        echo "    source $SHELL_CONFIG"
    fi
else
    echo "Installation failed - binary is not responding to version check."
    exit 1
fi

# Install Rust if needed
echo "Checking dependencies..."
if ! command -v cargo &> /dev/null; then
    echo "Rust/Cargo is not installed. Installing Rust..."
    if [[ "$OS" == "Linux" ]] || [[ "$OS" == "Darwin" ]]; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    elif [[ "$OS" == "Windows_NT" ]]; then
        curl --proto '=https' --tlsv1.2 -sSf https://win.rustup.rs -o rustup-init.exe
        ./rustup-init.exe -y
        rm rustup-init.exe
    fi
else
    echo "Rust/Cargo is already installed."
fi

# Final verification that helix is working
echo "Final verification..."
if run_with_timeout 10s helix --version; then
    echo "Helix CLI is working correctly!"
else
    echo "Warning: Helix CLI may not be working correctly."
    echo "Please try running 'source $SHELL_CONFIG' or restart your terminal."
    exit 1
fi


echo "Metrics are enabled by default. To disable them, run 'helix metrics --off'"
echo "Note that metrics are completely anonymous and do not contain any personal information."