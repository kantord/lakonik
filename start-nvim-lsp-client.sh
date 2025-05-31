
#!/usr/bin/env bash
#
# start-rust-lsp.sh
#
# Place this in your Rust project root. It will:
#   1. cd into the project directory
#   2. write a tiny “init.lua” that:
#        • sources your normal ~/.config/nvim/init.lua
#        • defines an autocmd so that *.lk files become filetype=lakonik
#        • explicitly requires `lspconfig.configs` and then registers
#          a custom server “lakonik” whose cmd = { "cargo", "run", "--", "lsp" }
#          and filetypes = { "lakonik" }
#   3. if you did not pass any filename, create a new temp file ending in .lk
#      in the project root and open it; otherwise open the file(s) you passed
#   4. launch nvim with -u pointing at that temporary init.lua
#   5. delete that temp init.lua once Neovim exits
#

# 1) Ensure we’re in the project root
cd "$(dirname "$0")" || exit 1

# 2) Create a temporary init.lua in /tmp
TMP_INIT="$(mktemp /tmp/nvim_project_lsp_XXXX.lua)"

cat > "$TMP_INIT" << 'EOF'
-- ──────────────────────────────────────────────────────────────────────────
-- Tiny init.lua that:
--   a) Loads your normal ~/.config/nvim/init.lua
--   b) Sets up an autocmd so that *.lk → filetype=lakonik
--   c) Explicitly requires `lspconfig.configs` so we can register a custom server
--   d) Defines “lakonik” → { default_config = { cmd = { "cargo", "run", "--", "lsp" }, … } }
--   e) Calls `lakonik.setup{}` so that Neovim’s LSP client will auto‐attach
--      whenever you open a file with filetype “lakonik”.
-- ──────────────────────────────────────────────────────────────────────────

-- 1) Source your regular init.lua (so your plugins/keymaps carry over)
--    Use pcall in case the path is different.
local ok, _ = pcall(dofile, vim.fn.stdpath("config") .. "/init.lua")
if not ok then
  -- If your init.lua lives elsewhere, adjust this path:
  -- pcall(dofile, os.getenv("HOME") .. "/.config/nvim/init.lua")
  print("[project-lsp] Warning: could not load your normal init.lua")
end

-- 2) Autocmd so that *.lk files become filetype=lakonik
vim.cmd [[
  augroup FTLakonik
    autocmd!
    autocmd BufRead,BufNewFile *.lk setfiletype lakonik
  augroup END
]]

-- 3) Require the core 'lspconfig' and also explicitly grab 'lspconfig.configs'
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

-- 4) Define our custom server under `lspconfig.configs`
--    (so that `lspconfig.lakonik.setup{}` becomes valid)
configs.lakonik = {
  default_config = {
    cmd = { "cargo", "run", "--", "lsp" },
    filetypes = { "lakonik" },
    root_dir = lspconfig.util.root_pattern("Cargo.toml", ".git"),
    -- you can add more fields here if needed, e.g.:
    -- settings = { ... }, etc.
  },
}

-- 5) Actually call setup so that Neovim attaches it on any buffer with filetype “lakonik”
lspconfig.lakonik.setup {}

-- (Optional) If you want to customize on_attach / capabilities, you could do:
-- lspconfig.lakonik.setup {
--   on_attach = function(client, bufnr) … end,
--   capabilities = require("cmp_nvim_lsp").default_capabilities(),
-- }

EOF

# 3) Determine which file(s) to open:
if [ $# -eq 0 ]; then
  # No filename passed → create a new .lk file in the project root
  NEWFILE="$(mktemp "${PWD}/lakonik_XXXX.lk")"
  touch "$NEWFILE"
  FILE_ARGS=("$NEWFILE")
else
  # Forward any passed arguments verbatim
  FILE_ARGS=("$@")
fi

# 4) Launch Neovim with our temp init.lua, opening the desired file(s)
nvim -u "$TMP_INIT" "${FILE_ARGS[@]}"

# 5) Cleanup the temp init.lua when Neovim exits
rm -f "$TMP_INIT"

