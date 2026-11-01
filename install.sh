#!/bin/sh
# Installs agent-ready on Linux or macOS and gets ready every agent it finds. Nothing else is needed:
# the binary depends on no runtime or library.
#   curl -fsSL https://raw.githubusercontent.com/marcmayol/agent-ready/main/install.sh | sh
#
# Optional variables:
#   AGENT_READY_INSTALL_DIR   where the binary goes (default ~/.local/bin)
#   AGENT_READY_VERSION       a specific release tag (default below)
#   AGENT_READY_NO_SETUP=1    only install the binary, don't touch any agent
set -eu

repo="marcmayol/agent-ready"
tag="${AGENT_READY_VERSION:-v0.1.0}"
dir="${AGENT_READY_INSTALL_DIR:-$HOME/.local/bin}"
base="https://github.com/$repo/releases/download/$tag"

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) target=x86_64-unknown-linux-musl ;;
  Linux-aarch64 | Linux-arm64) target=aarch64-unknown-linux-musl ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  *)
    echo "agent-ready: no prebuilt binary for $(uname -s) $(uname -m); build it with: cargo install --git https://github.com/$repo" >&2
    exit 1
    ;;
esac

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
paquete="agent-ready-$target.tar.gz"
echo "Downloading $paquete..."
curl -fsSL "$base/$paquete" -o "$tmp/$paquete"
curl -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS"

# The checksum must match before anything is installed.
esperada="$(grep " $paquete\$" "$tmp/SHA256SUMS" | cut -d' ' -f1)"
if command -v sha256sum >/dev/null 2>&1; then
  real="$(sha256sum "$tmp/$paquete" | cut -d' ' -f1)"
else
  real="$(shasum -a 256 "$tmp/$paquete" | cut -d' ' -f1)"
fi
if [ -z "$esperada" ] || [ "$esperada" != "$real" ]; then
  echo "agent-ready: the SHA-256 checksum of $paquete doesn't match; nothing was installed." >&2
  exit 1
fi

tar -xzf "$tmp/$paquete" -C "$tmp"
mkdir -p "$dir"
mv "$tmp/agent-ready" "$dir/agent-ready"
chmod +x "$dir/agent-ready"
echo "Installed: $dir/agent-ready ($("$dir/agent-ready" --version))"

if [ -n "${AGENT_READY_NO_SETUP:-}" ]; then
  echo "Next: $dir/agent-ready setup <agent>   (or --all, for every agent on this computer)"
  exit 0
fi
# The agents get the binary's full path, so they find it even if this folder isn't in PATH.
if ! "$dir/agent-ready" setup --all; then
  echo "agent-ready: some agents couldn't be set up (see above); the others are ready." >&2
fi
echo "An agent you install later: $dir/agent-ready setup <agent>"
