# VibeAnvil Install Script (PowerShell)
# Usage: irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$Repo = "ThanhNguyxn/vibeanvil"
$InstallDir = "$HOME\.local\bin"
$Version = "latest"

# Colors (using [char]27 for PowerShell 5 compatibility)
$ESC = [char]27
$Blue = "$ESC[34m"
$Green = "$ESC[32m"
$Yellow = "$ESC[33m"
$Red = "$ESC[31m"
$Reset = "$ESC[0m"

function Write-InfoMessage($Message) { Write-Host "${Blue}→${Reset} $Message" }
function Write-SuccessMessage($Message) { Write-Host "${Green}✓${Reset} $Message" }
function Write-WarningMessage($Message) { Write-Host "${Yellow}⚠${Reset} $Message" }
function Write-ErrorMessage($Message) { Write-Host "${Red}✗${Reset} $Message"; exit 1 }

# Detect Architecture
$Arch = $env:PROCESSOR_ARCHITECTURE
if ($Arch -eq "AMD64") {
    $Platform = "windows-x64"
}
elseif ($Arch -eq "ARM64") {
    $Platform = "windows-arm64" # Note: Release workflow doesn't build this yet, but good to have
}
else {
    Write-ErrorMessage "Unsupported architecture: $Arch"
}

# Create Install Directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# Get Download URL
$AssetName = "vibeanvil-${Platform}.exe"
if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/$AssetName"
    $ChecksumUrl = "https://github.com/$Repo/releases/latest/download/checksums.txt"
}
else {
    $Url = "https://github.com/$Repo/releases/download/$Version/$AssetName"
    $ChecksumUrl = "https://github.com/$Repo/releases/download/$Version/checksums.txt"
}

# Download Binary
$TempFile = [System.IO.Path]::GetTempFileName()
Write-InfoMessage "Downloading from: $Url"
try {
    Invoke-WebRequest -Uri $Url -OutFile $TempFile
}
catch {
    Write-ErrorMessage "Failed to download binary. Check if the release exists."
}

# Verify Checksum
Write-InfoMessage "Verifying checksum..."
try {
    $Checksums = (Invoke-WebRequest -Uri $ChecksumUrl).Content
    $Expected = ($Checksums -split "`r`n" | Select-String $AssetName).ToString().Split(' ')[0]
    
    if ($Expected) {
        $Actual = (Get-FileHash $TempFile -Algorithm SHA256).Hash.ToLower()
        if ($Actual -ne $Expected) {
            Write-ErrorMessage "Checksum verification failed!`nExpected: $Expected`nActual:   $Actual"
        }
        Write-SuccessMessage "Checksum verified"
    }
    else {
        Write-WarningMessage "Checksum not found for $AssetName, skipping verification"
    }
}
catch {
    Write-WarningMessage "Could not fetch checksums, skipping verification"
}

# Install
$InstallPath = Join-Path $InstallDir "vibeanvil.exe"
Move-Item -Force -Path $TempFile -Destination $InstallPath

Write-SuccessMessage "Installed to: $InstallPath"

# Check PATH
if ($env:PATH -notlike "*$InstallDir*") {
    Write-Host ""
    Write-WarningMessage "Add this to your PowerShell profile:"
    Write-Host ""
    Write-Host "    `$env:PATH += `";$InstallDir`""
    Write-Host ""
}

# Verify
Write-Host ""
if (Get-Command vibeanvil -ErrorAction SilentlyContinue) {
    Write-SuccessMessage "Installation complete!"
    Write-Host ""
    vibeanvil --version
}
else {
    Write-SuccessMessage "Binary installed. Restart your shell or run:"
    Write-Host ""
    Write-Host "    `$env:PATH += `";$InstallDir`""
    Write-Host "    vibeanvil --version"
}
