{ workspaceSrc, rustBuilder, rustVersion }:
(rustBuilder.makePackageSet {
  inherit rustVersion workspaceSrc;
  packageFun = import ./wasm-server-runner-Cargo.nix;
}).workspace.wasm-server-runner {}