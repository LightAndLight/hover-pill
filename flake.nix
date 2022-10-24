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

              # required by `.cargo/config.toml`
              (pkgs.callPackage ./nix/wasm-server-runner.nix {
                workspaceSrc = wasm-server-runner;
                inherit rustVersion;
              })
              pkgs.clang_14
            ];

            shellHook = 
              let
                config-toml = pkgs.writeText "config.toml" ''
                  [target.x86_64-unknown-linux-gnu]
                  linker = "clang"
                  rustflags = ["-C", "link-arg=-fuse-ld=${pkgs.mold}/bin/mold"]

                  [target.wasm32-unknown-unknown]
                  runner = "wasm-server-runner"
                '';
              in ''
                mkdir -p .cargo

                ln --symbolic --force ${config-toml} .cargo/config.toml
              '';
          };
      }
    );
}
