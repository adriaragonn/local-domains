$ErrorActionPreference = "Stop"

# El build-script de Tauri valida que los `resources`
# (bin/hosts-helper.exe) existan al compilar el lib.
# En un clon limpio ese exe no existe (está en .gitignore),
# así que creamos un placeholder vacío antes de compilar
# el helper de verdad.

$destinationDir = "src-tauri/bin"
$destination = "$destinationDir/hosts-helper.exe"

if (!(Test-Path $destinationDir)) {
    New-Item -ItemType Directory -Path $destinationDir | Out-Null
}

if (!(Test-Path $destination)) {
    Write-Host "==> Creando placeholder de hosts-helper.exe..."
    New-Item -ItemType File -Path $destination -Force | Out-Null
}

Write-Host "==> Compilando hosts-helper..."

Push-Location "src-tauri"

cargo build --release --bin hosts-helper

Pop-Location

Write-Host "==> Copiando hosts-helper.exe..."

$source = "src-tauri/target/release/hosts-helper.exe"

if (!(Test-Path $source)) {
    throw "No se encontró $source"
}

Copy-Item $source $destination -Force

Write-Host "==> Helper preparado."
Write-Host "    $destination"

Write-Host "==> Construyendo aplicación Tauri..."

npm run tauri build

Write-Host ""
Write-Host "==> BUILD COMPLETADO"