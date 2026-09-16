# TESTING — comprobaciones manuales

Lo que ya está hecho pero hay que verificar a mano (no lo cubre `cargo test`). Marcar `[x]` al
comprobarlo.

## Comprobado ya en el PC de desarrollo (2026-09-16)

- [x] El perfil creado en memoria (`/run/NetworkManager/...`) pasa a `/etc/NetworkManager/...` con
      `Update2` y flag TO_DISK, conservando contraseña y `permissions` (perfil de prueba con otro
      nombre, sin activar, borrado después).
- [x] `makepkg` construye el paquete y pasa `check()`. Instala binario, `.desktop` y licencia.
- [x] La app arranca sin errores.
- [x] Interfaz revisada con capturas en tema oscuro y claro: formulario alineado, botón Ver/Ocultar,
      aviso de perfil existente, tarjetas de error/éxito/conectando (estados forzados en builds temporales).

## En el centro (red `WEDU_PROF` al alcance)

- [ ] **Conexión buena:** usuario y contraseña correctos → mensaje verde. `nmcli connection show`
      muestra un solo `WEDU_PROF` y hay navegación.
- [ ] **Queda guardado:** tras reiniciar el portátil, se conecta solo a `WEDU_PROF` sin abrir la app.
- [ ] **Correo entero:** escribir `usuario@educa.madrid.org` funciona igual que `usuario`.
- [ ] **Contraseña mal con perfil previo bueno:** con `WEDU_PROF` ya configurado, meter una contraseña
      errónea → mensaje rojo explicando el rechazo. Después, `nmcli connection show` sigue mostrando
      el perfil antiguo y se puede reconectar con él.
      Anotar si KDE/GNOME abre además su propio diálogo pidiendo la contraseña.
- [ ] **Perfil privado:** en el mismo equipo, desde otra cuenta de Linux, `WEDU_PROF` no aparece
      como utilizable (o no conecta).
- [ ] **Capturar el certificado del servidor** para el punto de seguridad del ROADMAP:
      `journalctl -b -u wpa_supplicant | grep EAP-PEER-CERT` y pegar sujeto, emisor y huella en
      ROADMAP.md.

## Fuera del centro

- [ ] **Red no disponible:** conectar en casa → mensaje «No se encuentra la red WEDU_PROF» (o el de
      tiempo agotado) y ningún `WEDU_PROF` nuevo en `nmcli connection show`.
- [ ] **Wi-Fi apagado:** con el Wi-Fi desactivado → «El Wi-Fi está apagado».
- [ ] **Lanzador:** tras instalar el paquete, «Wi-Fi Educamadrid» aparece en el menú con icono de
      Wi-Fi y la ventana se agrupa bien en la barra de tareas.

## Interfaz (uso normal)

- [ ] Al abrir, el cursor está en «Usuario» y se puede escribir sin tocar el ratón.
- [ ] Tab pasa a «Contraseña» e Intro conecta.
- [ ] Al cambiar el escritorio entre tema claro y oscuro, la app lo sigue (al reabrirla).
- [ ] En un equipo sin `WEDU_PROF` no aparece el aviso de «ya configurado»; en uno con el perfil, sí.

## Paquete AUR (tras publicar)

- [ ] `yay -S educamadrid-wifi` en un Arch limpio (o el PC de pruebas) instala y abre la app.
