use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use zbus::blocking::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value};

const NM: &str = "org.freedesktop.NetworkManager";

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([420.0, 480.0])
        .with_resizable(false)
        .with_title("Wi-Fi Educamadrid"),
        ..Default::default()
    };

    eframe::run_native(
        "WeduConnectPro",
        options,
        Box::new(|cc| {
            // Forzar tema claro nada más empezar
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            // Devolvemos la App directamente sin el Ok()
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

impl eframe::App for WeduApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Escuchar el hilo de conexión
        if let Some(rx) = &self.receiver
            && let Ok(result) = rx.try_recv()
        {
            self.is_connecting = false;
            self.last_success = Some(result.is_ok());
            self.status_msg = match result {
                Ok(msg) => msg,
                Err(msg) => msg,
            };
            self.receiver = None;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Estilo general: espaciado y márgenes
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(egui::RichText::new("Conectividad Educamadrid").size(24.0).strong());
                ui.label(egui::RichText::new("Configuración de red WEDU_PROF").color(egui::Color32::GRAY));
                ui.add_space(20.0);

                // CONTENEDOR DE FORMULARIO
                egui::Frame::none()
                .fill(egui::Color32::from_rgb(245, 245, 247))
                .rounding(12.0)
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_width(340.0);

                    // Campo de Usuario
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Usuario").strong());
                    });
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.username)
                        .hint_text("Usuario sin @educa.madrid.org")
                        .desired_width(f32::INFINITY)
                        .margin(egui::vec2(8.0, 8.0))
                    );

                    ui.add_space(15.0);

                    // Campo de Contraseña
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Contraseña").strong());
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let edit = egui::TextEdit::singleline(&mut self.password)
                        .password(!self.show_password)
                        .hint_text("Tu clave secreta")
                        .desired_width(260.0)
                        .margin(egui::vec2(8.0, 8.0));

                        ui.add(edit);

                        // Botón de Ver/Ocultar (Icono simulado)
                        if ui.button(if self.show_password { "👁" } else { "🙈" }).on_hover_text("Mostrar/Ocultar").clicked() {
                            self.show_password = !self.show_password;
                        }
                    });
                });

                ui.add_space(25.0);

                // BOTÓN DE ACCIÓN (Estilo Material)
                let btn_text = if self.is_connecting { "Conectando..." } else { "CONECTAR AHORA" };
                let btn_enabled = !self.is_connecting && !self.username.is_empty() && !self.password.is_empty();

                ui.scope(|ui| {
                    // Personalizamos el color del botón
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(33, 150, 243);
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(25, 118, 210);

                    let button = egui::Button::new(egui::RichText::new(btn_text).color(egui::Color32::WHITE).strong())
                    .min_size(egui::vec2(200.0, 45.0))
                    .rounding(25.0);

                    ui.add_enabled_ui(btn_enabled, |ui| {
                        if ui.add(button).clicked() {
                            self.is_connecting = true;
                            self.last_success = None;
                            self.status_msg = String::from("Sincronizando con NetworkManager...");

                            let (tx, rx) = mpsc::channel();
                            self.receiver = Some(rx);
                            let user = self.username.clone();
                            let pass = self.password.clone();
                            let ctx = ctx.clone();

                            thread::spawn(move || {
                                let res = connect_to_wedu(&user, &pass);
                                let _ = tx.send(res);
                                ctx.request_repaint();
                            });
                        }
                    });
                });

                ui.add_space(30.0);

                // TARJETA DE ESTADO
                let card_color = match self.last_success {
                    Some(true) => egui::Color32::from_rgb(232, 245, 233), // Verde claro
                                 Some(false) => egui::Color32::from_rgb(255, 235, 238), // Rojo claro
                                 None => egui::Color32::from_rgb(240, 240, 240),      // Gris claro
                };

                egui::Frame::none()
                .fill(card_color)
                .rounding(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(340.0);
                    ui.horizontal(|ui| {
                        if self.is_connecting {
                            ui.spinner();
                        }
                        ui.label(egui::RichText::new(&self.status_msg).size(13.0));
                    });
                });
            });
        });
    }
}

