
#!/usr/bin/env bash
#
# start-rust-lsp.sh
#
# Place this in your Rust project root. It will:
#   1. cd into the project directory
#   2. write a tiny “init.lua” that:
#        • sources your normal ~/.config/nvim/init.lua
#        • explicitly requires `lspconfig.configs` and then registers
#          a custom server “rust_custom” whose cmd = { "cargo", "run", "--", "lsp" }
#          and filetypes = { "rust" }
#   3. launch nvim with -u pointing at that temporary init.lua
#   4. delete that temp file once Neovim exits
#

# 1) Ensure we’re in the project root
cd "$(dirname "$0")" || exit 1

# 2) Create a temporary init.lua in /tmp
TMP_INIT="$(mktemp /tmp/nvim_project_lsp_XXXX.lua)"

cat > "$TMP_INIT" << 'EOF'
-- ──────────────────────────────────────────────────────────────────────────
-- Tiny init.lua that:
--   a) Loads your normal ~/.config/nvim/init.lua
--   b) Explicitly requires `lspconfig.configs` so we can register a custom server
--   c) Defines “rust_custom” → { default_config = { cmd = { "cargo", "run", "--", "lsp" }, … } }
--   d) Calls `rust_custom.setup{}` so that Neovim’s LSP client will auto‐attach
--      whenever you open a Rust file.
-- ──────────────────────────────────────────────────────────────────────────

-- 1) Source your regular init.lua (so your plugins/keymaps carry over)
--    Use pcall in case the path is different.
local ok, _ = pcall(dofile, vim.fn.stdpath("config") .. "/init.lua")
if not ok then
  -- If your init.lua lives elsewhere, adjust this path:
  -- pcall(dofile, os.getenv("HOME") .. "/.config/nvim/init.lua")
  print("[project-lsp] Warning: could not load your normal init.lua")
end

-- 2) Require the core 'lspconfig' and also explicitly grab 'lspconfig.configs'
--    so we can register a custom server definition.
local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  vim.notify("[project-lsp] Error: nvim-lspconfig plugin not found!", vim.log.levels.ERROR)
  return
end

-- Make sure `lspconfig.configs` is present
local configs_ok, configs = pcall(require, "lspconfig.configs")
if not configs_ok then
  vim.notify("[project-lsp] Error: could not require 'lspconfig.configs'", vim.log.levels.ERROR)
  return
end

-- 3) Define our custom server under `lspconfig.configs`
--    (so that `lspconfig.rust_custom.setup{}` becomes valid)
configs.lakonik_debug = {
  default_config = {
    cmd = { "cargo", "run", "--", "lsp" },
    filetypes = { "rust" },
    root_dir = lspconfig.util.root_pattern("Cargo.toml", ".git"),
    -- you can add more fields here if needed, e.g.:
    -- settings = { ... }, root_dir, etc.
  },
}

-- 4) Actually call setup so that Neovim attaches it on any Rust buffer
lspconfig.lakonik_debug.setup {}

-- (Optional) If you want to customize on_attach / capabilities, you could do:
-- lspconfig.rust_custom.setup {
--   on_attach = function(client, bufnr) … end,
--   capabilities = require("cmp_nvim_lsp").default_capabilities(),
-- }

EOF

# 3) Launch nvim with our temp init.lua (forward any filename args)
nvim -u "$TMP_INIT" "$@"

# 4) Cleanup the temp init.lua when Neovim exits
rm -f "$TMP_INIT"

