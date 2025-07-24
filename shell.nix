# Legacy shell.nix for backward compatibility
# Use `nix develop` or `nix-shell` to enter the development environment

{ pkgs ? import <nixpkgs> { } }:

let
  flake = builtins.getFlake (toString ./.);
in
flake.devShells.${pkgs.system}.default