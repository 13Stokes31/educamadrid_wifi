use eframe::egui;
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;

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
        if let Some(rx) = &self.receiver {
            if let Ok(result) = rx.try_recv() {
                self.is_connecting = false;
                self.last_success = Some(result.is_ok());
                self.status_msg = match result {
                    Ok(msg) => msg,
                    Err(msg) => msg,
                };
                self.receiver = None;
            }
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

fn connect_to_wedu(username: &str, password: &str) -> Result<String, String> {
    let ssid = "WEDU_PROF";

    // Intentar borrar perfil previo
    let _ = Command::new("nmcli").args(["connection", "delete", ssid]).output();

    // Crear perfil TTLS / PAP
    let add_status = Command::new("nmcli")
    .args([
        "connection", "add",
        "type", "wifi",
        "con-name", ssid,
        "ifname", "*",
        "ssid", ssid,
        "wifi-sec.key-mgmt", "wpa-eap",
        "802-1x.eap", "ttls",
        "802-1x.phase2-auth", "pap",
        "802-1x.identity", username,
        "802-1x.password", password,
    ])
    .output();

    match add_status {
        Ok(output) if !output.status.success() => {
            return Err("❌ Error: No se pudo configurar el perfil.".to_string());
        }
        Err(_) => return Err("❌ Error: NetworkManager no responde.".to_string()),
        _ => {}
    }

    // Activar conexión
    let up_status = Command::new("nmcli").args(["connection", "up", ssid]).output();

    match up_status {
        Ok(output) if output.status.success() => Ok("✅ ¡Conectado con éxito!".to_string()),
        _ => Err("❌ Fallo en la autenticación. Revisa tus datos.".to_string()),
    }
}
