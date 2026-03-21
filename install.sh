#!/bin/sh
# Scope install script
# Usage: curl -fsSL https://raw.githubusercontent.com/rynhardt-potgieter/scope/main/install.sh | sh
set -e

REPO="rynhardt-potgieter/scope"
BINARY="scope"

# Determine OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "${OS}" in
        Linux)
            case "${ARCH}" in
                x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
                *)       echo "Error: unsupported Linux architecture: ${ARCH}" >&2; exit 1 ;;
            esac
            ;;
        Darwin)
            case "${ARCH}" in
                x86_64)      TARGET="x86_64-apple-darwin" ;;
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                *)           echo "Error: unsupported macOS architecture: ${ARCH}" >&2; exit 1 ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            case "${ARCH}" in
                x86_64)  TARGET="x86_64-pc-windows-msvc" ;;
                *)       echo "Error: unsupported Windows architecture: ${ARCH}" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Error: unsupported operating system: ${OS}" >&2
            exit 1
            ;;
    esac
}

# Get the latest release tag from GitHub
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "${VERSION}" ]; then
        echo "Error: could not determine latest release version" >&2
        exit 1
    fi
}

# Choose install directory
choose_install_dir() {
    if [ -d "${HOME}/.local/bin" ]; then
        INSTALL_DIR="${HOME}/.local/bin"
    elif [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "${INSTALL_DIR}"
    fi
}

# Download and install the binary
install() {
    detect_platform
    get_latest_version
    choose_install_dir

    if [ "${TARGET}" = "x86_64-pc-windows-msvc" ]; then
        FILENAME="${BINARY}-${VERSION}-${TARGET}.exe"
    else
        FILENAME="${BINARY}-${VERSION}-${TARGET}"
    fi

    URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

    echo "Installing Scope ${VERSION} for ${TARGET}..."
    echo "  Downloading from ${URL}"

    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "${TMP_DIR}"' EXIT

    curl -fsSL "${URL}" -o "${TMP_DIR}/${BINARY}"
    chmod +x "${TMP_DIR}/${BINARY}"
    mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"

    echo "  Installed to ${INSTALL_DIR}/${BINARY}"
    echo ""

    # Check if install dir is in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo "Note: ${INSTALL_DIR} is not in your PATH."
            echo "Add it with:"
            echo ""
            echo "  export PATH=\"${INSTALL_DIR}:\${PATH}\""
            echo ""
            ;;
    esac

    echo "Run 'scope --help' to get started."
}

install
