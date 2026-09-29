# Packs the Windows build from windows.nix into a single setup.exe.
{
  lib,
  runCommand,
  writeText,
  nsis,
  bundle,
}:

let
  version = (lib.importTOML ../Cargo.toml).package.version;

  # Each directory in rooms/ becomes one choice on the components page. Only
  # the first starts ticked, and the radio buttons keep it to one at a time.
  rooms = lib.attrNames (lib.filterAttrs (_: type: type == "directory") (builtins.readDir ../rooms));
  section = i: "room_${toString i}";

  roomsNsh = writeText "rooms.nsh" ''
    ${lib.concatImapStrings (i: room: ''
      Section ${lib.optionalString (i > 1) "/o "}"${lib.toSentenceCase room}" ${section i}
        !insertmacro InstallRoom ${room}
      SectionEnd
    '') rooms}
    !define FIRST_ROOM ''${${section 1}}

    !macro RoomRadioButtons
    ${lib.concatImapStrings (i: _: "  !insertmacro RadioButton \${${section i}}\n") rooms}!macroend
  '';
in
runCommand "roomware-setup-${version}.exe" { nativeBuildInputs = [ nsis ]; } ''
  # installer.nsi references the icon and rooms.nsh by paths relative to
  # itself, so they need to be copied into the same directory rather than
  # passed as lone store paths.
  mkdir build
  cp ${./installer.nsi} build/installer.nsi
  cp ${./escapeonmain.ico} build/escapeonmain.ico
  cp ${roomsNsh} build/rooms.nsh

  makensis -DBUNDLE=${bundle} -DROOMS=${../rooms} -DVERSION=${version} -DOUTFILE=$out build/installer.nsi
''
