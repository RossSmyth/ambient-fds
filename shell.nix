{
  system ? builtins.currentSystem or "x86_64-linux",
  inputs ? import ./npins { },
  pkgs ? import inputs.nixpkgs {
    inherit system;
    overlays = [ (import inputs.rust-overlay) ];
  },
  ambients ? import ./. { inherit system inputs pkgs; },
}:
pkgs.mkShell {
  packages =
    with pkgs;
    [
      (ambients.toolchain.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      })
    ]
    ++ ambients.buildInputs
    ++ ambients.nativeBuildInputs;
}
