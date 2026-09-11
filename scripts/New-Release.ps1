[CmdletBinding()]
param(
    [Parameter()]
    [ValidateSet("Major", "Minor", "Patch")]
    [string]$Type = "Patch"
)

$Root = Split-Path -Path $PSScriptRoot -Parent
Push-Location -Path $Root

try {
    $Status = git status --porcelain
    if ($Status) {
        if ($Host.UI.PromptForChoice(
                "Uncommitted changes",
                "There are uncommitted changes in the working directory. These changes will be included in the release commit. Do you want to continue?",
                @("&Continue", "&Abort"),
                1
            ) -eq 1) {
            exit
        }
    }

    Write-Host "Running cargo tests..."
    cargo test --locked

    if ($LASTEXITCODE -ne 0) {
        if ($Host.UI.PromptForChoice(
                "Tests Failed",
                "Cargo tests failed. Review the test output above to determine if the failures are acceptable. Do you want to proceed with failing tests?",
                @("&Continue", "&Abort"),
                1
            ) -eq 1) {
            exit
        }
    }

    $Toml = Get-Content -Path "Cargo.toml" -Raw
    $Version = ($Toml | Select-String -Pattern '(?m)^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"').Matches[0]

    $Major = [int]$Version.Groups[1].Value
    $Minor = [int]$Version.Groups[2].Value
    $Patch = [int]$Version.Groups[3].Value

    switch ($Type) {
        "Major" {
            $Major++
            $Minor = 0
            $Patch = 0
        }
        "Minor" {
            $Minor++
            $Patch = 0
        }
        "Patch" {
            $Patch++
        }
    }

    $NewVersion = "$Major.$Minor.$Patch"

    if ($Host.UI.PromptForChoice(
            "Ready to commit release",
            "The release is ready to be committed with version $NewVersion. Do you want to proceed?",
            @("&Continue", "&Abort"),
            0
        ) -eq 1) {
        exit
    }

    Set-Content -Path "Cargo.toml" -Value ($Toml -replace '(?m)^version\s*=\s*"\d+\.\d+\.\d+"', "version = `"$NewVersion`"").Trim() -Encoding Utf8NoBOM

    cargo update --workspace

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Cargo update failed."
        exit
    }

    git add .
    git commit -m "Bump version to $NewVersion"

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Git commit failed."
        exit
    }

    git tag -a "v$NewVersion" -m "Release version $NewVersion"

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Git tag failed."
        exit
    }

    git push origin main --follow-tags

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Git push failed."
        exit
    }
}
finally {
    Pop-Location
}
