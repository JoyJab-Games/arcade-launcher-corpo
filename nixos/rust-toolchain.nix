# Single pinned Rust toolchain, shared by the dev shell (devshell.nix) and
# the release package build (rust-build.nix) so the two can't quietly drift
# onto different rustc versions. `stable.latest` resolves against whatever
# rust-overlay revision flake.lock is pinned to — fixed/reproducible until
# someone deliberately runs `nix flake update` (see devshell.nix's original
# comment, now shared here instead of duplicated).
{ pkgs }:
pkgs.rust-bin.stable.latest.default
