# Copyright (c) 2025 Jonathon Burnham Cobb
# Licensed under the MIT-0 license.

Unicode True
SetCompressor LZMA

!Define COPYRIGHT "Copyright (c) 2025 ${AUTHOR}."

!Include "LogicLib.nsh"

!Define UNINSTALLER_NAME "$InstDir\uninstall.exe"

#
# LANGUAGE STRINGS
#

LoadLanguageFile "${NSISDIR}\Contrib\Language files\English.nlf"
LangString VersionText ${LANG_ENGLISH} "Version ${VERSION}"
SetFont /Lang=${LANG_ENGLISH} "Segoe UI" 9

#
# INSTALLER ATTRIBUTES
#

InstallDir "$LOCALAPPDATA\Programs\${PKG_NAME}"
InstallDirRegkey HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "InstallDir"
RequestExecutionLevel User
XPStyle On
InstallColors /Windows
InstProgressFlags smooth
Icon "${NSISDIR}\Contrib\Graphics\Icons\orange-install.ico"
UninstallIcon "${NSISDIR}\Contrib\Graphics\Icons\orange-uninstall.ico"
BrandingText $(VersionText)
ManifestSupportedOS Win10
ManifestDPIAware True
WindowIcon Off
LicenseData "${ROOT}\License.txt"
OutFile "${ROOT}\dist\${PKG_NAME}-${CONFIG}-${VERSION}.exe"

VIProductVersion "${VERSION}.0"
VIFileVersion "${VERSION}.0"

VIAddVersionKey /Lang=${LANG_ENGLISH} "ProductName" "${PKG_NAME}"
VIAddVersionKey /Lang=${LANG_ENGLISH} "CompanyName" "${AUTHOR}"
VIAddVersionKey /Lang=${LANG_ENGLISH} "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey /Lang=${LANG_ENGLISH} "FileDescription" "${PKG_NAME} Installer"
VIAddVersionKey /Lang=${LANG_ENGLISH} "ProductVersion" "${VERSION}.0"
VIAddVersionKey /Lang=${LANG_ENGLISH} "FileVersion" "${VERSION}.0"

Page License
Page Directory
Page InstFiles
UninstPage UninstConfirm
UninstPage InstFiles

#
# INSTALLER SECTIONS
#

Section
    # Clean, if needed
    ${If} ${FileExists} "$INSTDIR\${BIN_NAME}"
        RMDir /R $INSTDIR
    ${EndIf}

    # Install Binaries
    SetOverwrite On
    CreateDirectory $INSTDIR
    SetOutPath $INSTDIR

    File "${ROOT}\target\${CONFIG}\*.exe"
    File "${ROOT}\LICENSE.txt"

    # Update PATH
    EnVar::SetHKCU
    EnVar::AddValue "PATH" "$INSTDIR"

    # Register and create uninstaller
    WriteRegStr HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "InstallDir" $INSTDIR
    WriteRegStr HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "Version" ${VERSION}
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}" "DisplayName" "$(^Name)"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}" "UninstallString" '"${UNINSTALLER_NAME}"'
    WriteUninstaller ${UNINSTALLER_NAME}
SectionEnd

#
# UNINSTALLER SECTIONS
#

Section "Uninstall"
    # Remove files
    Delete "${UNINSTALLER_NAME}"

    RMDir /R $INSTDIR
    RMDir /R "$LOCALAPPDATA\${AUTHOR}\${PKG_NAME}"

    # Update PATH
    EnVar::SetHKCU
    EnVar::DeleteValue "PATH" "$INSTDIR" 

    # Remove uninstaller registration
    DeleteRegValue HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "InstallDir"
    DeleteRegValue HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "Version"
    DeleteRegKey HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup"
    DeleteRegKey HKCU "Software\${AUTHOR}\${PKG_NAME}"

    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}" "UninstallString"
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}" "DisplayName"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PKG_NAME}"
SectionEnd