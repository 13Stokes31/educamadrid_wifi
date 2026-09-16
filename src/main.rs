use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use zbus::blocking::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

const NM: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";
const SETTINGS_CONN: &str = "org.freedesktop.NetworkManager.Settings.Connection";
const SSID: &str = "WEDU_PROF";
const DOMINIO: &str = "@educa.madrid.org";

// Estados y motivos de NetworkManager (nm-dbus-interface.h).
const DEVICE_TYPE_WIFI: u32 = 2;
const DEVICE_STATE_DISCONNECTED: u32 = 30;
const DEVICE_STATE_FAILED: u32 = 120;
const ACTIVE_STATE_ACTIVATED: u32 = 2;
const ACTIVE_STATE_DEACTIVATED: u32 = 4;
const UPDATE2_FLAG_TO_DISK: u32 = 0x1;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 500.0])
            .with_resizable(false)
            .with_title("Wi-Fi Educamadrid")
            .with_app_id("educamadrid-wifi"),
        ..Default::default()
    };

    eframe::run_native(
        "educamadrid-wifi",
        options,
        Box::new(|cc| {
            // Forzar tema claro nada más empezar
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            Box::new(WeduApp::default()) as Box<dyn eframe::App>
        }),
    )
}

struct WeduApp {
    username: String,
    password: String,
    show_password: bool,
    status_msg: String,
    is_connecting: bool,
    last_success: Option<bool>,
    receiver: Option<Receiver<Result<String, String>>>,
}

impl Default for WeduApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            show_password: false,
            status_msg: String::from("Introduce tus credenciales para conectar."),
            is_connecting: false,
            last_success: None,
            receiver: None,
        }
    }
}

impl WeduApp {
    fn conectar(&mut self, ctx: &egui::Context) {
        self.show_password = false;
        let user = match normalizar_usuario(&self.username) {
            Ok(u) => u,
            Err(msg) => {
                self.last_success = Some(false);
                self.status_msg = msg;
                return;
            }
        };
        self.username = user.clone();
        self.is_connecting = true;
        self.last_success = None;
        self.status_msg = String::from("Conectando con NetworkManager...");

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let pass = self.password.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let res = connect_to_wedu(&user, &pass);
            let _ = tx.send(res);
            ctx.request_repaint();
        });
    }
}

impl eframe::App for WeduApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Escuchar el hilo de conexión
        if let Some(rx) = &self.receiver
            && let Ok(result) = rx.try_recv()
        {
            self.is_connecting = false;
            self.last_success = Some(result.is_ok());
            if result.is_ok() {
                // Ya está guardada en NetworkManager: no hace falta tenerla en pantalla.
                self.password.clear();
            }
            self.status_msg = match result {
                Ok(msg) => msg,
                Err(msg) => msg,
            };
            self.receiver = None;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(
                    egui::RichText::new("Conectividad Educamadrid")
                        .size(24.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("Configuración de red WEDU_PROF")
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(20.0);

                // CONTENEDOR DE FORMULARIO
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(245, 245, 247))
                    .rounding(12.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_width(340.0);

                        ui.label(egui::RichText::new("Usuario").strong());
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut self.username)
                                .hint_text("Usuario sin @educa.madrid.org")
                                .desired_width(f32::INFINITY)
                                .margin(egui::vec2(8.0, 8.0)),
                        );

                        ui.add_space(15.0);

                        ui.label(egui::RichText::new("Contraseña").strong());
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.password)
                                    .password(!self.show_password)
                                    .hint_text("Tu clave secreta")
                                    .desired_width(260.0)
                                    .margin(egui::vec2(8.0, 8.0)),
                            );

                            if ui
                                .button(if self.show_password { "👁" } else { "🙈" })
                                .on_hover_text("Mostrar/Ocultar")
                                .clicked()
                            {
                                self.show_password = !self.show_password;
                            }
                        });
                    });

                ui.add_space(25.0);

                // BOTÓN DE ACCIÓN
                let btn_text = if self.is_connecting {
                    "Conectando..."
                } else {
                    "CONECTAR AHORA"
                };
                let btn_enabled = !self.is_connecting
                    && !self.username.trim().is_empty()
                    && !self.password.is_empty();

                ui.scope(|ui| {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        egui::Color32::from_rgb(33, 150, 243);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                        egui::Color32::from_rgb(25, 118, 210);

                    let button = egui::Button::new(
                        egui::RichText::new(btn_text)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    )
                    .min_size(egui::vec2(200.0, 45.0))
                    .rounding(25.0);

                    ui.add_enabled_ui(btn_enabled, |ui| {
                        if ui.add(button).clicked() {
                            self.conectar(ctx);
                        }
                    });
                });

                ui.add_space(30.0);

                // TARJETA DE ESTADO
                let card_color = match self.last_success {
                    Some(true) => egui::Color32::from_rgb(232, 245, 233), // Verde claro
                    Some(false) => egui::Color32::from_rgb(255, 235, 238), // Rojo claro
                    None => egui::Color32::from_rgb(240, 240, 240),       // Gris claro
                };

                egui::Frame::none()
                    .fill(card_color)
                    .rounding(8.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(340.0);
                        ui.horizontal_wrapped(|ui| {
                            if self.is_connecting {
                                ui.spinner();
                            }
                            ui.label(egui::RichText::new(&self.status_msg).size(13.0));
                        });
                    });

                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Herramienta no oficial, sin relación con EducaMadrid.")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
    }
}

