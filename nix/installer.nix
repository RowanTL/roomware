# Packs the Windows build from windows.nix into a single setup.exe.
{
  lib,
  runCommand,
  nsis,
  bundle,
}:

let
  version = (lib.importTOML ../Cargo.toml).package.version;
in
runCommand "roomware-setup-${version}.exe" { nativeBuildInputs = [ nsis ]; } ''
  # installer.nsi references the icon by a path relative to itself, so both
  # need to be copied into the same directory rather than passed as lone
  # store paths.
  mkdir build
  cp ${./installer.nsi} build/installer.nsi
  cp ${./escapeonmain.ico} build/escapeonmain.ico

  makensis -DBUNDLE=${bundle} -DVERSION=${version} -DOUTFILE=$out build/installer.nsi
''
