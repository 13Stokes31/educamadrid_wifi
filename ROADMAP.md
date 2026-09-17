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
- **Captura (2026-09-15/17, un centro, 43 autenticaciones, siempre idéntica):**
  - depth=0 (servidor): `CN=agile.wedu.comunidad.madrid`, O=Madrid Digital, OU=SOGEM.
    sha256 `312de39fb52b7dd6040b7235e1b39234a7e59c4f8aeb46b18c4821226f2a7b5c`
  - depth=1 (intermedia): `CN=CA Comunicaciones ICM (firma)`, O=Agencia de Informática y
    Comunicaciones. sha256 `ea37c9477cf291bd2885f6d3cc556409b0df301167f414a29d4c49c64e3f227d`
  - depth=2 (raíz): `CN=CA Comunicaciones ICM`, mismo O. sha256
    `65125e7e91144342b55e747bca66a5323cead4f21e7220cca0d30bb69413b7cb`
  - Es una CA **privada** de la Comunidad de Madrid: no está en el almacén del sistema y no se
    ha encontrado publicada. El log solo trae huellas, no el certificado en sí.
- **Estado:** MEJORA OPCIONAL, no urgente (decisión del usuario, 2026-09-17). La conexión funciona
  sin esto; solo protege frente a un punto de acceso falso. Se retomará si llega a tenerse el
  fichero de la CA raíz (capturándolo en el centro con la señal D-Bus `Certification` de
  wpa_supplicant, o pidiéndolo a Madrid Digital) y comprobando que su sha256 coincide con la
  capturada. No insistir mientras tanto.

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
