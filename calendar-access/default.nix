{ rustPlatform, targetPlatform, lib, buildPackages, lld, pkgs }:
rustPlatform.buildRustPackage ({
  name = "calendar_access";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  depsBuildBuild = lib.optionals pkgs.stdenv.buildPlatform.isDarwin [
    buildPackages.darwin.libiconv
    buildPackages.stdenv.cc
    lld
  ];
})
