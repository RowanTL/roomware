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
  makensis -DBUNDLE=${bundle} -DVERSION=${version} -DOUTFILE=$out ${./installer.nsi}
''
