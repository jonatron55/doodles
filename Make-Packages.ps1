[CmdletBinding()]
param(
    [Parameter(HelpMessage = "If present, build release binaries instead of debug.")]
    [switch] $Release,

    [Parameter(HelpMessage = "If present, clean all build artifacts before building.")]
    [switch] $Clean,

    [Parameter(HelpMessage = "If present, increment the major version number.")]
    [switch] $MajorIncrement,

    [Parameter(HelpMessage = "If present, increment the minor version number.")]
    [switch] $MinorIncrement,

    [Parameter(HelpMessage = "If present, increment the patch version number.")]
    [switch] $PatchIncrement,

    [Parameter(HelpMessage = "The code signing certificate used to sign both the binaries and the installer. If not provided, the package will not be signed. Use the accompanying New-DevSigningCertificate.ps1 script to create a new code signing certificate.")]
    $Certificate = $null,

    [Parameter(HelpMessage = "Timestamp server URL to use when signing.")]
    $TimestampServer = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"
try {
    Import-Module -Name PSToml
}
catch {
    Write-Error "PSToml module not found. Install it with 'Install-Module -Name PSToml'."
    exit 1
}


# Read metadata from Cargo.toml.
$cargo = Get-Content -Path "Cargo.toml"
$cargo = ConvertFrom-Toml $cargo

$packageName = $cargo.package.name
$displayName = $cargo.package.description
$author = $cargo.package.authors[0]
$version = $cargo.package.version
$packageFilename = "$packageName-$config-$version"

# Increment version number.
$parts = $version.Split(".")

if ($MajorIncrement -and $MinorIncrement) {
    Write-Error "Cannot increment both major and minor version numbers."
    exit 1
}

if ($MajorIncrement) {
    $parts[0] = "$([int]$parts[0] + 1)"
    $parts[1] = "0"
    $parts[2] = "0"
}
elseif ($MinorIncrement) {
    $parts[1] = "$([int]$parts[1] + 1)"
    $parts[2] = "0"
}
elseif ($PatchIncrement) {
    $parts[2] = "$([int]$parts[2] + 1)"
}

$version = $parts -join '.'
$config = if ($Release) { "release" } else { "debug" }

Write-Host -ForegroundColor Cyan "====== Build $packageName version $version ======"
Write-Host

Write-Host -ForegroundColor Blue "  Display Name: $displayName"
Write-Host -ForegroundColor Blue "  Author: $author"

# Update version in Cargo.toml.
$cargo = $cargo -replace "^version = ""(.*)""", "version = ""$version"""
$cargo | Set-Content -Path "Cargo.toml"

# Clean build artifacts.
if ($Clean) {
    cargo.exe clean
}

# Build the binary.
if ($Release) {
    cargo.exe build --release
}
else {
    cargo.exe build
}

# Export certificate, if provided.
if ($Certificate) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Exporting $(certificate.Thumbprint) ======"
    Write-Host

    Export-Certificate -Cert $Certificate -FilePath "dist\$(certificate.Thumbprint).cer" -Type CERT -Force
}

# Sign the binary, if a certificate is provided.
$ErrorActionPreference = "Continue"

if ($Certificate) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Signing binaries ======"
    Write-Host

    foreach ($bin in $cargo.bin) {
        Set-AuthenticodeSignature `
            -FilePath "target\$config\$($bin.name).exe" `
            -Certificate $Certificate `
            -TimestampServer $TimestampServer
    }
}
else {
    Write-Host
    Write-Host -ForegroundColor Yellow "====== Skipping signing ======"
    Write-Host
}

# Create a plain zip archive.
Write-Host
Write-Host -ForegroundColor Cyan "====== Creating archive ======"
Write-Host

Compress-Archive `
    -Path "target\$config\*.exe", "License.txt" `
    -DestinationPath "$packageFilename.zip" `
    -CompressionLevel Optimal `
    -Force

if ((Test-Path "$packageFilename.zip") -and $Certificate) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Signing $packageFilename.zip ======"
    Write-Host

    Set-AuthenticodeSignature `
        -FilePath "$packageFilename.zip" `
        -Certificate $Certificate `
        -TimestampServer $TimestampServer
}

# Build the standalone installer.
Write-Host
Write-Host -ForegroundColor Cyan "====== Building standalone installer ======"
Write-Host

if (-not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist"
}

makensis.exe `
    -DBIN_NAME="$binName" `
    -DPKG_NAME="$packageName" `
    -DAUTHOR="$author" `
    -DVERSION="$version" `
    -DCONFIG="$config" `
    -DROOT="$(Get-Location)" `
    -X"Name `"$displayName`"" `
    "src\package\installer.nsi"


