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

    [Parameter(HelpMessage = "If present, build zip archive.")]
    [switch] $Zip,

    [Parameter(HelpMessage = "If present, build standalone installer.")]
    [switch] $Installer,

    [Parameter(HelpMessage = "If present, build MSIX package.")]
    [switch] $MSIX,

    [Parameter(HelpMessage = "The code signing certificate used to sign both the binaries and the installer. If not provided, the package will not be signed. Use the accompanying New-DevSigningCertificate.ps1 script to create a new code signing certificate.")]
    $Certificate = $null,

    [Parameter(HelpMessage = "Timestamp server URL to use when signing.")]
    $TimestampServer = "http://timestamp.digicert.com"
)

# Produces and MSIX app manifest based on a Cargo manifest and a signing certificate.
function New-AppxManifest($cargo, $certificate) {
    $id = "$($cargo.package.authors[0].Replace(' ', '')).$($cargo.package.description.Replace(' ', ''))"


    $appx = [System.Text.StringBuilder]::new()

    [void] $appx.AppendLine("<?xml version='1.0' encoding='utf-8'?>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("<Package xmlns='http://schemas.microsoft.com/appx/manifest/foundation/windows10'")
    [void] $appx.AppendLine("         xmlns:uap='http://schemas.microsoft.com/appx/manifest/uap/windows10'")
    [void] $appx.AppendLine("         xmlns:uap5='http://schemas.microsoft.com/appx/manifest/uap/windows10/5'")
    [void] $appx.AppendLine("         xmlns:uap10='http://schemas.microsoft.com/appx/manifest/uap/windows10/10'")
    [void] $appx.AppendLine("         xmlns:desktop4='http://schemas.microsoft.com/appx/manifest/desktop/windows10/4'")
    [void] $appx.AppendLine("         xmlns:rescap='http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities'")
    [void] $appx.AppendLine("         IgnorableNamespaces='uap uap5 uap10 desktop4 rescap'>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Identity Name='$id'")
    [void] $appx.AppendLine("            Version='$($cargo.package.version).0'")
    [void] $appx.AppendLine("            Publisher='$($certificate.Subject)'")
    [void] $appx.AppendLine("            ProcessorArchitecture='x64' />")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Properties>")
    [void] $appx.AppendLine("    <DisplayName>$($cargo.package.description)</DisplayName>")
    [void] $appx.AppendLine("    <PublisherDisplayName>$($cargo.package.authors[0])</PublisherDisplayName>")
    [void] $appx.AppendLine("    <Logo>Logo.png</Logo>")
    [void] $appx.AppendLine("  </Properties>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Resources>")
    [void] $appx.AppendLine("    <Resource Language='en-us' />")
    [void] $appx.AppendLine("  </Resources>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Dependencies>")
    [void] $appx.AppendLine("    <TargetDeviceFamily Name='Windows.Desktop'")
    [void] $appx.AppendLine("                        MinVersion='10.0.19041.0'")
    [void] $appx.AppendLine("                        MaxVersionTested='10.0.26100.0' />")
    [void] $appx.AppendLine("  </Dependencies>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Capabilities>")
    [void] $appx.AppendLine("    <rescap:Capability Name='runFullTrust' />")
    [void] $appx.AppendLine("  </Capabilities>")
    [void] $appx.AppendLine()
    [void] $appx.AppendLine("  <Applications>")

    foreach ($bin in $cargo.bin) {
        [void] $appx.AppendLine("    <Application Id='$($bin.name)'")
        [void] $appx.AppendLine("                 Executable='$($bin.name).exe'")
        [void] $appx.AppendLine("                 desktop4:Subsystem='console'")
        [void] $appx.AppendLine("                 desktop4:SupportsMultipleInstances='true'")
        [void] $appx.AppendLine("                 uap10:Subsystem='console'")
        [void] $appx.AppendLine("                 uap10:SupportsMultipleInstances='true'")
        [void] $appx.AppendLine("                 uap10:TrustLevel='mediumIL'")
        [void] $appx.AppendLine("                 uap10:RuntimeBehavior='win32App'>")
        [void] $appx.AppendLine("      <uap:VisualElements DisplayName='$($bin.name)'")
        [void] $appx.AppendLine("                          Description='$($cargo.package.description)'")
        [void] $appx.AppendLine("                          BackgroundColor='transparent'")
        [void] $appx.AppendLine("                          Square150x150Logo='Logo.png'")
        [void] $appx.AppendLine("                          Square44x44Logo='Logo.png' />")
        [void] $appx.AppendLine("      <Extensions>")
        [void] $appx.AppendLine("        <uap5:Extension Category='windows.appExecutionAlias'>")
        [void] $appx.AppendLine("          <uap5:AppExecutionAlias desktop4:Subsystem='console'")
        [void] $appx.AppendLine("                                  uap10:Subsystem='console'>")
        [void] $appx.AppendLine("            <uap5:ExecutionAlias Alias='$($bin.name).exe' />")
        [void] $appx.AppendLine("          </uap5:AppExecutionAlias>")
        [void] $appx.AppendLine("        </uap5:Extension>")
        [void] $appx.AppendLine("      </Extensions>")
        [void] $appx.AppendLine("    </Application>")
    }

    [void] $appx.AppendLine("  </Applications>")
    [void] $appx.AppendLine("</Package>")

    return $appx.ToString()
}

$ErrorActionPreference = "Stop"
try {
    Import-Module -Name PSToml
}
catch {
    Write-Error "PSToml module not found. Install it with 'Install-Module -Name PSToml'."
    exit 1
}

$config = if ($Release) { "release" } else { "debug" }

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

Write-Host -ForegroundColor Cyan "====== Build $packageName version $version ======"
Write-Host

Write-Host -ForegroundColor Blue "  Display Name: $displayName"
Write-Host -ForegroundColor Blue "  Author: $author"

# Update version in Cargo.toml.
# $cargo = $cargo -replace "^version = ""(.*)""", "version = ""$version"""
# $cargo | Set-Content -Path "Cargo.toml"

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

if (-not (Test-Path ".\dist")) {
    New-Item -ItemType Directory -Path "dist"
}

# Export certificate, if provided.
if ($Certificate) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Exporting $($certificate.Thumbprint) ======"
    Write-Host

    Export-Certificate -Cert $Certificate -FilePath "dist\$($certificate.Thumbprint).cer" -Type CERT -Force
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
if ($Zip) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Creating archive ======"
    Write-Host

    if (Test-Path "dist\$packageFilename.zip") {
        Remove-Item "dist\$packageFilename.zip" -Force
    }

    Compress-Archive `
        -Path "target\$config\*.exe", "License.txt" `
        -DestinationPath "dist\$packageFilename.zip" `
        -CompressionLevel Optimal `
        -Force
}

