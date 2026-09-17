# ROADMAP

Registro único de problemas conocidos y mejoras pendientes. Lo ya hecho que falta comprobar a
mano está en `TESTING.md`.

Origen: auditoría externa del 2026-08-28 (antiguo `CHATGPT.md`) y revisión del 2026-09-16.

## Seguridad

### Validar la identidad del servidor RADIUS
- **Qué:** el perfil no fija CA ni `domain-suffix-match`, así que un punto de acceso falso
  `WEDU_PROF` podría llevarse las credenciales (PAP dentro de TTLS).
- **Por qué no está hecho:** la guía oficial (v2.9 y v2.13) manda no validar el certificado y no
  publica CA ni dominio. Inventarlos rompería la conexión.
- **Cómo avanzar:** en el centro, conectar una vez y mirar el certificado que presenta el servidor:
  `journalctl -b -u wpa_supplicant | grep EAP-PEER-CERT` (sujeto, emisor y huella). Si lo emite
  una CA pública estable, añadir en `perfil()` `802-1x.system-ca-certs=true` +
  `802-1x.domain-suffix-match=<nombre>`. Si es autofirmado o cambia, valorar fijar su CA con
  `802-1x.ca-cert`. Probar en varios centros antes de publicarlo: si el certificado cambia entre
  centros, la validación dejaría a gente sin conexión.
- **Estado:** bloqueado hasta capturar el certificado en el centro.

### Contraseña fuera de disco (opcional)
- **Qué:** hoy NetworkManager guarda la contraseña en `/etc/NetworkManager/system-connections`
  (solo root). Alternativa: `802-1x.password-flags=1` para que la guarde el agente de secretos del
  escritorio (KWallet / GNOME Keyring).
- **Por qué importa:** en equipos compartidos con varios administradores.
- **Contras:** depende de que haya agente. Sin él, el Wi-Fi no reconecta solo. Documentado en el README.
- **Estado:** por decidir.

### Borrado seguro de la contraseña en memoria
- **Qué:** la contraseña se clona para el hilo y no se sobrescribe al liberarse. Ya se vacía el
  campo tras conectar.
- **Cómo:** `zeroize` sobre las copias. Añade una dependencia por una ganancia pequeña.
- **Estado:** baja prioridad, por decidir.

## Funcionalidad

- **Varias tarjetas Wi-Fi:** se usa la primera encendida. Si alguien tiene dos (p. ej. un USB),
  podría hacer falta un selector. Estado: sin demanda, no hacer salvo que aparezca.
- **Diálogo de contraseña del escritorio al fallar:** si la contraseña es incorrecta,
  NetworkManager puede pedirla al agente del escritorio (KDE/GNOME muestra su propio diálogo)
  mientras la app espera. Estado: verificar en el centro (ver `TESTING.md`). Si molesta, estudiarlo.
- **Contradicción en la guía oficial:** su tabla resumen dice PEAP+MSCHAPv2 para `WEDU_PROF`,
  pero todas las capturas y pasos dicen TTLS+PAP. TTLS+PAP está confirmado en uso real. Si algún
  día deja de funcionar, probar PEAP/MSCHAPv2.

## Mantenimiento

- **Actualizar egui/eframe** (0.27 → actual): cambia la API de la interfaz. Estado: cuando haga falta.
- **CI en GitHub Actions** con `cargo test` + `cargo clippy -D warnings`. Estado: propuesto.
- **Publicado** (2026-09-17): GitHub público con releases `v0.2.0` y `v0.2.1`, y AUR
  (`educamadrid-wifi` 0.2.1-1). El clon del repo AUR está en `~/Proyectos/aur/educamadrid-wifi`.
  Para nuevas versiones, ver «Publicar una versión» en el README.
