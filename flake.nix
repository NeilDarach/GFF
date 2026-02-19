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
    secrets = {
      url = "git+ssh://git@github.com/NeilDarach/secrets.git?shallow=1";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, nixnvim, rust, rust-overlay, secrets, ... }@inputs:
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
          in f { inherit nixpkgs pkgs makePkgs; });
    in {
      devShells = forEachSupportedSystem ({ pkgs, ... }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            nodejs
            sops
            typst
            inputs.nixnvim.packages.${pkgs.stdenv.hostPlatform.system}.nvim
            just
            (rust-bin.stable.latest.default.override {
              targets = [ "x86_64-unknown-linux-musl" "aarch64-apple-darwin" ];
              extensions = [ "rust-src" "rust-analyzer" ];

            })
          ];
          shellHook = ''
            export COOKIE="$(${pkgs.sops}/bin/sops --extract '["gff-website-cookie"]' --decrypt ${secrets}/secrets.yaml)"
            if [[ ! -f "google-auth.json" ]]; then
              ${pkgs.sops}/bin/sops --extract '["gff-google-calendar-auth"]' --decrypt "${secrets}/secrets.yaml" > google-auth.json
            fi
            if [[ ! -d gff-fetch-summary/node_modules ]]; then
              (cd gff-fetch-summary; npm install)
            fi
            export GFF_AUTH="$PWD/google-auth.json"
            export GFF_FILTER_ID="$(${pkgs.sops}/bin/sops --extract '["gff-filter-id"]' --decrypt ${secrets}/secrets.yaml)"
            export GFF_FULL_ID="$(${pkgs.sops}/bin/sops --extract '["gff-full-id"]' --decrypt ${secrets}/secrets.yaml)"
            export GFF_CALLBACK="https://goip.org.uk/gff/change"
          '';
        };
      });
      nixosModules.default = { lib, config, pkgs, ... }: {
        options.gff = {
          enable = lib.mkEnableOption "calendar-access";
          credentialsFile = lib.mkOption {
            type = lib.types.str;
            default = "./google-auth.json";
            description =
              "A path to a file containing google service account details";
          };
          callbackUrl = lib.mkOption {
            type = lib.types.str;
            default = "https://giop.org.uk/gff";
            description =
              "The externally accessable URL that google will call to notify of calendar changes";
          };
          envFile = lib.mkOption {
            type = lib.types.str;
            default = "";
            description = "A file containing environment variables to load";
          };
          publishSite = lib.mkOption {
            type = lib.types.str;
            default = "root@giop.org.uk:/opt/nginx/www";
            description = "Where to send pdfs";
          };
          stateDir = lib.mkOption {
            type = lib.types.str;
            default = "/var/run/gff";
            description = "Where to keep json files";
          };
        };
        config = let
          gff =
            self.packages.${pkgs.stdenv.hostPlatform.system}.calendar-access;
        in lib.mkIf config.gff.enable {
          environment.systemPackages = [ gff ];
          users.groups.gff = { };
          users.users.gff = {
            isSystemUser = true;
            group = "gff";
          };
          networking.firewall.allowedTCPPorts = [ 3020 ];
          systemd.services.gff = {
            description =
              "Synchronize two google calendars to show who is going to which films";
            serviceConfig = {
              Type = "simple";
              ExecStart = "${gff}/bin/calendar-access";
              EnvironmentFile = config.gff.envFile;
              User = "gff";
              Group = "gff";
            };
            wantedBy = [ "multi-user.target" ];
          };
        };
      };

      packages = forEachSupportedSystem ({ pkgs, makePkgs, nixpkgs, }: {
        default = let p = self.packages.${pkgs.stdenv.hostPlatform.system};
        in pkgs.symlinkJoin {
          name = "gff-combined";
          paths = with p; [ scripts calendar-access ];
        };

        scripts = pkgs.callPackage ./scripts { };
        calendar-access = pkgs.callPackage ./calendar-access { };
        calendar-acccess-pi =
          (makePkgs "aarch64-unknown-linux-musl").callPackage ./calendar-access
          { };
        calendar-acccess-x86 =
          (makePkgs "x86_64-unknown-linux-musl").callPackage ./calendar-access
          { };
      });

    };
}