// Crea (o recrea) el perfil WEDU_PROF y lo activa, hablando con NetworkManager
// por D-Bus. Frente a `nmcli`, la contraseña viaja DENTRO de la llamada D-Bus y
// nunca aparece en la línea de comandos (`ps`/`/proc/<pid>/cmdline`).
fn connect_to_wedu(username: &str, password: &str) -> Result<String, String> {
    let ssid = "WEDU_PROF";

    let conn = Connection::system()
        .map_err(|e| format!("❌ No se pudo hablar con el sistema (D-Bus): {e}"))?;

    // Borrar perfiles WEDU_PROF previos (mejor esfuerzo, como el `delete` de antes)
    borrar_perfiles(&conn, ssid);

    // Construir el perfil de conexión: a{sa{sv}} (secciones → clave → valor).
    let mut perfil: HashMap<&str, HashMap<&str, Value>> = HashMap::new();

    let mut s_con = HashMap::new();
    s_con.insert("id", Value::from(ssid));
    s_con.insert("type", Value::from("802-11-wireless"));
    perfil.insert("connection", s_con);

    let mut s_wifi = HashMap::new();
    s_wifi.insert("ssid", Value::from(ssid.as_bytes().to_vec())); // ssid es 'ay' (bytes)
    s_wifi.insert("mode", Value::from("infrastructure"));
    s_wifi.insert("security", Value::from("802-11-wireless-security"));
    perfil.insert("802-11-wireless", s_wifi);

    let mut s_sec = HashMap::new();
    s_sec.insert("key-mgmt", Value::from("wpa-eap"));
    perfil.insert("802-11-wireless-security", s_sec);

    let mut s_eap = HashMap::new();
    s_eap.insert("eap", Value::from(vec!["ttls"]));
    s_eap.insert("phase2-auth", Value::from("pap"));
    s_eap.insert("identity", Value::from(username));
    s_eap.insert("password", Value::from(password));
    perfil.insert("802-1x", s_eap);

    let mut s_ipv4 = HashMap::new();
    s_ipv4.insert("method", Value::from("auto"));
    perfil.insert("ipv4", s_ipv4);

    let mut s_ipv6 = HashMap::new();
    s_ipv6.insert("method", Value::from("auto"));
    perfil.insert("ipv6", s_ipv6);

    // Dispositivo Wi-Fi sobre el que activar
    let dev = wifi_device(&conn)
        .ok_or_else(|| "❌ No se encontró ninguna tarjeta Wi-Fi.".to_string())?;
    let raiz = ObjectPath::try_from("/").unwrap();

    // Añadir + activar en una sola llamada (queda guardado para la bandeja de KDE)
    let reply = conn
        .call_method(
            Some(NM),
            "/org/freedesktop/NetworkManager",
            Some(NM),
            "AddAndActivateConnection",
            &(perfil, &dev, &raiz),
        )
        .map_err(|e| format!("❌ No se pudo crear la conexión: {e}"))?;
    let (_perfil_path, activa): (OwnedObjectPath, OwnedObjectPath) = reply
        .body()
        .deserialize()
        .map_err(|e| format!("❌ Respuesta inesperada de NetworkManager: {e}"))?;

    esperar_activacion(&conn, &activa)
}

// Borra todos los perfiles guardados cuyo id sea `ssid` (mejor esfuerzo).
fn borrar_perfiles(conn: &Connection, ssid: &str) {
    let Ok(reply) = conn.call_method(
        Some(NM),
        "/org/freedesktop/NetworkManager/Settings",
        Some("org.freedesktop.NetworkManager.Settings"),
        "ListConnections",
        &(),
    ) else {
        return;
    };
    let Ok(paths): Result<Vec<OwnedObjectPath>, _> = reply.body().deserialize() else {
        return;
    };
    for p in paths {
        let Ok(s) = conn.call_method(
            Some(NM),
            p.as_str(),
            Some("org.freedesktop.NetworkManager.Settings.Connection"),
            "GetSettings",
            &(),
        ) else {
            continue;
        };
        let ajustes: HashMap<String, HashMap<String, OwnedValue>> =
            match s.body().deserialize() {
                Ok(a) => a,
                Err(_) => continue,
            };
        let id = ajustes
            .get("connection")
            .and_then(|c| c.get("id"))
            .and_then(|v| String::try_from(v.try_clone().ok()?).ok());
        if id.as_deref() == Some(ssid) {
            let _ = conn.call_method(
                Some(NM),
                p.as_str(),
                Some("org.freedesktop.NetworkManager.Settings.Connection"),
                "Delete",
                &(),
            );
        }
    }
}

// Primer dispositivo de tipo Wi-Fi (NM_DEVICE_TYPE_WIFI = 2).
fn wifi_device(conn: &Connection) -> Option<OwnedObjectPath> {
    let reply = conn
        .call_method(Some(NM), "/org/freedesktop/NetworkManager", Some(NM), "GetDevices", &())
        .ok()?;
    let devices: Vec<OwnedObjectPath> = reply.body().deserialize().ok()?;
    for d in devices {
        let Ok(r) = conn.call_method(
            Some(NM),
            d.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager.Device", "DeviceType"),
        ) else {
            continue;
        };
        let tipo = r
            .body()
            .deserialize::<OwnedValue>()
            .ok()
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0);
        if tipo == 2 {
            return Some(d);
        }
    }
    None
}

// Sondea el estado de la conexión activa hasta ~20 s.
// Estados NMActiveConnectionState: 2 = activada, 4 = desactivada (fallo).
fn esperar_activacion(conn: &Connection, activa: &OwnedObjectPath) -> Result<String, String> {
    for i in 0..40 {
        thread::sleep(Duration::from_millis(500));
        let estado = conn
            .call_method(
                Some(NM),
                activa.as_str(),
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &("org.freedesktop.NetworkManager.Connection.Active", "State"),
            )
            .ok()
            .and_then(|m| m.body().deserialize::<OwnedValue>().ok())
            .and_then(|v| u32::try_from(v).ok());
        match estado {
            Some(2) => return Ok("✅ ¡Conectado con éxito!".to_string()),
            Some(4) => return Err("❌ Fallo en la autenticación o red fuera de alcance.".to_string()),
            Some(_) => continue, // activándose
            // La conexión activa desapareció (NM la retira al fallar): también es fallo.
            None if i > 0 => return Err("❌ Fallo en la autenticación o red fuera de alcance.".to_string()),
            None => continue,
        }
    }
    Err("❌ La conexión tardó demasiado. Revisa que WEDU_PROF esté al alcance.".to_string())
}
