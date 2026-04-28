{
  description = "wasm-engine — wasmtime-based runtime that executes WASM/WASI modules dispatched by wasm-operator";

  nixConfig = {
    allow-import-from-derivation = true;
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    forge = {
      url = "github:pleme-io/forge";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.substrate.follows = "substrate";
      inputs.crate2nix.follows = "crate2nix";
    };
    crate2nix = {
      url = "github:nix-community/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, substrate, forge, crate2nix, ... }:
    (import "${substrate}/lib/rust-service-flake.nix" {
      inherit nixpkgs substrate forge crate2nix;
    }) {
      inherit self;
      serviceName = "wasm-engine";
      registry = "ghcr.io/pleme-io/wasm-engine";
      packageName = "wasm-engine";
      namespace = "tatara-system";
      architectures = ["amd64" "arm64"];
      ports = { http = 8080; health = 8080; metrics = 8080; };
      serviceType = "rest";

      # Skip the on-disk module/{default,nixos}.nix loaders — the trio
      # below produces all three module shapes from one spec.
      moduleDir = null;
      nixosModuleFile = null;

      module = {
        description = "Wasmtime-based runtime that executes WASM/WASI modules dispatched by wasm-operator";
      };
    };
}
