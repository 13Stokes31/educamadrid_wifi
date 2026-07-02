# Educamadrid Wi-Fi (`WeduConnectPro`)

Pequeña GUI para conectar fácilmente a la red Wi-Fi corporativa **`WEDU_PROF`** de Educamadrid (`@educa.madrid.org`). El conectarse a esa red desde Linux es engorroso porque usa autenticación EAP-TTLS / PAP — esta app te lo configura con un par de clics.

## Qué hace

1. Te pide usuario (sin `@educa.madrid.org`) y contraseña.
2. Habla directamente con **NetworkManager por D-Bus** para:
   - Borrar el perfil `WEDU_PROF` si ya existía.
   - Crear uno nuevo con `key-mgmt=wpa-eap`, `802-1x.eap=ttls`, `802-1x.phase2-auth=pap`.
   - Activarlo y esperar hasta ~20 s a que la conexión quede establecida.
3. Muestra un indicador con el resultado (verde / rojo).

> **Seguridad:** la contraseña viaja **dentro de la llamada D-Bus**, no como argumento de
> `nmcli`. Así no aparece en la lista de procesos (`ps` / `/proc/<pid>/cmdline`), donde otro
> usuario de la máquina podría verla. (Antes se pasaba por línea de comandos a `nmcli`.)

## Compilar

```bash
cargo build --release
```

Salida: `target/release/educamadrid_wifi` — binario autocontenido, ~10 MB.

## Requisitos del sistema

- **NetworkManager** en marcha (paquete `networkmanager` en Arch, casi siempre instalado). La app le habla por D-Bus en el bus del sistema; no necesita el binario `nmcli`.
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
- Si Educamadrid cambia los parámetros EAP en algún momento, se ajustan en la función `connect_to_wedu` de `src/main.rs` (el diccionario `perfil`, sección `802-1x`).
