#!/usr/bin/env bash

# exodus-py Installation and Testing Script
# This script handles dependency installation, building, and testing
# for both macOS and Ubuntu environments using uv for Python dependency management.

set -e  # Exit on error
set -u  # Exit on undefined variable

# Determine script directory (absolute path)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            if [[ "$ID" == "ubuntu" ]] || [[ "$ID_LIKE" == *"ubuntu"* ]] || [[ "$ID_LIKE" == *"debian"* ]]; then
                echo "ubuntu"
            elif [[ "$ID" == "rhel" ]] || [[ "$ID" == "centos" ]] || [[ "$ID" == "rocky" ]] || [[ "$ID" == "almalinux" ]] || [[ "$ID_LIKE" == *"rhel"* ]] || [[ "$ID_LIKE" == *"fedora"* ]]; then
                echo "rhel"
            else
                echo "unknown"
            fi
        else
            echo "unknown"
        fi
    else
        echo "unknown"
    fi
}

# Check if libraries are installed
check_library() {
    local lib=$1
    if command -v pkg-config &> /dev/null; then
        pkg-config --exists "$lib" 2>/dev/null
        return $?
    fi
    return 1
}

# Get library version
get_library_version() {
    local lib=$1
    if command -v pkg-config &> /dev/null && pkg-config --exists "$lib" 2>/dev/null; then
        pkg-config --modversion "$lib" 2>/dev/null
    else
        echo "unknown"
    fi
}

# RHEL-specific: Check if HDF5 is installed (doesn't rely on pkg-config)
check_hdf5_rhel() {
    # Check for h5cc compiler wrapper
    if command -v h5cc &> /dev/null; then
        return 0
    fi
    # Check for library files in standard locations
    if [ -f /usr/lib64/libhdf5.so ] || [ -f /usr/lib/libhdf5.so ]; then
        return 0
    fi
    return 1
}

# RHEL-specific: Check if NetCDF is installed (doesn't rely on pkg-config)
check_netcdf_rhel() {
    # Check for nc-config
    if command -v nc-config &> /dev/null; then
        return 0
    fi
    # Check for library files in standard locations
    if [ -f /usr/lib64/libnetcdf.so ] || [ -f /usr/lib/libnetcdf.so ]; then
        return 0
    fi
    return 1
}

# RHEL-specific: Get HDF5 version
get_hdf5_version_rhel() {
    if command -v h5cc &> /dev/null; then
        # Extract version from h5cc -showconfig
        h5cc -showconfig 2>/dev/null | grep "HDF5 Version:" | awk '{print $3}' || echo "unknown"
    else
        echo "unknown"
    fi
}

# RHEL-specific: Get NetCDF version
get_netcdf_version_rhel() {
    if command -v nc-config &> /dev/null; then
        nc-config --version 2>/dev/null | awk '{print $2}' || echo "unknown"
    else
        echo "unknown"
    fi
}

