{
  description = "Rust dev environment flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-edit
            pkgs.cargo-watch
            pkgs.cargo-audit
            pkgs.clippy
            pkgs.rustfmt
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
            echo "Rust development environment ready!"
          '';
        };
      });
}

