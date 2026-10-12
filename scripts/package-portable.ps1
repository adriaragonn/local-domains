$ErrorActionPreference = "Stop"

# Empaqueta una versión portable (sin instalador ni servicio):
#   local-domains-portable-<version>.zip
# con el exe de la app + hosts-helper.exe + instrucciones.

$version = (Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json).version
$portableDir = "portable/LocalDomains-portable-$version"
$zipPath = "portable/LocalDomains-portable-$version.zip"

Write-Host "==> Compilando helper + app (release)..."

# Igual que en build.ps1: el build-script de Tauri exige que el
# resource bin/hosts-helper.exe exista antes de compilar.
$resourceDir = "src-tauri/bin"
$resourceExe = "$resourceDir/hosts-helper.exe"

if (!(Test-Path $resourceDir)) {
    New-Item -ItemType Directory -Path $resourceDir | Out-Null
}

if (!(Test-Path $resourceExe)) {
    Write-Host "==> Creando placeholder de hosts-helper.exe..."
    New-Item -ItemType File -Path $resourceExe -Force | Out-Null
}

Push-Location "src-tauri"
cargo build --release --bin hosts-helper
Pop-Location

$helperExe = "src-tauri/target/release/hosts-helper.exe"
if (!(Test-Path $helperExe)) {
    throw "No se encontró $helperExe"
}

# Copia el helper real como resource antes del `tauri build`.
Copy-Item $helperExe $resourceExe -Force

Write-Host "==> Compilando frontend + Tauri..."
npm run build
npx tauri build --bundles nsis

$appExe = "src-tauri/target/release/local-domains-manager.exe"
if (!(Test-Path $appExe)) {
    throw "No se encontró $appExe. Revisa el nombre del binario en src-tauri/Cargo.toml"
}

Write-Host "==> Preparando carpeta portable..."
if (Test-Path $portableDir) {
    Remove-Item $portableDir -Recurse -Force
}
New-Item -ItemType Directory -Path $portableDir | Out-Null

Copy-Item $appExe "$portableDir/LocalDomains.exe" -Force
Copy-Item $helperExe "$portableDir/hosts-helper.exe" -Force
Copy-Item "README.md" "$portableDir/README.md" -Force
Copy-Item "LICENSE" "$portableDir/LICENSE.txt" -Force

@'
LOCAL DOMAINS - VERSION PORTABLE
================================

1. Clic derecho en hosts-helper.exe > "Ejecutar como administrador".
   Déjalo abierto (es el que puede escribir en el archivo hosts).

2. Abre LocalDomains.exe y gestiona tus dominios.

3. Si la app dice "Helper no disponible", es que el paso 1 no está corriendo.

Para desinstalar: borra esta carpeta. El helper no instala nada.
'@ | Out-File -Encoding UTF8 "$portableDir/LEEME.txt"

Write-Host "==> Comprimiendo..."
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}
Compress-Archive -Path "$portableDir/*" -DestinationPath $zipPath

Write-Host ""
Write-Host "==> PORTABLE LISTO:"
Write-Host "    $zipPath"
Write-Host "    Instalador NSIS en: src-tauri/target/release/bundle/nsis/"
