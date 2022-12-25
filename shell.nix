{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell
{
    name = "nuba-env";
    buildInputs = with pkgs; [
        rustc
        cargo
    ];

    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

    shellHook = ''
        export PATH="$HOME/.cargo/bin:$PATH"
    '';
}