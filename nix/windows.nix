# Cross compiles RoomWare for 64-bit Windows and bundles it with the GTK and
# GStreamer DLLs it needs to run, following the mingw example from
# oxalica/rust-overlay.
#
# The libraries are MSYS2's Windows builds rather than nixpkgs ones, because
# nixpkgs can't build GTK for Windows. `nix/update-msys2.py` pins them.
#
# This is vibe coded
{
  lib,
  stdenv,
  runCommand,
  fetchurl,
  makeRustPlatform,
  buildPackages,
  binutils,
  windows,
  zstd,
  rustToolchain,
}:

let
  # MSYS2 installs the UCRT64 environment under this prefix, and its .pc files
  # and DLL lookups are all written for it.
  prefix = "ucrt64";

  sources = map (
    package:
    fetchurl {
      urls = map (mirror: "${mirror}/mingw/${prefix}/${package.filename}") [
        "https://repo.msys2.org"
        "https://mirror.msys2.org"
      ];
      # MSYS2 writes a package epoch as `~`, which a store path can't hold.
      name = builtins.replaceStrings [ "~" ] [ "-" ] package.filename;
      inherit (package) sha256;
    }
  ) (lib.importJSON ./msys2-packages.json);

  # GTK, GStreamer and everything below them, as Windows binaries laid out the
  # way MSYS2 installs them.
  sysroot = runCommand "msys2-${prefix}" { nativeBuildInputs = [ zstd ]; } ''
    mkdir -p $out
    for package in ${toString sources}; do
      # Every package also holds pacman's own metadata files, which are dotfiles.
      tar --zstd -xf "$package" -C $out --exclude='.*'
    done

    # The .pc files describe an install at /${prefix}. Point them at this one.
    find $out/${prefix} -name '*.pc' -exec sed -i "s|/${prefix}|$out/${prefix}|g" {} +
  '';

  rustPlatform = makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

  roomware = rustPlatform.buildRustPackage {
    pname = "roomware";
    inherit ((lib.importTOML ../Cargo.toml).package) version;

    src = lib.fileset.toSource {
      root = ../.;
      fileset = lib.fileset.unions [
        ../Cargo.toml
        ../Cargo.lock
        ../src
      ];
    };
    cargoLock.lockFile = ../Cargo.lock;

    buildInputs = [ windows.pthreads ];

    # The gtk4 crates find their C libraries through pkg-config. Point it at
    # the sysroot and nothing else, so it can't reach this machine's Linux
    # libraries instead.
    env = {
      PKG_CONFIG = "${buildPackages.pkgconf-unwrapped}/bin/pkgconf";
      PKG_CONFIG_LIBDIR = "${sysroot}/${prefix}/lib/pkgconfig:${sysroot}/${prefix}/share/pkgconfig";
      PKG_CONFIG_ALLOW_CROSS = "1";
    };

    # Running the tests would need Windows.
    doCheck = false;

    meta = {
      description = "Escape room game master and player displays, for Windows";
      mainProgram = "roomware.exe";
      platforms = lib.platforms.windows;
    };
  };
in
# Everything the game master needs on the Windows machine, in one directory.
runCommand "roomware-windows-${roomware.version}" { nativeBuildInputs = [ binutils ]; } ''
  mkdir -p $out/bin $out/lib $out/share/glib-2.0/schemas
  cp ${roomware}/bin/roomware.exe $out/bin

  # GTK and GStreamer find their modules and data relative to where they were
  # loaded from, so the layout beside the exe has to match MSYS2's.
  mkdir -p $out/lib/gstreamer-1.0 $out/libexec
  cp ${sysroot}/${prefix}/lib/gstreamer-1.0/*.dll $out/lib/gstreamer-1.0
  cp -r ${sysroot}/${prefix}/libexec/gstreamer-1.0 $out/libexec
  cp -r ${sysroot}/${prefix}/share/icons $out/share
  chmod -R u+w $out

  # GSettings only reads schemas once they are compiled together.
  cp ${sysroot}/${prefix}/share/glib-2.0/schemas/*.xml $out/share/glib-2.0/schemas
  ${buildPackages.glib.dev}/bin/glib-compile-schemas $out/share/glib-2.0/schemas

  # Windows loads DLLs from beside the exe, and the sysroot holds far more of
  # them than this program ever touches. So take what the exe and the plugins
  # import, then what those import in turn, and nothing else. An import need
  # not name its DLL in the same case as the file on disk, hence the lower
  # casing.
  declare -A available
  for dll in ${sysroot}/${prefix}/bin/*.dll; do
    name=''${dll##*/}
    available[''${name,,}]=$dll
  done

  queue=($out/bin/roomware.exe $out/lib/gstreamer-1.0/*.dll $out/libexec/gstreamer-1.0/*.exe)
  declare -A taken
  while [ ''${#queue[@]} -gt 0 ]; do
    current=''${queue[0]}
    queue=("''${queue[@]:1}")

    for name in $(${stdenv.cc.targetPrefix}objdump -p "$current" | sed -n 's/^\s*DLL Name: //p'); do
      key=''${name,,}
      source=''${available[$key]-}

      # Anything missing here is a Windows DLL that the machine already has.
      if [ -n "$source" ] && [ -z "''${taken[$key]-}" ]; then
        taken[$key]=1
        cp "$source" $out/bin
        queue+=($out/bin/''${source##*/})
      fi
    done
  done
''
