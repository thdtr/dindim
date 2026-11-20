# Installs dindim for the current user on Windows. No administrator rights needed.
#
#   irm https://raw.githubusercontent.com/thdtr/dindim/main/scripts/install.ps1 | iex
#
# To pick a version or uninstall, run it as a script block:
#   & ([scriptblock]::Create((irm https://raw.githubusercontent.com/thdtr/dindim/main/scripts/install.ps1))) -Uninstall

param(
    [string]$Version = $env:DINDIM_VERSION,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Repo = 'thdtr/dindim'
$InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\dindim'
$Shortcut = Join-Path ([Environment]::GetFolderPath('Programs')) 'dindim.lnk'

if ($Uninstall) {
    Get-Process dindim -ErrorAction SilentlyContinue | Stop-Process -Force
    Remove-Item $InstallDir -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $Shortcut -Force -ErrorAction SilentlyContinue
    Write-Host 'dindim was removed. Your data was not touched.'
    return
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'dindim needs 64-bit Windows.'
}

if (-not $Version) {
    $releases = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases?per_page=1"
    $Version = $releases[0].tag_name
}

$Asset = "dindim-$Version-windows-x86_64.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"
$Temp = Join-Path ([IO.Path]::GetTempPath()) ("dindim-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $Temp | Out-Null

try {
    Write-Host "Downloading dindim $Version..."
    $Zip = Join-Path $Temp $Asset
    Invoke-WebRequest $Url -OutFile $Zip -UseBasicParsing
    $Expected = ((Invoke-WebRequest "$Url.sha256" -UseBasicParsing).Content -split '\s+')[0]
    $Actual = (Get-FileHash $Zip -Algorithm SHA256).Hash
    if ($Expected -ne $Actual.ToLower()) {
        throw 'Checksum mismatch, the download may be corrupted.'
    }

    Get-Process dindim -ErrorAction SilentlyContinue | Stop-Process -Force
    Expand-Archive $Zip -DestinationPath $Temp -Force
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item (Join-Path $Temp "dindim-$Version-windows-x86_64\*") $InstallDir -Recurse -Force

    $Shell = New-Object -ComObject WScript.Shell
    $Link = $Shell.CreateShortcut($Shortcut)
    $Link.TargetPath = Join-Path $InstallDir 'dindim.exe'
    $Link.WorkingDirectory = $InstallDir
    $Link.Save()

    Write-Host "dindim $Version is installed. Find it in the Start menu."
}
finally {
    Remove-Item $Temp -Recurse -Force -ErrorAction SilentlyContinue
}
