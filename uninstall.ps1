# VibeAnvil Uninstall Script (PowerShell)
# Usage: irm https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.ps1 | iex

$ErrorActionPreference = 'Stop'

$InstallDir = "$HOME\.local\bin"

# Colors
$Blue = "`e[34m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Red = "`e[31m"
$Reset = "`e[0m"

function Write-InfoMessage($Message) { Write-Host "${Blue}→${Reset} $Message" }
function Write-SuccessMessage($Message) { Write-Host "${Green}✓${Reset} $Message" }
function Write-WarningMessage($Message) { Write-Host "${Yellow}⚠${Reset} $Message" }
function Write-ErrorMessage($Message) { Write-Host "${Red}✗${Reset} $Message"; exit 1 }

Write-Host ""
Write-Host "${Red}╔═══════════════════════════════════════════╗${Reset}"
Write-Host "${Red}║        🗑️  VibeAnvil Uninstaller           ║${Reset}"
Write-Host "${Red}╚═══════════════════════════════════════════╝${Reset}"
Write-Host ""

$InstallPath = Join-Path $InstallDir "vibeanvil.exe"
$BackupPath = Join-Path $InstallDir "vibeanvil.exe.bak"

# Remove main binary
if (Test-Path $InstallPath) {
    Remove-Item -Force $InstallPath
    Write-SuccessMessage "Removed: $InstallPath"
}
else {
    Write-WarningMessage "Binary not found at: $InstallPath"
}

# Remove backup if exists
if (Test-Path $BackupPath) {
    Remove-Item -Force $BackupPath
    Write-SuccessMessage "Removed backup: $BackupPath"
}

# Remove workspace (optional)
$WorkspaceDir = ".vibeanvil"
if (Test-Path $WorkspaceDir) {
    Write-Host ""
    $Confirm = Read-Host "Remove workspace directory (.vibeanvil)? [y/N]"
    if ($Confirm -eq "y" -or $Confirm -eq "Y") {
        Remove-Item -Recurse -Force $WorkspaceDir
        Write-SuccessMessage "Removed workspace: $WorkspaceDir"
    }
    else {
        Write-InfoMessage "Keeping workspace"
    }
}

# Remove global data
$GlobalDir = "$HOME\.vibeanvil"
if (Test-Path $GlobalDir) {
    Write-Host ""
    $Confirm = Read-Host "Remove global data (~/.vibeanvil)? [y/N]"
    if ($Confirm -eq "y" -or $Confirm -eq "Y") {
        Remove-Item -Recurse -Force $GlobalDir
        Write-SuccessMessage "Removed global data: $GlobalDir"
    }
    else {
        Write-InfoMessage "Keeping global data (BrainPack, etc.)"
    }
}

Write-Host ""
Write-Host "${Green}╔═══════════════════════════════════════════╗${Reset}"
Write-Host "${Green}║     ✅ VibeAnvil uninstalled successfully  ║${Reset}"
Write-Host "${Green}╚═══════════════════════════════════════════╝${Reset}"
Write-Host ""
Write-Host "Thank you for using VibeAnvil! 👋"
Write-Host ""
