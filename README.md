# Educamadrid Wi-Fi (`WeduConnectPro`)

Pequeña GUI para conectar fácilmente a la red Wi-Fi corporativa **`WEDU_PROF`** de Educamadrid (`@educa.madrid.org`). El conectarse a esa red desde Linux es engorroso porque usa autenticación EAP-TTLS / PAP — esta app te lo configura con un par de clics.

## Qué hace

1. Te pide usuario (sin `@educa.madrid.org`) y contraseña.
2. Lanza `nmcli` por debajo para:
   - Borrar el perfil `WEDU_PROF` si ya existía.
   - Crear uno nuevo con `wifi-sec.key-mgmt=wpa-eap`, `802-1x.eap=ttls`, `802-1x.phase2-auth=pap`.
   - Activarlo (`nmcli connection up WEDU_PROF`).
3. Muestra un indicador con el resultado (verde / rojo).

## Compilar

```bash
cargo build --release
```

Salida: `target/release/educamadrid_wifi` — binario autocontenido, ~10 MB.

## Requisitos del sistema

- `nmcli` (paquete `networkmanager` en Arch, casi siempre instalado).
- La red `WEDU_PROF` tiene que estar al alcance — la app no detecta SSIDs, solo crea el perfil.

## Uso

```bash
./target/release/educamadrid_wifi
```

Introduces el usuario (parte antes del `@`), tu contraseña, pulsas **CONECTAR AHORA** y listo.

## Archivos / inputs

No requiere ningún archivo externo. Toda la configuración va por la GUI.

## Notas

- El perfil queda guardado en NetworkManager con el nombre `WEDU_PROF`. Para conectarte la próxima vez puedes hacerlo directamente desde la bandeja de KDE sin abrir la app.
- Si Educamadrid cambia los parámetros EAP en algún momento, hay que ajustarlos en [`src/main.rs:181-200`](src/main.rs#L181-L200).