# Sign the installer, if a certificate is provided.
if ($LASTEXITCODE -eq 0 -and $Certificate) {
    Write-Error "Build failed."
    $targetBin = "dist\$packageFilename.exe"
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Signing $targetBin ======"
    Write-Host

    Set-AuthenticodeSignature `
        -FilePath $targetBin `
        -Certificate $Certificate `
        -TimestampServer $TimestampServer
}

# Build the MSIX package.
Write-Host
Write-Host -ForegroundColor Cyan "====== Building MSIX package ======"
Write-Host

$manifest = New-AppxManifest -cargo $cargo -certificate $Certificate
$manifest | Set-Content -Path "target\package\AppxManifest.xml" -Encoding utf8

$mapping = [System.Text.StringBuilder]::new()
$mapping.AppendLine("[Files]")
$mapping.AppendLine("`"target\package\AppxManifest.xml`"`t`"AppxManifest.xml`"")
foreach ($bin in $cargo.bin) {
    $mapping.AppendLine("`"target\$config\$($bin.name).exe`t`"$($bin.name).exe`"")
}
$mapping.AppendLine("`"License.txt`t`"License.txt`"")
$mapping.ToString() | Set-Content -Path "target\package\mapping.txt"

MakeAppx.exe pack `
    -f "target\package\mapping.txt" `
    -p "dist\$packageFilename.msix" `
    -o

if ($LASTEXITCODE -eq 0 -and $Certificate) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Signing MSIX package ======"
    Write-Host

    Set-AuthenticodeSignature `
        -FilePath "dist\$packageFilename.msix" `
        -Certificate $Certificate `
        -TimestampServer $TimestampServer
}

# Produces and MSIX app manifest based on a Cargo manifest and a signing certificate.
function New-AppxManifest($cargo, $certificate) {
    $id = "$($cargo.package.authors[0].Replace(' ', '')).$($cargo.description.Replace(' ', ''))"


    $appx = [System.Text.StringBuilder]::new()

    $appx.AppendLine("<?xml version='1.0' encoding='utf-8'?>")
    $appx.AppendLine()
    $appx.AppendLine("<Package xmlns='http://schemas.microsoft.com/appx/2010/manifest'")
    $appx.AppendLine("         xmlns:uap='http://schemas.microsoft.com/appx/manifest/uap/windows10'")
    $appx.AppendLine("         xmlns:uap5='http://schemas.microsoft.com/appx/manifest/uap/windows10/5'")
    $appx.AppendLine("         xmlns:uap10='http://schemas.microsoft.com/appx/manifest/uap/windows10/10'")
    $appx.AppendLine("         xmlns:desktop4='http://schemas.microsoft.com/appx/manifest/desktop/windows10/4'")
    $appx.AppendLine("         xmlns:rescap='http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities'")
    $appx.AppendLine("         ignorableNamespaces='uap'>")
    $appx.AppendLine()
    $appx.AppendLine("  <Identity Name='$id'")
    $appx.AppendLine("            Version='$($cargo.package.version).0'")
    $appx.AppendLine("            Publisher='$($certificate.Subject)'")
    $appx.AppendLine("            ProcessorArchitecture='x64' />")
    $appx.AppendLine()
    $appx.AppendLine("  <Properties>")
    $appx.AppendLine("    <DisplayName>$($cargo.package.description)</DisplayName>")
    $appx.AppendLine("    <PublisherDisplayName>$($cargo.package.authors[0])</PublisherDisplayName>")
    $appx.AppendLine("    <Logo></Logo>")
    $appx.AppendLine("  </Properties>")
    $appx.AppendLine()
    $appx.AppendLine("  <Resources>")
    $appx.AppendLine("    <Resource Language='en-us' />")
    $appx.AppendLine("  </Resources>")
    $appx.AppendLine()
    $appx.AppendLine("  <Dependencies>")
    $appx.AppendLine("    <TargetDeviceFamily Name='Windows.Desktop'")
    $appx.AppendLine("                        MinVersion='10.0.19041.0'")
    $appx.AppendLine("                        MaxVersionTested='10.0.26100.0' />")
    $appx.AppendLine("  </Dependencies>")
    $appx.AppendLine()
    $appx.AppendLine("  <Capabilities>")
    $appx.AppendLine("    <rescap:Capability Name='runFullTrust' />")
    $appx.AppendLine("  </Capabilities>")
    $appx.AppendLine()
    $appx.AppendLine("  <Applications>")

    foreach ($bin in $cargo.bin) {
        $appx.AppendLine("    <Application Id='$($bin.name)'")
        $appx.AppendLine("                 Executable='$($bin.name).exe'")
        $appx.AppendLine("                 desktop4:Subsystem='console'")
        $appx.AppendLine("                 uap10:Subsystem='console'")
        $appx.AppendLine("                 uap10:TrustLevel='mediumIL'")
        $appx.AppendLine("                 uap10:RuntimeBehavior='win32App'>")
        $appx.AppendLine("      <Extensions>")
        $appx.AppendLine("        <uap5:Extension>")
        $appx.AppendLine("          <uap5:AppExecutionAlias desktop4:Subsystem='console'")
        $appx.AppendLine("                                  uap10:Subsystem='console'>")
        $appx.AppendLine("            <uap5:ExecutionAlias Alias='$($bin.name)' />")
        $appx.AppendLine("          </uap5:AppExecutionAlias>")
        $appx.AppendLine("        </uap5:Extension>")
        $appx.AppendLine("      </Extensions>")
        $appx.AppendLine("    </Application>")
    }

    $appx.AppendLine("  </Applications>")
    $appx.AppendLine("</Package>")

    return $appx.ToString()
}
