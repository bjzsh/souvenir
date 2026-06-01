{ pkgs ? import <nixpkgs> {} }:
let
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/main.tar.gz") {};
in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
      (with fenix; combine [
        stable.toolchain
        targets.wasm32-unknown-unknown.stable.rust-std
      ])

      sqlx-cli
      nodejs_22
      bun
      binaryen

      jq
    ];
  }