# Install system dependencies
install_system_deps() {
    local os=$1
    log_info "Checking system dependencies for $os..."

    # Check if libraries are already installed
    local netcdf_installed=false
    local hdf5_installed=false

    if check_library "netcdf"; then
        netcdf_installed=true
        local netcdf_version=$(get_library_version "netcdf")
        log_success "NetCDF already installed: version $netcdf_version"
    fi

    if check_library "hdf5"; then
        hdf5_installed=true
        local hdf5_version=$(get_library_version "hdf5")
        log_success "HDF5 already installed: version $hdf5_version"
    fi

    case $os in
        macos)
            if ! command -v brew &> /dev/null; then
                log_error "Homebrew not found. Please install Homebrew first:"
                log_error "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
                exit 1
            fi

            # Install missing packages
            local packages_to_install=()

            if ! $netcdf_installed; then
                if brew list netcdf &> /dev/null; then
                    log_success "NetCDF installed via Homebrew (but pkg-config not detecting it)"
                else
                    packages_to_install+=("netcdf")
                fi
            fi

            if ! $hdf5_installed; then
                if brew list hdf5 &> /dev/null; then
                    log_success "HDF5 installed via Homebrew (but pkg-config not detecting it)"
                else
                    packages_to_install+=("hdf5")
                fi
            fi

            if [ ${#packages_to_install[@]} -gt 0 ]; then
                log_info "Installing missing packages via Homebrew: ${packages_to_install[*]}"
                brew install "${packages_to_install[@]}"
            else
                log_success "All required packages already installed"
            fi

            # Set environment variables for macOS
            if command -v brew &> /dev/null; then
                export NETCDF_DIR=$(brew --prefix netcdf 2>/dev/null || echo "/usr/local")
                export HDF5_DIR=$(brew --prefix hdf5 2>/dev/null || echo "/usr/local")

                # Set dynamic library paths for macOS
                local brew_prefix=$(brew --prefix)
                export DYLD_FALLBACK_LIBRARY_PATH="${brew_prefix}/lib:${NETCDF_DIR}/lib:${HDF5_DIR}/lib:${DYLD_FALLBACK_LIBRARY_PATH:-}"

                log_success "NetCDF location: $NETCDF_DIR"
                log_success "HDF5 location: $HDF5_DIR"
                log_success "Dynamic library path configured"
            fi
            ;;

        ubuntu)
            # Check if packages are installed via dpkg
            local packages_to_install=()

            if ! $netcdf_installed; then
                if ! dpkg -l libnetcdf-dev 2>/dev/null | grep -q "^ii"; then
                    packages_to_install+=("libnetcdf-dev")
                else
                    log_success "libnetcdf-dev already installed"
                fi
            fi

            if ! $hdf5_installed; then
                if ! dpkg -l libhdf5-dev 2>/dev/null | grep -q "^ii"; then
                    packages_to_install+=("libhdf5-dev")
                else
                    log_success "libhdf5-dev already installed"
                fi
            fi

            # Always ensure pkg-config is installed
            if ! command -v pkg-config &> /dev/null; then
                packages_to_install+=("pkg-config")
            fi

            if [ ${#packages_to_install[@]} -gt 0 ]; then
                log_info "Installing missing packages via apt-get: ${packages_to_install[*]}"

                # Check if we have sudo
                if command -v sudo &> /dev/null; then
                    sudo apt-get update || log_warning "apt-get update failed, continuing anyway"
                    sudo apt-get install -y "${packages_to_install[@]}"
                else
                    # Try without sudo (for restricted environments)
                    log_warning "sudo not available, attempting installation without sudo"
                    apt-get update || log_warning "apt-get update failed, continuing anyway"
                    apt-get install -y "${packages_to_install[@]}"
                fi
            else
                log_success "All required packages already installed"
            fi

            # Set environment variables for Ubuntu
            if [ -d "/usr/lib/x86_64-linux-gnu/hdf5/serial" ]; then
                export HDF5_DIR=/usr/lib/x86_64-linux-gnu/hdf5/serial
                log_success "HDF5 location: $HDF5_DIR"
            fi
            ;;

        rhel)
            # RHEL/CentOS/Rocky/AlmaLinux - use system-installed packages only
            # Note: RHEL typically doesn't ship pkg-config .pc files for HDF5,
            # so we use alternative detection methods
            log_info "Using system-installed HDF5 and NetCDF libraries..."

            # Check for HDF5 using RHEL-specific method if pkg-config didn't find it
            if ! $hdf5_installed; then
                if check_hdf5_rhel; then
                    hdf5_installed=true
                    local hdf5_version=$(get_hdf5_version_rhel)
                    log_success "HDF5 found via h5cc/library check: version $hdf5_version"
                else
                    log_error "HDF5 not found. Please ensure hdf5-devel is installed as a system package."
                    log_error "  Example: sudo dnf install hdf5-devel"
                    exit 1
                fi
            fi

            # Check for NetCDF using RHEL-specific method if pkg-config didn't find it
            if ! $netcdf_installed; then
                if check_netcdf_rhel; then
                    netcdf_installed=true
                    local netcdf_version=$(get_netcdf_version_rhel)
                    log_success "NetCDF found via nc-config/library check: version $netcdf_version"
                else
                    log_error "NetCDF not found. Please ensure netcdf-devel is installed as a system package."
                    log_error "  Example: sudo dnf install netcdf-devel"
                    exit 1
                fi
            fi

            # Set environment variables for RHEL
            # The hdf5-sys and netcdf-sys crates need these to find the libraries
            export HDF5_DIR=/usr
            export NETCDF_DIR=/usr

            # Add library path for linker
            if [ -d /usr/lib64 ]; then
                export LIBRARY_PATH="/usr/lib64:${LIBRARY_PATH:-}"
                export LD_LIBRARY_PATH="/usr/lib64:${LD_LIBRARY_PATH:-}"
            fi

            # Add include path
            if [ -d /usr/include ]; then
                export CPATH="/usr/include:${CPATH:-}"
            fi

            log_success "HDF5_DIR set to: $HDF5_DIR"
            log_success "NETCDF_DIR set to: $NETCDF_DIR"
            log_success "All required system packages are available"
            ;;

        *)
            log_error "Unsupported OS: $os"
            log_error "This script supports macOS, Ubuntu, and RHEL-based distributions"
            exit 1
            ;;
    esac

    # Verify installation
    echo ""
    log_info "Verifying library installation..."
    if check_library "netcdf"; then
        log_success "NetCDF is available: version $(get_library_version 'netcdf')"
    else
        log_warning "NetCDF not detected via pkg-config (may still work)"
    fi

    if check_library "hdf5"; then
        log_success "HDF5 is available: version $(get_library_version 'hdf5')"
    else
        log_warning "HDF5 not detected via pkg-config (may still work)"
    fi
}

