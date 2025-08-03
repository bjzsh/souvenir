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
      wasm-pack
      wasm-bindgen-cli_0_2_100
      binaryen

      jq
    ];
  }
