{
  description = "RoomWare dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "clippy" "rust-src" ];
        };

        # The same nixpkgs, building for 64-bit Windows. MSYS2's libraries are
        # built against the UCRT, so this has to be as well.
        pkgsWindows = import nixpkgs {
          localSystem = system;
          crossSystem = {
            config = "x86_64-w64-mingw32";
            libc = "ucrt";
          };
        };
      in
      {
        # `nix build .#windows` leaves the Windows build in ./result.
        packages.windows = pkgsWindows.callPackage ./nix/windows.nix {
          # A flake output can't be cross compiled on its own, so the toolchain
          # is built from the Windows package set rather than an overlay. It
          # picks up that target's std on its own.
          rustToolchain =
            (rust-overlay.lib.mkRustBin { } pkgsWindows.buildPackages).stable.latest.minimal;
        };

        # `nix run .#update-msys2` re-pins the Windows libraries.
        apps.update-msys2 = {
          type = "app";
          program = pkgs.lib.getExe (pkgs.writeShellApplication {
            name = "update-msys2";
            runtimeInputs = with pkgs; [ python3 zstd ];
            text = ''python3 ${./nix/update-msys2.py} "''${1:-nix/msys2-packages.json}"'';
          });
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            meson
            ninja
            desktop-file-utils
            glib
            gcc
          ];

          buildInputs = with pkgs; [
            gtk4
            libadwaita
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-bad
            gst_all_1.gst-libav
          ];
        };
      });
}
