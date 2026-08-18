{
  system ? builtins.currentSystem or "x86_64-linux",
  inputs ? import ./npins { },
  pkgs ? import inputs.nixpkgs { inherit system; },
}:
let
  inherit (pkgs) rustPlatform lib;
in
rustPlatform.buildRustPackage {
  pname = "ambients";
  version = "0.0.1";

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./src
      ./Cargo.lock
      ./Cargo.toml
      ./examples
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  cargoBuildFlags = [
    "--lib"
  ];
  
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];

  buildInputs = with pkgs; [
    systemdLibs
  ];
}