// Acepta "nombre.apellido" y también "nombre.apellido@educa.madrid.org" pegado entero.
fn normalizar_usuario(entrada: &str) -> Result<String, String> {
    let u = entrada.trim();
    let u = if u.to_lowercase().ends_with(DOMINIO) {
        &u[..u.len() - DOMINIO.len()]
    } else {
        u
    };
    if u.is_empty() || u.contains('@') || u.contains(char::is_whitespace) {
        return Err(format!(
            "❌ Usuario no válido: escribe solo la parte anterior a {DOMINIO}."
        ));
    }
    Ok(u.to_string())
}

// Perfil de conexión a{sa{sv}} (secciones → clave → valor), tal como lo describe
// la guía oficial de EducaMadrid: WPA2-Enterprise, TTLS, fase 2 PAP y SIN validar
// el certificado del servidor (EducaMadrid no publica CA ni dominio; ver README).
// `login`: usuario local dueño del perfil; si se indica, ningún otro usuario del
// equipo puede usarlo (connection.permissions).
fn perfil<'a>(
    usuario: &'a str,
    contrasena: &'a str,
    login: Option<&str>,
) -> HashMap<&'static str, HashMap<&'static str, Value<'a>>> {
    let mut p = HashMap::new();

    let mut s_con = HashMap::new();
    s_con.insert("id", Value::from(SSID));
    s_con.insert("type", Value::from("802-11-wireless"));
    if let Some(login) = login {
        s_con.insert("permissions", Value::from(vec![format!("user:{login}")]));
    }
    p.insert("connection", s_con);

    let mut s_wifi = HashMap::new();
    s_wifi.insert("ssid", Value::from(SSID.as_bytes().to_vec())); // ssid es 'ay' (bytes)
    s_wifi.insert("mode", Value::from("infrastructure"));
    s_wifi.insert("security", Value::from("802-11-wireless-security"));
    p.insert("802-11-wireless", s_wifi);

    let mut s_sec = HashMap::new();
    s_sec.insert("key-mgmt", Value::from("wpa-eap"));
    p.insert("802-11-wireless-security", s_sec);

    let mut s_eap = HashMap::new();
    s_eap.insert("eap", Value::from(vec!["ttls"]));
    s_eap.insert("phase2-auth", Value::from("pap"));
    s_eap.insert("identity", Value::from(usuario));
    s_eap.insert("password", Value::from(contrasena));
    p.insert("802-1x", s_eap);

    let mut s_ipv4 = HashMap::new();
    s_ipv4.insert("method", Value::from("auto"));
    p.insert("ipv4", s_ipv4);

    let mut s_ipv6 = HashMap::new();
    s_ipv6.insert("method", Value::from("auto"));
    p.insert("ipv6", s_ipv6);

    p
}

