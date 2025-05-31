#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR" || exit 1

TMP_INIT="$(mktemp /tmp/nvim_project_lsp_XXXX.lua)"

cat > "$TMP_INIT" << 'EOF'
local ok, _ = pcall(dofile, vim.fn.stdpath("config") .. "/init.lua")
if not ok then
  print("[project-lsp] Warning: could not load your normal init.lua")
end

vim.cmd [[
  augroup FTLakonik
    autocmd!
    autocmd BufRead,BufNewFile *.lk setfiletype lakonik
  augroup END
]]

local lspconfig_ok, lspconfig = pcall(require, "lspconfig")
if not lspconfig_ok then
  vim.notify("[project-lsp] Error: nvim-lspconfig plugin not found!", vim.log.levels.ERROR)
  return
end

local configs_ok, configs = pcall(require, "lspconfig.configs")
if not configs_ok then
  vim.notify("[project-lsp] Error: could not require 'lspconfig.configs'", vim.log.levels.ERROR)
  return
end

configs.lakonik = {
  default_config = {
    cmd = { "cargo", "run", "--", "lsp" },
    filetypes = { "lakonik" },
    root_dir = lspconfig.util.root_pattern("Cargo.toml", ".git"),
    -- you can add more fields here if needed, e.g.:
    -- settings = { ... }, etc.
  },
}

lspconfig.lakonik.setup {}
EOF

NEWFILE="$(mktemp "${SCRIPT_DIR}/lakonik_XXXX.lk")"

nvim -u "$TMP_INIT" "$NEWFILE"
rm -f "$TMP_INIT"
rm -f "$NEWFILE"
