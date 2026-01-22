{
  description =
    "Process the Glasgow Film festival website data and synchronize google calendars";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    nixnvim = {
      url = "github:NeilDarach/nixNvim";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
    rust = {
      url = "github:NeilDarach/flakes?dir=rust";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = { nixpkgs.follows = "nixpkgs"; };
    };
  };

  outputs = { self, nixpkgs, nixnvim, rust, rust-overlay, ... }@inputs:
    let
      supportedSystems =
        [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f:
        nixpkgs.lib.genAttrs supportedSystems (system:
          let
            pkgs = import nixpkgs {
              inherit system;
              overlays = (rust.overlays.withExtensions {
                targ = [ "x86_64-unknown-linux-musl" ];
              });
            };
          in f { inherit pkgs; });
    in {
      devShells = forEachSupportedSystem ({ pkgs }: {
        default = rust.makeDevShell pkgs {
          packages = with pkgs; [
            nodejs
            sops
            typst
            inputs.nixnvim.packages.${pkgs.stdenv.hostPlatform.system}.nvim
          ];
          shellHook = ''
            export COOKIE="$(${pkgs.sops}/bin/sops --extract '["gff_website_cookie"]' --decrypt ${
              toString ./secrets.yaml
            })"
            if [[ ! -f "google-auth.json" ]]; then
              ${pkgs.sops}/bin/sops --extract '["google_calendar_credentials"]' --decrypt "${
                toString ./secrets.yaml
              }" > google-auth.json
            fi
          '';
        };
      });
    };
}

