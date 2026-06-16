; Rust Viewer Pro — Windows installer (NSIS)
; Builds: makensis -DVERSION=x.y.z installer.nsi
; Produces an installer that registers the app as an image handler and lets the
; user opt into associating image files (so it appears in Windows "Default apps").

Unicode true

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

!define APPNAME      "Rust Viewer Pro"
!define EXENAME      "rust-viewer-pro.exe"
!define PROGID       "RustViewerPro.Image"
!define PUBLISHER    "micferna"
!define CAPKEY       "Software\RustViewerPro\Capabilities"

!include "MUI2.nsh"

Name "${APPNAME}"
OutFile "rust-viewer-pro-setup-x86_64.exe"
InstallDir "$PROGRAMFILES64\${APPNAME}"
InstallDirRegKey HKLM "Software\${APPNAME}" "InstallDir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName"   "${APPNAME}"
VIAddVersionKey "FileVersion"   "${VERSION}"
VIAddVersionKey "CompanyName"   "${PUBLISHER}"
VIAddVersionKey "LegalCopyright" "© 2026 ${PUBLISHER}"
VIAddVersionKey "FileDescription" "${APPNAME} installer"

!define MUI_ICON "..\..\assets\icon-256.ico"
!define MUI_UNICON "..\..\assets\icon-256.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "French"

; --- Core install ------------------------------------------------------------
Section "${APPNAME} (required)" SecCore
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "..\..\target\release\${EXENAME}"
  File "..\..\assets\icon-256.ico"

  WriteRegStr HKLM "Software\${APPNAME}" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Start-menu shortcut
  CreateDirectory "$SMPROGRAMS\${APPNAME}"
  CreateShortcut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\${EXENAME}" "" "$INSTDIR\icon-256.ico"

  ; Add/Remove Programs entry
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\${EXENAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\uninstall.exe"

  ; ProgId used to open images
  WriteRegStr HKLM "Software\Classes\${PROGID}" "" "Image File"
  WriteRegStr HKLM "Software\Classes\${PROGID}\DefaultIcon" "" "$INSTDIR\${EXENAME},0"
  WriteRegStr HKLM "Software\Classes\${PROGID}\shell\open\command" "" '"$INSTDIR\${EXENAME}" "%1"'

  ; Capabilities so the app appears in Windows "Default apps"
  WriteRegStr HKLM "${CAPKEY}" "ApplicationName" "${APPNAME}"
  WriteRegStr HKLM "${CAPKEY}" "ApplicationDescription" "Fast, cross-platform image viewer"
  WriteRegStr HKLM "Software\RegisteredApplications" "${APPNAME}" "${CAPKEY}"
SectionEnd

; --- Optional file association ------------------------------------------------
Section "Associate image files (PNG, JPEG, BMP, WebP, GIF)" SecAssoc
  ; Register as a capable opener for each type (non-destructive: it adds the app
  ; to "Open with" and "Default apps" without hijacking an existing default).
  !macro Assoc EXT
    WriteRegStr HKLM "Software\Classes\${EXT}\OpenWithProgids" "${PROGID}" ""
    WriteRegStr HKLM "${CAPKEY}\FileAssociations" "${EXT}" "${PROGID}"
  !macroend
  !insertmacro Assoc ".png"
  !insertmacro Assoc ".jpg"
  !insertmacro Assoc ".jpeg"
  !insertmacro Assoc ".bmp"
  !insertmacro Assoc ".webp"
  !insertmacro Assoc ".gif"

  ; Tell the shell associations changed.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd

LangString DESC_SecCore  ${LANG_ENGLISH} "The application and a Start-menu shortcut."
LangString DESC_SecAssoc ${LANG_ENGLISH} "Make Rust Viewer Pro available as a default image viewer. You can confirm the default in Windows Settings > Default apps."
LangString DESC_SecCore  ${LANG_FRENCH}  "L'application et un raccourci dans le menu Démarrer."
LangString DESC_SecAssoc ${LANG_FRENCH}  "Proposer Rust Viewer Pro comme visionneuse d'images par défaut. Le choix final se confirme dans Paramètres Windows > Applications par défaut."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecCore}  $(DESC_SecCore)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecAssoc} $(DESC_SecAssoc)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; --- Uninstall ----------------------------------------------------------------
Section "Uninstall"
  Delete "$INSTDIR\${EXENAME}"
  Delete "$INSTDIR\icon-256.ico"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk"
  RMDir "$SMPROGRAMS\${APPNAME}"

  DeleteRegKey HKLM "Software\Classes\${PROGID}"
  DeleteRegValue HKLM "Software\RegisteredApplications" "${APPNAME}"
  DeleteRegKey HKLM "Software\RustViewerPro"
  DeleteRegKey HKLM "Software\${APPNAME}"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"

  ; Remove our OpenWithProgids entries.
  DeleteRegValue HKLM "Software\Classes\.png\OpenWithProgids"  "${PROGID}"
  DeleteRegValue HKLM "Software\Classes\.jpg\OpenWithProgids"  "${PROGID}"
  DeleteRegValue HKLM "Software\Classes\.jpeg\OpenWithProgids" "${PROGID}"
  DeleteRegValue HKLM "Software\Classes\.bmp\OpenWithProgids"  "${PROGID}"
  DeleteRegValue HKLM "Software\Classes\.webp\OpenWithProgids" "${PROGID}"
  DeleteRegValue HKLM "Software\Classes\.gif\OpenWithProgids"  "${PROGID}"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, i 0, i 0)'
SectionEnd
