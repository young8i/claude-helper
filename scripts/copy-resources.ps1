# Copy Chinese localization resources from parent project
param()

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$HelperDir = Split-Path -Parent $ScriptDir
$ParentDir = Split-Path -Parent $HelperDir
$ResourcesSrc = Join-Path $ParentDir "resources"
$ScriptsSrc = Join-Path $ParentDir "scripts"
$ResourcesDst = Join-Path $HelperDir "resources"
$ScriptsDst = Join-Path $HelperDir "scripts"

Write-Host "📦 Copying Chinese localization resources..."

# Copy resource files
if (Test-Path $ResourcesSrc) {
  New-Item -ItemType Directory -Path $ResourcesDst -Force | Out-Null
  Copy-Item (Join-Path $ResourcesSrc "*.json") $ResourcesDst -Force
  Copy-Item (Join-Path $ResourcesSrc "*.strings") $ResourcesDst -Force -ErrorAction SilentlyContinue
  Write-Host "✅ Resources copied to $ResourcesDst"
} else {
  Write-Host "⚠️  Resources not found at $ResourcesSrc"
}

# Copy scripts
if (Test-Path $ScriptsSrc) {
  New-Item -ItemType Directory -Path $ScriptsDst -Force | Out-Null
  Copy-Item (Join-Path $ScriptsSrc "*.py") $ScriptsDst -Force -ErrorAction SilentlyContinue
  Copy-Item (Join-Path $ScriptsSrc "*.ps1") $ScriptsDst -Force -ErrorAction SilentlyContinue
  Write-Host "✅ Scripts copied to $ScriptsDst"
} else {
  Write-Host "⚠️  Scripts not found at $ScriptsSrc"
}

# Copy install scripts from parent root
$installMac = Join-Path $ParentDir "install-mac.command"
if (Test-Path $installMac) {
  Copy-Item $installMac $ResourcesDst -Force
  Write-Host "✅ install-mac.command copied"
}
$installWin = Join-Path $ParentDir "install-windows.bat"
if (Test-Path $installWin) {
  Copy-Item $installWin $ResourcesDst -Force
  Write-Host "✅ install-windows.bat copied"
}

Write-Host ""
Write-Host "🎉 Resources ready!"
