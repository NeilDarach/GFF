{ lib, pkgs }:
pkgs.stdenv.mkDerivation rec {
  pname = "gff-scripts";
  version = "0.1.2";
  src = ./..;
  nativeBuildInputs = with pkgs; [ makeWrapper ];
  buildInputs = with pkgs; [ jq coreutils curl typst bash openssh ];
  dontUnpack = true;
  dontPatch = true;
  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    mkdir -p $out/bin $out/brochure
    cp $src/scripts/* $out/bin
    cp $src/brochure/* $out/brochure
  '';
  postFixup = ''
    for script in $out/bin/*.sh $out/bin/gff-* ; do
      wrapProgram $script --set PATH '${lib.makeBinPath buildInputs}'
    done
  '';

}
