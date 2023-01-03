{
  description = "Userspace tools for bcachefs";

  # Nixpkgs / NixOS version to use.
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  inputs.utils.url = "github:numtide/flake-utils";
  inputs.flake-compat = {
    url = "github:edolstra/flake-compat";
    flake = false;
  };

  outputs = { self, nixpkgs, utils, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        binary = pkgs.callPackage ./binary.nix {
            testWithValgrind = false;
        };
        mount = pkgs.callPackage ./rust-src/mount/default.nix { inherit binary; };
        bcachefs = pkgs.callPackage ./base.nix {
          inherit binary mount;
          };
      in {
        packages = {
          inherit binary mount;
          default = bcachefs;
        };
      });
}
