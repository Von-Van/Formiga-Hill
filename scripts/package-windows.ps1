# Builds Formiga Hill for Windows, as a portable zip and a per-user installer that writes the
# registry values Formiga Desktop looks for (formiga-travel's discovery module names them).
param(
    [switch]$SkipInstaller
)

$ErrorActionPreference = "Stop"
$RepoDir = Split-Path -Parent $PSScriptRoot
$DistDir = Join-Path $RepoDir "dist"
$Exe = Join-Path $RepoDir "target\x86_64-pc-windows-msvc\release\formiga-hill.exe"
$Metadata = cargo metadata --no-deps --format-version 1 --manifest-path (Join-Path $RepoDir "Cargo.toml") | ConvertFrom-Json
$Version = if ($env:FORMIGA_HILL_VERSION) { $env:FORMIGA_HILL_VERSION } else { ($Metadata.packages | Where-Object name -eq "formiga-hill").version }
$Version = $Version.TrimStart("v")
$Portable = Join-Path $DistDir "Formiga-Hill-$Version-windows-x64.zip"
$Installer = Join-Path $DistDir "Formiga-Hill-$Version-windows-x64.msi"
$PortableDir = Join-Path $DistDir "Formiga-Hill-portable"

function Write-Checksum([string]$Path) {
    $Hash = (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLowerInvariant()
    $Name = Split-Path -Leaf $Path
    Set-Content -NoNewline -Path "$Path.sha256" -Value "$Hash  $Name`n"
}

Push-Location $RepoDir
try {
    rustup target add x86_64-pc-windows-msvc
    cargo build --release -p formiga-hill --target x86_64-pc-windows-msvc
    # The travel version comes from the binary itself, so the installer can never claim another.
    $TravelVersion = (& $Exe --travel-version).Trim()
    # And so does the icon, from the same picture as the window's.
    $IconDir = Join-Path $DistDir "icon"
    & $Exe --icon $IconDir | Out-Null
    $Icon = Join-Path $IconDir "FormigaHill.ico"
    if ($env:FORMIGA_SIGNTOOL_CERT_SHA1) {
        signtool sign /sha1 $env:FORMIGA_SIGNTOOL_CERT_SHA1 /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Exe
    }
    New-Item -ItemType Directory -Force $DistDir | Out-Null
    if (Test-Path $PortableDir) { Remove-Item -Recurse -Force $PortableDir }
    New-Item -ItemType Directory -Force $PortableDir | Out-Null
    Copy-Item $Exe (Join-Path $PortableDir "Formiga Hill.exe")
    Copy-Item (Join-Path $RepoDir "packaging\windows\README.txt") (Join-Path $PortableDir "Read Me.txt")
    Compress-Archive -Force -Path (Join-Path $PortableDir "*") -DestinationPath $Portable
    Write-Checksum $Portable

    if (-not $SkipInstaller) {
        if (-not (Get-Command wix -ErrorAction SilentlyContinue)) {
            throw "WiX 4 CLI is required for the MSI. Install it with: dotnet tool install --global wix --version 4.0.5"
        }
        wix build `
            -d "HillExe=$Exe" `
            -d "HillVersion=$Version" `
            -d "TravelVersion=$TravelVersion" `
            -d "HillIcon=$Icon" `
            -arch x64 `
            -o $Installer `
            (Join-Path $RepoDir "packaging\windows\FormigaHill.wxs")
        if ($env:FORMIGA_SIGNTOOL_CERT_SHA1) {
            signtool sign /sha1 $env:FORMIGA_SIGNTOOL_CERT_SHA1 /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Installer
        }
        Write-Checksum $Installer
    }
} finally {
    Pop-Location
}
