{
  system ? builtins.currentSystem or "x86_64-linux",
  inputs ? import ./npins { },
  pkgs ? import inputs.nixpkgs { inherit system; },
  ambient-fds ? import ./. { inherit system inputs pkgs; },
}:
let
  inherit (pkgs) mkShell;
in
mkShell {
  inputsFrom = [
    ambient-fds
  ];

  packages = with pkgs; [
    clippy
    rust-analyzer
    rustfmt
  ];
}
