{ kversion ? "linux_5_15"
, pkgs ? import <nixpkgs> {} }:

with pkgs;

let
  tools = pkgs.callPackage ./default.nix { doCheck = false ;} ;
in
mkShell {
  buildInputs = [
    linuxKernel.packages.${kversion}.perf
    gdb
    # lsp code completion in neovim/emacs
    clangd
    rust-analyzer
    rnix-lsp
  ];
  inputsFrom = [
    tools
  ];
}
