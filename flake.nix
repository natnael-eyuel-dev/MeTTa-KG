{
  description = "MeTTa-KG reproducible build (optional)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage rec {
          pname = "metta-kg";
          version = "0.1.0";
          src = ./.;
          cargoLock = { lockFile = ./api/Cargo.lock; };

          nativeBuildInputs = [ pkgs.nodejs_20 pkgs.pkg-config pkgs.openssl pkgs.postgresql ];

          preBuild = ''
            echo "Building frontend"
            (cd frontend && npm ci && npm run build)
          '';

          buildAndTestSubdir = "api";

          cargoBuildFlags = [ "--release" ];

          RUSTFLAGS = "-C strip=symbols";

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/metta-kg $out/bin/
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.nodejs_20
            pkgs.openssl
            pkgs.postgresql
          ];
        };
      }
    );
}
