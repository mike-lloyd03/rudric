{
  description = "Rudric, the keeper of secrets";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = inputs:
    with inputs;
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages = {
        ${system}.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rudric";
          version = "0.1.6";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          checkFlags = [ "--skip=integration::test_init" ];
        };
      };

      devShells.${system}.default =
        pkgs.mkShell { packages = with pkgs; [ sqlite ]; };
    };
}