// Crea el perfil WEDU_PROF y lo activa hablando con NetworkManager por D-Bus (la
// contraseña no aparece en `ps`). El perfil nuevo vive solo en memoria hasta que
// conecta: si falla, se descarta y el perfil anterior (si lo había) sigue intacto.
fn connect_to_wedu(usuario: &str, contrasena: &str) -> Result<String, String> {
    let conn = Connection::system()
        .map_err(|e| format!("❌ No se pudo hablar con el sistema (D-Bus): {e}"))?;

    let dev = wifi_device(&conn)?;
    let login = std::env::var("USER").ok();
    let raiz = ObjectPath::try_from("/").unwrap();
    let mut opciones: HashMap<&str, Value> = HashMap::new();
    opciones.insert("persist", Value::from("memory"));

    let reply = conn
        .call_method(
            Some(NM),
            NM_PATH,
            Some(NM),
            "AddAndActivateConnection2",
            &(
                perfil(usuario, contrasena, login.as_deref()),
                &dev,
                &raiz,
                opciones,
            ),
        )
        .map_err(|e| format!("❌ No se pudo crear la conexión: {e}"))?;
    let (nuevo, activa, _): (
        OwnedObjectPath,
        OwnedObjectPath,
        HashMap<String, OwnedValue>,
    ) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("❌ Respuesta inesperada de NetworkManager: {e}"))?;

    if let Err(msg) = esperar_activacion(&conn, &activa, &dev) {
        // Borrar el perfil en memoria también cancela un intento que siga en curso.
        borrar(&conn, &nuevo);
        return Err(msg);
    }

    // Ha conectado: ahora sí se guarda en disco y se retiran los perfiles antiguos.
    let guardado = conn.call_method(
        Some(NM),
        nuevo.as_str(),
        Some(SETTINGS_CONN),
        "Update2",
        &(
            HashMap::<&str, HashMap<&str, Value>>::new(), // vacío = mantener ajustes
            UPDATE2_FLAG_TO_DISK,
            HashMap::<&str, Value>::new(),
        ),
    );
    if let Err(e) = guardado {
        return Ok(format!(
            "⚠️ Conectado, pero no se pudo guardar el perfil (habrá que repetirlo la próxima vez): {e}"
        ));
    }
    for viejo in perfiles_wedu(&conn) {
        if viejo != nuevo {
            borrar(&conn, &viejo);
        }
    }
    Ok("✅ ¡Conectado con éxito! El perfil queda guardado.".to_string())
}

// Lee una propiedad D-Bus de un objeto de NetworkManager.
fn propiedad(conn: &Connection, path: &str, iface: &str, nombre: &str) -> Option<OwnedValue> {
    conn.call_method(
        Some(NM),
        path,
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &(iface, nombre),
    )
    .ok()?
    .body()
    .deserialize::<OwnedValue>()
    .ok()
}

// Rutas de los perfiles guardados cuyo nombre (id) es WEDU_PROF.
fn perfiles_wedu(conn: &Connection) -> Vec<OwnedObjectPath> {
    let Ok(reply) = conn.call_method(
        Some(NM),
        "/org/freedesktop/NetworkManager/Settings",
        Some("org.freedesktop.NetworkManager.Settings"),
        "ListConnections",
        &(),
    ) else {
        return Vec::new();
    };
    let paths: Vec<OwnedObjectPath> = reply.body().deserialize().unwrap_or_default();
    paths
        .into_iter()
        .filter(|p| {
            let Ok(s) = conn.call_method(
                Some(NM),
                p.as_str(),
                Some(SETTINGS_CONN),
                "GetSettings",
                &(),
            ) else {
                return false;
            };
            let ajustes: HashMap<String, HashMap<String, OwnedValue>> =
                s.body().deserialize().unwrap_or_default();
            let id = ajustes
                .get("connection")
                .and_then(|c| c.get("id"))
                .and_then(|v| String::try_from(v.try_clone().ok()?).ok());
            id.as_deref() == Some(SSID)
        })
        .collect()
}

fn borrar(conn: &Connection, perfil: &OwnedObjectPath) {
    let _ = conn.call_method(
        Some(NM),
        perfil.as_str(),
        Some(SETTINGS_CONN),
        "Delete",
        &(),
    );
}

// Tarjeta Wi-Fi utilizable: la primera que NetworkManager gestiona y no está
// apagada (estado >= DISCONNECTED). Distingue "no hay tarjeta" de "Wi-Fi apagado".
fn wifi_device(conn: &Connection) -> Result<OwnedObjectPath, String> {
    let devices: Vec<OwnedObjectPath> = conn
        .call_method(Some(NM), NM_PATH, Some(NM), "GetDevices", &())
        .ok()
        .and_then(|r| r.body().deserialize().ok())
        .unwrap_or_default();
    let dev_iface = "org.freedesktop.NetworkManager.Device";
    let leer = |d: &OwnedObjectPath, nombre| {
        propiedad(conn, d.as_str(), dev_iface, nombre)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0)
    };

    let wifis: Vec<_> = devices
        .into_iter()
        .filter(|d| leer(d, "DeviceType") == DEVICE_TYPE_WIFI)
        .collect();
    if wifis.is_empty() {
        return Err("❌ No se encontró ninguna tarjeta Wi-Fi.".to_string());
    }
    wifis
        .into_iter()
        .find(|d| leer(d, "State") >= DEVICE_STATE_DISCONNECTED)
        .ok_or_else(|| "❌ El Wi-Fi está apagado. Actívalo y vuelve a intentarlo.".to_string())
}

