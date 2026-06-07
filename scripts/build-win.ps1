# Build Claude ZH Helper for Windows
param()

$ErrorActionPreference = "Stop"

Write-Host "🪟 Building Claude 中文助手 for Windows..." -ForegroundColor Cyan
Write-Host ""

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$HelperDir = Split-Path -Parent $ScriptDir
Set-Location $HelperDir

# Check prerequisites
function Check-Cmd {
  param($Name, $InstallHint)
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    Write-Host "❌ $Name is required but not installed." -ForegroundColor Red
    Write-Host "   Install: $InstallHint" -ForegroundColor Yellow
    exit 1
  }
}

Check-Cmd "node" "winget install OpenJS.NodeJS.LTS"
Check-Cmd "cargo" "winget install Rustlang.Rustup"

# Copy resources
Write-Host "📦 Copying resources..." -ForegroundColor Yellow
& "$ScriptDir\copy-resources.ps1"

# Install npm dependencies
Write-Host ""
Write-Host "📦 Installing npm dependencies..." -ForegroundColor Yellow
npm install

# Build for Windows
Write-Host ""
Write-Host "🔨 Building for Windows..." -ForegroundColor Yellow
npm run tauri:build

Write-Host ""
Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host "📂 Output: src-tauri/target/release/bundle/"

$bundlePath = Join-Path $HelperDir "src-tauri\target\release\bundle"
if (Test-Path $bundlePath) {
  Get-ChildItem $bundlePath -Recurse -File | Select-Object FullName, Length | Format-Table -AutoSize
}
