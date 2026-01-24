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
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      overlays = [ (import rust-overlay) ];
      forEachSupportedSystem = f:
        nixpkgs.lib.genAttrs supportedSystems (system:
          let
            pkgs = import nixpkgs { inherit system overlays; };
            makePkgs = config:
              import nixpkgs {
                inherit overlays system;
                crossSystem = {
                  inherit config;
                  rustc = { inherit config; };
                  isStatic = true;
                };
              };
          in f { inherit pkgs makePkgs; });
    in {
      devShells = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            nodejs
            sops
            typst
            inputs.nixnvim.packages.${pkgs.stdenv.hostPlatform.system}.nvim
            (rust-bin.stable.latest.default.override {
              targets = [ "x86_64-unknown-linux-musl" "aarch64-apple-darwin" ];
            })
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
      packages = forEachSupportedSystem ({ pkgs, makePkgs }: {
        default = pkgs.callPackage ./calendar-access { };
        pi =
          (makePkgs "aarch64-unknown-linux-musl").callPackage ./calendar-access
          { };
        mac = if (pkgs.stdenv.hostPlatform.system == "aarch64-darwin") then
          pkgs.callPackage ./calendar-access { }
        else
          (makePkgs "aarch64-unknown-darwin-musl").callPackage ./calendar-access
          { };
        x86 =
          (makePkgs "x86_64-unknown-linux-musl").callPackage ./calendar-access
          { };
      });

    };
}