// Traduce el motivo de fallo del dispositivo (NMDeviceStateReason) a algo legible.
fn explicar_motivo(motivo: u32) -> String {
    match motivo {
        7 | 8 | 10 => "❌ La red rechazó el usuario o la contraseña.".to_string(),
        11 => "❌ El servidor no respondió a tiempo (señal débil o red saturada).".to_string(),
        53 => format!("❌ No se encuentra la red {SSID}. ¿Estás en el centro?"),
        5 | 17 => "❌ Usuario aceptado, pero la red no dio dirección IP (DHCP).".to_string(),
        n => format!("❌ No se pudo conectar (motivo {n} de NetworkManager)."),
    }
}

// Sondea la conexión activa hasta ~30 s, anotando el motivo si el dispositivo falla.
fn esperar_activacion(
    conn: &Connection,
    activa: &OwnedObjectPath,
    dev: &OwnedObjectPath,
) -> Result<(), String> {
    let mut motivo = None;
    for i in 0..60 {
        thread::sleep(Duration::from_millis(500));

        if let Some((estado, m)) = propiedad(
            conn,
            dev.as_str(),
            "org.freedesktop.NetworkManager.Device",
            "StateReason",
        )
        .and_then(|v| <(u32, u32)>::try_from(v).ok())
            && estado == DEVICE_STATE_FAILED
        {
            motivo = Some(m);
        }

        let estado = propiedad(
            conn,
            activa.as_str(),
            "org.freedesktop.NetworkManager.Connection.Active",
            "State",
        )
        .and_then(|v| u32::try_from(v).ok());
        match estado {
            Some(ACTIVE_STATE_ACTIVATED) => return Ok(()),
            Some(ACTIVE_STATE_DEACTIVATED) => {}
            Some(_) => continue, // activándose
            // La conexión activa desapareció (NM la retira al fallar): también es fallo.
            None if i > 0 => {}
            None => continue,
        }
        return Err(match motivo {
            Some(m) => explicar_motivo(m),
            None => {
                "❌ No se pudo conectar (usuario/contraseña o red fuera de alcance).".to_string()
            }
        });
    }
    Err(format!(
        "❌ La conexión tardó demasiado. Revisa que {SSID} esté al alcance."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usuario_se_normaliza() {
        assert_eq!(normalizar_usuario("  ana.lopez ").unwrap(), "ana.lopez");
        assert_eq!(
            normalizar_usuario("ana.lopez@Educa.Madrid.org").unwrap(),
            "ana.lopez"
        );
    }

    #[test]
    fn usuario_invalido_se_rechaza() {
        for mal in ["", "   ", "ana lopez", "ana@gmail.com", "@educa.madrid.org"] {
            assert!(normalizar_usuario(mal).is_err(), "debería rechazar {mal:?}");
        }
    }

    #[test]
    fn perfil_sigue_la_guia_oficial() {
        let p = perfil("ana.lopez", "secreta", Some("ana"));
        assert_eq!(
            p["802-11-wireless-security"]["key-mgmt"],
            Value::from("wpa-eap")
        );
        assert_eq!(p["802-1x"]["eap"], Value::from(vec!["ttls"]));
        assert_eq!(p["802-1x"]["phase2-auth"], Value::from("pap"));
        assert_eq!(p["802-1x"]["identity"], Value::from("ana.lopez"));
        assert_eq!(
            p["802-11-wireless"]["ssid"],
            Value::from(b"WEDU_PROF".to_vec())
        );
        assert_eq!(
            p["connection"]["permissions"],
            Value::from(vec!["user:ana".to_string()])
        );
    }

    #[test]
    fn perfil_sin_login_no_restringe() {
        let p = perfil("ana.lopez", "secreta", None);
        assert!(!p["connection"].contains_key("permissions"));
    }
}
