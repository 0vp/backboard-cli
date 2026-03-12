#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
TARGET_BIN="${CARGO_BIN_DIR}/backboard-cli"
WRAPPER_BIN="${CARGO_BIN_DIR}/wuvo"

echo "Installing backboard-cli from: ${ROOT_DIR}"
cargo install --path "${ROOT_DIR}" --force

mkdir -p "${CARGO_BIN_DIR}"
cat > "${WRAPPER_BIN}" <<EOF
#!/usr/bin/env bash
set -euo pipefail

export AGENT_PROMPTS_DIR="\${AGENT_PROMPTS_DIR:-${ROOT_DIR}/prompts}"
export AGENT_MODEL_CATALOG_PATH="\${AGENT_MODEL_CATALOG_PATH:-${ROOT_DIR}/config/models.json}"
export AGENT_CONFIG_PATH="\${AGENT_CONFIG_PATH:-${ROOT_DIR}/config/local.json}"

exec "${TARGET_BIN}" "\$@"
EOF

chmod +x "${WRAPPER_BIN}"

if [[ ":${PATH}:" != *":${CARGO_BIN_DIR}:"* ]]; then
  echo
  echo "Add Cargo bin to PATH if needed:"
  echo "  export PATH=\"${CARGO_BIN_DIR}:\$PATH\""
fi

echo
echo "Installed. Run with: wuvo"
echo "After pulling code updates, rerun: ./install.sh"