# Build the standalone installer.
if ($Installer) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Building standalone installer ======"
    Write-Host

    if (-not (Test-Path "dist")) {
        New-Item -ItemType Directory -Path "dist"
    }

    makensis `
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
        $targetBin = "dist\$packageFilename.exe"
        Write-Host
        Write-Host -ForegroundColor Cyan "====== Signing $targetBin ======"
        Write-Host

        Set-AuthenticodeSignature `
            -FilePath $targetBin `
            -Certificate $Certificate `
            -TimestampServer $TimestampServer
    }
}

# Build the MSIX package.
if ($MSIX) {
    Write-Host
    Write-Host -ForegroundColor Cyan "====== Building MSIX package ======"
    Write-Host

    if (-not (Test-Path "target\package")) {
        New-Item -ItemType Directory -Path "target\package"
    }

    Copy-Item -Path "assets\doodle.png" -Destination "target\package\Logo.png" -Force
    $manifest = New-AppxManifest -cargo $cargo -certificate $Certificate
    $manifest | Set-Content -Path "target\package\AppxManifest.xml" -Encoding utf8

    $mapping = [System.Text.StringBuilder]::new()
    $mapping.AppendLine("[Files]")
    $mapping.AppendLine("`"target\package\AppxManifest.xml`"`t`"AppxManifest.xml`"")
    $mapping.AppendLine("`"target\package\Logo.png`"`t`"Logo.png`"")
    foreach ($bin in $cargo.bin) {
        $mapping.AppendLine("`"target\$config\$($bin.name).exe`"`t`"$($bin.name).exe`"")
    }
    $mapping.AppendLine("`"License.txt`"`t`"License.txt`"")
    $mapping.ToString() | Set-Content -Path "target\package\mapping.txt"

    MakeAppx.exe pack `
        -f "target\package\mapping.txt" `
        -p "dist\$packageFilename.msix" `
        -o

    if ($LASTEXITCODE -eq 0 -and $Certificate) {
        $targetMsix = "dist\$packageFilename.msix"
        Write-Host
        Write-Host -ForegroundColor Cyan "====== Signing MSIX package ======"
        Write-Host

        $signTool = Get-Command -Name "signtool.exe" -ErrorAction SilentlyContinue
        if (-not $signTool) {
            Write-Error "signtool.exe was not found in PATH. Install Windows SDK and ensure signtool.exe is available."
            exit 1
        }

        & $signTool.Source sign `
            /fd SHA256 `
            /sha1 $Certificate.Thumbprint `
            /tr $TimestampServer `
            /td SHA256 `
            /v `
            $targetMsix

        if ($LASTEXITCODE -ne 0) {
            Write-Error "SignTool failed to sign $targetMsix"
            exit $LASTEXITCODE
        }
    }
}
