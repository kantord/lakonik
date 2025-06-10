> [!CAUTION]
> This project is in an extremely early stage of development. It is being developed in the open, but it is not ready to be used yet

# Lakonik

Lakonik is a grammar for AI: it provides end-users with an alternative way to interacti with LLMs and AI agents.

It attemps to combine ideas from `vim` and tools like AutoHotKey with the goal of steamlining interaction and
minimizing the requried keystrokes to write a prompt.


## Development

### LSP

The easiest way to do manual testing on the LSP server is to use the included
**start-nvim-lsp-client.sh** script. This will start an instance of `nvim`,
with a custom config file that automatically connects to a development build of
the LSP server.

This scripts requires `lspconfig` to be installed in your local `nvim` configuration.
If you don't use `nvim` for development, the easiest way to get a working
instance of `nvim` is to install a neovim distribution such as
[NvChad](https://nvchad.com/).


