{ lib
, stdenv
, glibc
, udev
, llvmPackages
, rustPlatform
, binary
, ...
}: rustPlatform.buildRustPackage ( let 
  cargo = lib.trivial.importTOML ./Cargo.toml;
in {
  pname = "mount.bcachefs";
  version = cargo.package.version;

  src = builtins.path { path = ../.; name = "rust-src"; };
  sourceRoot = "rust-src/mount";

  cargoLock = { lockFile = ./Cargo.lock; };

  nativeBuildInputs = [ binary rustPlatform.bindgenHook ];
  buildInputs = [ binary ];

  LIBBCACHEFS_LIB ="${binary}/lib";
  LIBBCACHEFS_INCLUDE = binary.src;

  doCheck = false;
})
