$ErrorActionPreference = "Stop"

Write-Host "==> Compilando hosts-helper..."

Push-Location "src-tauri"

cargo build --release --bin hosts-helper

Pop-Location

Write-Host "==> Copiando hosts-helper.exe..."

$source = "src-tauri/target/release/hosts-helper.exe"
$destinationDir = "src-tauri/bin"
$destination = "$destinationDir/hosts-helper.exe"

if (!(Test-Path $source)) {
    throw "No se encontró $source"
}

if (!(Test-Path $destinationDir)) {
    New-Item -ItemType Directory -Path $destinationDir | Out-Null
}

Copy-Item $source $destination -Force

Write-Host "==> Helper preparado."
Write-Host "    $destination"

Write-Host "==> Construyendo aplicación Tauri..."

npm run tauri build

Write-Host ""
Write-Host "==> BUILD COMPLETADO"