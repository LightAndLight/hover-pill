{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo2nix.url = "github:cargo2nix/cargo2nix?ref=release-0.11.0";
    wasm-server-runner = {
      url = "github:jakobhellermann/wasm-server-runner";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, cargo2nix, wasm-server-runner }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            cargo2nix.overlays.cargo2nix
          ];
        };

        rustVersion = "1.62.1";
      
      in {
        devShell =
          pkgs.mkShell {
            LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib:${pkgs.udev}/lib:${pkgs.alsaLib}/lib";
            buildInputs = [
              (pkgs.rust-bin.stable.${rustVersion}.default.override {
                targets = pkgs.rust-bin.stable.${rustVersion}.default.targets or [] ++ [ "wasm32-unknown-unknown" ];
                extensions = [
                  "cargo"
                  "clippy"
                  "rustc"
                  "rust-src"
                  "rustfmt"
                ];
              })

              cargo2nix.packages.${system}.cargo2nix
              (pkgs.callPackage ./nix/wasm-server-runner.nix {
                workspaceSrc = wasm-server-runner;
                inherit rustVersion;
              })
              pkgs.wasm-bindgen-cli
              pkgs.binaryen

              # Required to build Rust dependencies
              pkgs.cmake
              pkgs.pkgconfig
              pkgs.fontconfig
              pkgs.xorg.libX11
              pkgs.xorg.libXcursor
              pkgs.xorg.libXrandr
              pkgs.xorg.libXi
              pkgs.alsaLib
              pkgs.udev
             
              # required by wasm-pack
              pkgs.openssl

              pkgs.mold
              pkgs.clang_14
            ];
          };
      }
    );
}
