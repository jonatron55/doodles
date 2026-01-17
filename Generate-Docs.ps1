# generate-docs.ps1
# Generates markdown documentation for each binary from their --help output.

$ErrorActionPreference = "Stop"

# Helper function to format option headings
# Splits "-n, --iter <ITER>" into "`-n`, `--iter <ITER>`"
function Format-OptionHeading {
    param([string]$option)

    if ($option -match '^(-[a-zA-Z]),\s*(.+)$') {
        return "``$($Matches[1])``, ``$($Matches[2])``"
    }
    else {
        return "``$option``"
    }
}

# Helper function to format description text
# - Converts [text] to *text*
# - Converts 'option' to `option` (but not contractions)
function Format-Description {
    param([string]$text)

    # Convert [text] to *text* (italics)
    $text = $text -replace '\[([^\]]+)\]', '*$1*'

    # Convert 'option' to `option` where option starts with - or < (avoids contractions)
    # Matches '-x', '--option', '<VALUE>' style arguments
    $text = $text -replace "'(-[^']+)'", '`$1`'
    $text = $text -replace "'(<[^']+>)'", '`$1`'

    # Also convert 'word(...)' patterns (function-like syntax)
    $text = $text -replace "'([a-zA-Z_][a-zA-Z0-9_]*\([^']*\))'", '`$1`'

    return Format-Paragraph $text
}

function Format-Paragraph {
    param([string]$text)

    # Split into paragraphs (preserve blank lines as paragraph breaks)
    $paragraphs = $text -split '(\r?\n\s*\r?\n)'

    $result = $paragraphs | ForEach-Object {
        # If it's a blank line separator, keep it as a single blank line
        if ($_ -match '^\s*$') {
            ""
        }
        else {
            # Wrap non-empty paragraphs to 120 characters
            $wrapped = $_ -split '(.{1,120})(\s+|$)' | Where-Object { $_.Trim() -ne '' } | ForEach-Object { $_.Trim() }
            $wrapped -join "`n"
        }
    }

    return $result -join "`n"
}

# List of binaries to document
$binaries = @("doodle", "sorty", "conway", "digirain", "maze")

# Ensure docs directory exists
$docsDir = Join-Path $PSScriptRoot "docs"
if (-not (Test-Path $docsDir)) {
    New-Item -ItemType Directory -Path $docsDir | Out-Null
    Write-Host "Created docs directory: $docsDir"
}

foreach ($bin in $binaries) {
    Write-Host "Generating documentation for '$bin'..."

    # Run cargo to get help text
    $helpOutput = & cargo run --release --bin $bin -- --help 2>&1

    if ($LASTEXITCODE -ne 0) {
        Write-Warning "Failed to get help for '$bin': $helpOutput"
        continue
    }

    # Convert to string if it's an array, filtering out cargo build output lines
    $helpLines = $helpOutput | Where-Object {
        $_ -notmatch '^\s*(Compiling|Building|Finished|Running|Downloading|Downloaded)\s'
    }

    $helpText = $helpLines -join "`n"

    # Parse and format the help text into proper markdown
    $lines = $helpText -split "`n"
    $mdLines = @()
    $inOptions = $false
    $currentOption = $null
    $optionDescription = @()

    foreach ($line in $lines) {
        # Check for Usage: line
        if ($line -match '^Usage:\s*(.+)$') {
            $mdLines += ""
            $mdLines += "Usage"
            $mdLines += "-----"
            $mdLines += ""
            $mdLines += '```'
            $mdLines += $Matches[1]
            $mdLines += '```'
            continue
        }

        # Check for Options: header
        if ($line -match '^Options:\s*$') {
            $inOptions = $true
            $mdLines += ""
            $mdLines += "### Options ###"
            continue
        }

        if ($inOptions) {
            # Check for a new option line (starts with spaces then -)
            if ($line -match '^\s{1,4}(-[a-zA-Z], --[a-zA-Z][-a-zA-Z0-9]*(?:\s+<[^>]+>)?|--[a-zA-Z][-a-zA-Z0-9]*(?:\s+<[^>]+>)?|-[a-zA-Z](?:\s+<[^>]+>)?)') {
                # Flush previous option
                if ($currentOption) {
                    $mdLines += ""
                    # Format option: split at comma and wrap each part in backticks
                    $formattedOption = Format-OptionHeading $currentOption
                    $mdLines += "#### $formattedOption ####"
                    $mdLines += ""
                    $desc = ($optionDescription -join "`n").Trim()
                    if ($desc) {
                        $mdLines += Format-Description $desc
                    }
                }
                $currentOption = $Matches[1].Trim()
                $optionDescription = @()
            }
            # Description lines (indented more deeply)
            elseif ($line -match '^\s{6,}(.*)$' -or $line -match '^\s*$') {
                $text = if ($Matches[1]) { $Matches[1] } else { "" }
                $optionDescription += $text
            }
        }
        # First line is typically the description
        elseif ($mdLines.Count -eq 0 -and $line.Trim()) {
            $mdLines += "$(Format-Paragraph $line.Trim())"
        }
    }

    # Flush last option
    if ($currentOption) {
        $mdLines += ""
        # Format option: split at comma and wrap each part in backticks
        $formattedOption = Format-OptionHeading $currentOption
        $mdLines += "#### $formattedOption ####"
        $mdLines += ""
        $desc = ($optionDescription -join "`n").Trim()
        if ($desc) {
            $mdLines += Format-Description $desc
        }
    }

    # Create markdown content
    $mdContent = @"
<!-- Automatically generated by 'Generate-Docs.ps1' from '$bin --help' -->

``$bin``
$('=' * ($bin.Length + 2))

$($mdLines -join "`n")
"@

    # Write to file
    $mdPath = Join-Path $docsDir "$bin.md"
    Set-Content -Path $mdPath -Value $mdContent -Encoding UTF8
    Write-Host "  -> Written to $mdPath"
}
