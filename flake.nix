{
  inputs = {
    nixpkgs.url = "github:NickCao/nixpkgs/nixos-unstable-small";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = import nixpkgs { inherit system; }; in
        with pkgs; rec{
          devShells.default = mkShell {
            inputsFrom = [ packages.default ];
          };
          packages.default = rustPlatform.buildRustPackage {
            name = "knot-sys";
            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = [ rustPlatform.bindgenHook ];
            buildInputs = [ knot-dns ];
          };
        }
      );
}
