# Wi-Fi Educamadrid (`educamadrid-wifi`)

Pequeña aplicación gráfica para Linux que configura y conecta la red Wi-Fi del profesorado
**`WEDU_PROF`** de los centros educativos de la Comunidad de Madrid (proyecto *Escuelas
Conectadas*), usando tu cuenta de EducaMadrid.

> **Herramienta no oficial.** No está hecha ni respaldada por EducaMadrid, Madrid Digital ni
> la Comunidad de Madrid. Se limita a aplicar en NetworkManager la configuración que publica
> su guía de usuario.

Configurar `WEDU_PROF` a mano en Linux es engorroso (WPA2-Enterprise, TTLS, PAP…). Con esta
app escribes usuario y contraseña, pulsas un botón y listo.

## Qué hace

1. Te pide el usuario de EducaMadrid (lo de antes de `@educa.madrid.org`; si pegas el correo
   entero, lo recorta) y la contraseña.
2. Habla con **NetworkManager por D-Bus** (no usa `nmcli`, así la contraseña nunca aparece en
   la lista de procesos) y crea un perfil llamado `WEDU_PROF` con:
   - `key-mgmt=wpa-eap`, `802-1x.eap=ttls`, `802-1x.phase2-auth=pap`;
   - `connection.permissions=user:<tu usuario de Linux>`: el perfil es **tuyo**; otras cuentas
     del mismo ordenador no pueden usarlo para conectarse con tus credenciales.
3. El perfil nuevo se crea **solo en memoria** y se activa. Espera hasta ~30 s:
   - **Si conecta**, lo guarda en disco y borra los perfiles `WEDU_PROF` antiguos.
   - **Si falla**, descarta el perfil nuevo y deja intactos los que ya tuvieras. Muestra el motivo
     que da NetworkManager (contraseña rechazada, red no encontrada, sin IP…).
4. Si al abrirla el equipo ya tiene `WEDU_PROF` configurado, lo avisa: solo hace falta volver a
   conectar si has cambiado la contraseña.
5. Tras conectar, vacía el campo de contraseña. Desde entonces el equipo se conecta solo a
   `WEDU_PROF` (también desde la bandeja del escritorio), sin abrir la app.

## Seguridad: léelo antes de usarla

**La app no verifica la identidad del servidor de la red.** La guía oficial de EducaMadrid
(*Guía de usuario de acceso a la nueva wifi – Escuelas Conectadas*, v2.9 y v2.13) indica
expresamente desmarcar «Verificar la identidad del servidor» / marcar «No se necesita ningún
certificado CA» y dejar el dominio vacío, y no publica ningún certificado ni nombre de servidor.
La app hace lo mismo.

Qué implica: alguien con un punto de acceso falso llamado `WEDU_PROF` al alcance de tu
dispositivo podría recibir tu usuario y tu contraseña de EducaMadrid (con PAP viajan en claro
dentro del túnel TLS). Es el mismo riesgo que tiene cualquier dispositivo configurado según la
guía oficial (Windows, Android, macOS…). En `ROADMAP.md` está anotado cómo cerrarlo si llega a
conocerse el certificado real del servidor.

**Dónde queda la contraseña.** NetworkManager la guarda en
`/etc/NetworkManager/system-connections/WEDU_PROF.nmconnection`. Ese fichero solo lo puede
leer `root`. A través de NetworkManager también puede leerla el usuario dueño del perfil
(p. ej. `nmcli --show-secrets connection show WEDU_PROF`). No se guarda en ningún otro sitio.

## Instalación

### Arch Linux (AUR)

```bash
yay -S educamadrid-wifi      # o paru, o makepkg a mano
```

Instala el binario en `/usr/bin/educamadrid-wifi` y un lanzador («Wi-Fi Educamadrid») en el
menú de aplicaciones.

### Desde el código (cualquier distribución)

Requisitos:

- **Rust ≥ 1.88** (`rustup` o el paquete `rust` de tu distribución).
- **NetworkManager** en marcha. Casi todas las distribuciones de escritorio lo traen. La app no
  necesita `nmcli`.
- Librerías gráficas habituales de un escritorio (OpenGL, Wayland o X11, xkbcommon).

```bash
git clone https://github.com/13Stokes31/educamadrid_wifi.git
cd educamadrid_wifi
cargo build --release
./target/release/educamadrid-wifi
```

Para tenerla en el menú de aplicaciones:

```bash
sudo install -Dm755 target/release/educamadrid-wifi /usr/local/bin/educamadrid-wifi
sudo install -Dm644 educamadrid-wifi.desktop /usr/local/share/applications/educamadrid-wifi.desktop
```

## Uso

1. Ve al centro (la red `WEDU_PROF` tiene que estar al alcance: la app no busca redes).
2. Abre «Wi-Fi Educamadrid», escribe usuario y contraseña de EducaMadrid y pulsa
   **CONECTAR** (o Intro).
3. Verde = conectado y guardado. Rojo = no se pudo; el mensaje dice por qué y tu configuración
   anterior sigue como estaba.

## Cómo está hecho

Un único fichero, `src/main.rs`:

| Función | Qué hace |
|---|---|
| `WeduApp` / `update` | Interfaz (egui/eframe; sigue el tema claro/oscuro del sistema). La conexión va en un hilo aparte para no congelar la ventana. |
| `normalizar_usuario` | Quita espacios y el `@educa.madrid.org`; rechaza lo que no parezca un usuario. |
| `perfil` | Construye los ajustes del perfil. **Si EducaMadrid cambia los parámetros de la red, se tocan aquí.** |
| `perfiles_wedu` | Lista los perfiles `WEDU_PROF` visibles para el usuario (aviso al abrir y limpieza tras conectar). |
| `connect_to_wedu` | Crea en memoria + activa (`AddAndActivateConnection2`); si conecta, guarda en disco (`Update2`) y borra perfiles viejos. |
| `wifi_device` | Elige la primera tarjeta Wi-Fi encendida. |
| `esperar_activacion` / `explicar_motivo` | Sondea el estado y traduce el motivo de fallo de NetworkManager. |

Tests automáticos: `cargo test` (normalización del usuario y contenido del perfil). La parte
que habla con la red real se comprueba a mano: ver `TESTING.md`.

## Publicar una versión (mantenimiento)

1. Subir `version` en `Cargo.toml` y `pkgver` en `packaging/aur/PKGBUILD` (y `pkgrel=1`).
2. `cargo test && cargo clippy --all-targets -- -D warnings`, commit.
3. Etiqueta y subida: `git tag vX.Y.Z && git push origin master vX.Y.Z` y crear la *release* en GitHub.
4. En `packaging/aur/`: `updpkgsums && makepkg -f && makepkg --printsrcinfo > .SRCINFO`.
5. Copiar `PKGBUILD` y `.SRCINFO` al clon del repo AUR
   (`ssh://aur@aur.archlinux.org/educamadrid-wifi.git`), commit y push. AUR solo acepta la rama
   `master`.

## Documentos del proyecto

- `ROADMAP.md`: problemas conocidos y mejoras pendientes.
- `TESTING.md`: comprobaciones manuales pendientes (las que exigen estar en el centro).

## Licencia

MIT. Ver `LICENSE`.