# Install uv if not present
install_uv() {
    if command -v uv &> /dev/null; then
        log_success "uv is already installed: $(uv --version)"
    else
        log_info "Installing uv package manager..."
        curl -LsSf https://astral.sh/uv/install.sh | sh

        # Add uv to PATH for current session
        export PATH="$HOME/.cargo/bin:$PATH"

        if command -v uv &> /dev/null; then
            log_success "uv installed successfully: $(uv --version)"
        else
            log_error "Failed to install uv. Please install manually from https://github.com/astral-sh/uv"
            exit 1
        fi
    fi
}

# Check for Rust
check_rust() {
    if ! command -v rustc &> /dev/null; then
        log_error "Rust compiler not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        log_success "Rust installed: $(rustc --version)"
    else
        log_success "Rust is already installed: $(rustc --version)"
    fi
}

# Install build dependencies
install_build_deps() {
    log_info "Installing build dependencies (maturin)..."
    uv venv .venv
    uv pip install maturin
    log_success "Build dependencies installed"
}

# Build the distribution
build_wheel() {
    log_info "Building wheel distribution in release mode..."

    # Navigate to exodus-py directory
    cd "$SCRIPT_DIR"

    # Build the wheel
    uv run maturin build --release

    # Find the built wheel in the workspace target directory
    # (Cargo workspace puts target/ at the workspace root, not in individual crates)
    WORKSPACE_TARGET_DIR="$(dirname "$SCRIPT_DIR")/target/wheels"
    WHEEL_FILE=$(find "$WORKSPACE_TARGET_DIR" -name "exodus_py-*.whl" -type f | sort -r | head -n 1)

    if [ -z "$WHEEL_FILE" ]; then
        log_error "Wheel file not found in $WORKSPACE_TARGET_DIR"
        exit 1
    fi

    log_success "Wheel built successfully: $WHEEL_FILE"
}

# Test wheel installation in clean venv
test_wheel() {
    log_info "Testing wheel installation in clean virtual environment..."

    local test_venv="$SCRIPT_DIR/test-venv"

    # Remove old test venv if it exists
    if [ -d "$test_venv" ]; then
        log_info "Removing old test environment..."
        rm -rf "$test_venv"
    fi

    # Create clean venv using uv
    log_info "Creating clean virtual environment with uv..."
    uv venv "$test_venv"

    # Activate venv
    source "$test_venv/bin/activate"
    log_success "Virtual environment activated"

    # Install the wheel
    log_info "Installing wheel: $WHEEL_FILE"
    uv pip install "$WHEEL_FILE"

    # Verify installation
    log_info "Verifying installation..."
    python -c "import exodus; print(f'exodus-py version: {exodus.__version__}')" || {
        log_error "Failed to import exodus module"
        deactivate
        exit 1
    }

    log_success "Wheel installation verified successfully"

    # Run additional tests if pytest is available
    if [ -d "$SCRIPT_DIR/tests" ]; then
        log_info "Installing test dependencies..."
        uv pip install pytest numpy

        log_info "Running test suite..."
        if ! pytest "$SCRIPT_DIR/tests" -v; then
            log_error "Tests failed!"
            deactivate
            exit 1
        fi
        log_success "All tests passed"
    fi

    # Run mypy type checking
    log_info "Installing mypy for type checking..."
    uv pip install mypy

    log_info "Running mypy type checker on package..."
    if mypy "$SCRIPT_DIR/python/exodus" --show-error-codes --pretty; then
        log_success "Type checking passed with no errors"
    else
        log_warning "Type checking found issues"
    fi

    # Examples execution intentionally removed to avoid running optional
    # example scripts during installation/testing.

    # Deactivate and keep venv for inspection
    deactivate
    log_success "Test environment available at: $test_venv"
    log_info "To activate test environment: source $test_venv/bin/activate"
}

# Main execution
main() {
    log_info "Starting exodus-py installation and testing..."
    echo ""

    # Detect OS
    OS=$(detect_os)
    log_info "Detected OS: $OS"
    echo ""

    # Install system dependencies
    install_system_deps "$OS"
    echo ""

    # Install uv
    install_uv
    echo ""

    # Check Rust installation
    check_rust
    echo ""

    # Install build dependencies
    install_build_deps
    echo ""

    # Build wheel
    build_wheel
    echo ""

    # Test wheel installation
    test_wheel
    echo ""

    log_success "=========================================="
    log_success "Installation and testing complete!"
    log_success "=========================================="
    log_info "Wheel location: $WHEEL_FILE"
    log_info "Test environment: $SCRIPT_DIR/test-venv"
    log_info ""
    log_info "To use the installed package:"
    log_info "  source $SCRIPT_DIR/test-venv/bin/activate"
    log_info "  python -c 'import exodus; print(exodus.__version__)'"
}

# Run main function
main "$@"
