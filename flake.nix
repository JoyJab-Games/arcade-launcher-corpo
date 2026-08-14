{
  description = "Arcade-Launcher — Controller-only Arcade-Cabinet Launcher (Godot 4 + Rust/gdext + NixOS)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: import ./nixos inputs;
}
