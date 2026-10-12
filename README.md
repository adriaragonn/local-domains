# Local Domains

Gestiona tus dominios locales en Windows sin pelearte con el archivo `hosts` ni con el UAC.

Crea `mi-app.test → 127.0.0.1`, actívalo con un click y la app lo escribe en tu `hosts` usando un helper elevado. Al cerrar, la ventana se minimiza al tray.

## Descargar

Ve a [Releases](../../releases) y baja:

- **Portable (recomendado para probar):** `LocalDomains-portable-X.Y.Z.zip`
  1. Descomprime.
  2. Clic derecho en `hosts-helper.exe` → *Ejecutar como administrador* (déjalo abierto).
  3. Abre `LocalDomains.exe`.
- **Instalador:** `.exe` NSIS (requiere admin, registra el helper como servicio `LocalDomainsHelper`).

> Sin firma de código Windows mostrará SmartScreen → *Más información → Ejecutar de todas formas*.

## Por qué existe

Editar `C:\Windows\System32\drivers\etc\hosts` a mano es lento y peligroso: necesitas Notepad como admin, no hay validación, no hay proyectos, no hay ON/OFF rápido.

Local Domains lo reduce a: **añadir → ON → abrir en el navegador.**

## Funciones

- CRUD de dominios con validación (hostname + IPv4/IPv6)
- ON/OFF individual, activar/desactivar todos
- Proyectos (`tienda`, `cliente-x`): filtra y activa un proyecto entero de golpe
- Buscador por hostname, IP o proyecto
- Abrir en navegador, copiar hostname
- Import/export JSON (la importación entra desactivada por seguridad)
- Limpieza de caché DNS con un botón
- Copia de seguridad automática (`hosts.ldm.bak`) antes de cada escritura
- Bloque propio en el hosts (`# LOCAL-DOMAINS-MANAGER … # END-…`), no toca el resto
- Tray: cerrar minimiza, `Abrir / Salir` desde el icono
- Soporte de ruta hosts en Windows (`System32\drivers\etc\hosts`) y Unix (`/etc/hosts`)
- Aviso si el helper elevado no está corriendo

## Cómo funciona

- UI: Tauri + TypeScript (`src/`)
- Estado: `domains.json` en el `app_data_dir` (`src-tauri/src/repositories/`)
- Escritura: `HostsManager` solo toca su bloque (`src-tauri/src/hosts/manager.rs`)
- Permisos: `hosts-helper` (servicio/consola) escucha en un named pipe y ejecuta `enable / disable / flush` con privilegios (`src-tauri/src/bin/hosts-helper.rs`, cliente en `src-tauri/src/helper/client.rs`)

Protocolo del helper: `PING`, `ENABLE|ip|host`, `DISABLE|host`, `FLUSH` → responde `OK|…` o `ERROR|…`.

## Desarrollo

```powershell
# helper (consola, como admin para probar escritura real)
cargo run --manifest-path src-tauri/Cargo.toml --bin hosts-helper

# app
npm install
npm run tauri dev
```

## Build release

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
```

Compila el helper, lo copia a `src-tauri/bin/` (empaquetado como resource en `tauri.conf.json`) y lanza `tauri build`.

> Para distribuir fuera de tu máquina: firma el instalador NSIS y configura el updater de Tauri. Sin firma, Windows SmartScreen avisará.

## Modelo de datos

```json
[{ "id": "uuid", "hostname": "mi-app.test", "ip": "127.0.0.1", "enabled": false, "group": "tienda" }]
```

`group` es opcional y tiene `default ""` para migrar ficheros antiguos sin romper.
