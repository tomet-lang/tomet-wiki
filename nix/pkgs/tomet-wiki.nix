{
  craneLib,
}:
let
  src = ../..;

  commonArgs = {
    inherit src;

    pname = "tomet-wiki";
    version = "0.1.0";
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    cargoExtraArgs = "-p tomet-wiki";

    doCheck = true;
  }
)
