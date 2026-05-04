# Copyright (c) 2025 Jonathon Burnham Cobb
# Licensed under the MIT-0 license.

Unicode true
SetCompressor lzma

!define COPYRIGHT "Copyright (c) 2025 ${AUTHOR}."

!include "LogicLib.nsh"

!define UNINSTALLER_NAME "$InstDir\uninstall.exe"

# 
# LANGUAGE STRINGS
# 

LoadLanguageFile "${NSISDIR}\Contrib\Language files\English.nlf"
LangString VersionText ${LANG_ENGLISH} "Version ${VERSION}"
SetFont /LANG=${LANG_ENGLISH} "Segoe UI" 9

# 
# INSTALLER ATTRIBUTES
# 

InstallDir "$LOCALAPPDATA\Programs\${PKG_NAME}"
InstallDirRegKey HKCU "Software\${AUTHOR}\${PKG_NAME}\Setup" "InstallDir"
RequestExecutionLevel user
XPStyle on
InstallColors /windows
InstProgressFlags smooth
Icon "${ROOT}\assets\doodle.ico"
UninstallIcon "${NSISDIR}\Contrib\Graphics\Icons\orange-uninstall.ico"
BrandingText $(VersionText)
ManifestSupportedOS Win10
ManifestDPIAware true
; WindowIcon off
LicenseData "${ROOT}\src\package\License-nowrap.txt"
OutFile "${ROOT}\dist\${PKG_NAME}-${CONFIG}-${VERSION}.exe"

VIProductVersion "${VERSION}.0"
VIFileVersion "${VERSION}.0"

VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${PKG_NAME}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${AUTHOR}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${PKG_NAME} Installer"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${VERSION}.0"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${VERSION}.0"

Page license
Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

# 
# INSTALLER SECTIONS
# 

Section
	# Clean, if needed
	${If} ${FileExists} "$INSTDIR\${BIN_NAME}"
		RMDir /r $INSTDIR
	${EndIf}

	# Install Binaries
	SetOverwrite on
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

	RMDir /r $INSTDIR
	RMDir /r "$LOCALAPPDATA\${AUTHOR}\${PKG_NAME}"

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
