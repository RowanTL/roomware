; Wraps the Windows build into one setup.exe.
; makensis is given BUNDLE, VERSION and OUTFILE with -D flags.

Unicode true
SetCompressor /SOLID lzma

Name "RoomWare ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\RoomWare"
InstallDirRegKey HKLM "Software\RoomWare" "InstallDir"
; Writing to Program Files and HKLM needs it.
RequestExecutionLevel admin
ShowInstDetails show
ShowUnInstDetails show

!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\RoomWare"

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "RoomWare"
  SetOutPath "$INSTDIR"
  File /r "${BUNDLE}/*"

  ; The shortcut takes its working directory from the last SetOutPath, so hints
  ; and config files written as relative paths resolve against the install.
  CreateShortcut "$SMPROGRAMS\RoomWare.lnk" "$INSTDIR\bin\roomware.exe"

  WriteRegStr HKLM "Software\RoomWare" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "RoomWare"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\bin\roomware.exe"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1

  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\RoomWare.lnk"
  RMDir /r "$INSTDIR"
  DeleteRegKey HKLM "Software\RoomWare"
  DeleteRegKey HKLM "${UNINSTALL_KEY}"
SectionEnd
