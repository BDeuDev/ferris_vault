use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
mod crypto;
use crypto::decrypt::decrypt;
use crypto::encrypt::encrypt;
use crypto::master_key::{hash_master_key, load_master_key_hash, save_master_key_hash};

fn main() -> Result<(), eframe::Error> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("assets/icon.png"))
        .expect("The icon data must be valid");
    let mut options = eframe::NativeOptions::default();

    options.viewport = egui::ViewportBuilder::default().with_inner_size([600.0, 500.0]);
    options.viewport.icon = Some(Arc::new(icon));
    eframe::run_native(
        "Ferris Vault - Password Generator",
        options,
        Box::new(|_cc| Box::new(App::default())),
    )
}

#[derive(Serialize, Deserialize)]
struct SavedPasswords {
    entries: HashMap<String, String>,
}

struct App {
    length: usize,
    include_numbers: bool,
    include_symbols: bool,
    include_uppercase: bool,
    password: String,
    save_title: String,
    saved_passwords: HashMap<String, String>,
    show_passwords: HashMap<String, bool>,
    key_input: String,
    master_key: Option<String>,
    set_master_key: bool,
    master_key_error: Option<String>,
    authenticated: bool,
    decrypted_cache: HashMap<String, String>,
    pending_decryption: HashMap<String, bool>,
    tx: Sender<(String, String)>,
    rx: Receiver<(String, String)>,
    save_tx: Sender<(String, String, String)>,
    save_rx: Receiver<(String, String, String)>,
    copy_tx: Sender<String>,
    copy_rx: Receiver<String>,
}

impl Default for App {
    fn default() -> Self {
        let mut saved_passwords = HashMap::new();
        let mut show_passwords = HashMap::new();
        let authenticated = false;
        let set_master_key = load_master_key_hash().is_none();
        let (tx, rx) = crossbeam_channel::unbounded();
        let (save_tx, save_rx) = crossbeam_channel::unbounded();
        let (copy_tx, copy_rx) = crossbeam_channel::unbounded();

        if let Ok(json) = std::fs::read_to_string("src/passwords.json") {
            if let Ok(parsed) = serde_json::from_str::<SavedPasswords>(&json) {
                saved_passwords = parsed.entries;
                for key in saved_passwords.keys() {
                    show_passwords.insert(key.clone(), false);
                }
            }
        }

        Self {
            length: 12,
            include_numbers: true,
            include_symbols: true,
            include_uppercase: true,
            password: String::new(),
            save_title: String::new(),
            saved_passwords,
            show_passwords,
            key_input: String::new(),
            master_key: None,
            set_master_key,
            master_key_error: None,
            authenticated,
            decrypted_cache: HashMap::new(),
            pending_decryption: HashMap::new(),
            tx,
            rx,
            save_tx,
            save_rx,
            copy_tx,
            copy_rx,
        }
    }
}

impl App {
    fn generate_password(&mut self) {
        let mut charset = "abcdefghijklmnopqrstuvwxyz".to_string();
        if self.include_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        if self.include_numbers {
            charset.push_str("0123456789");
        }
        if self.include_symbols {
            charset.push_str("!@#$%^&*()-_=+[]{};:,.<>?");
        }

        let mut rng = rand::thread_rng();
        self.password = (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap_or('_')
            })
            .collect();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            while let Ok((title, decrypted)) = self.rx.try_recv() {
                self.decrypted_cache.insert(title.clone(), decrypted);
                self.pending_decryption.remove(&title);
            }
            while let Ok(text) = self.copy_rx.try_recv() {
                ctx.output_mut(|o| o.copied_text = text);
            }

            while let Ok((title, encrypted, _)) = self.save_rx.try_recv() {
                self.saved_passwords.insert(title.clone(), encrypted);
                self.show_passwords.insert(title, false);

                let json = serde_json::to_string_pretty(&SavedPasswords {
                    entries: self.saved_passwords.clone(),
                })
                .unwrap();

                std::fs::write("src/passwords.json", json).ok();
            }

            if !self.authenticated {
                ui.heading("🔐 Ingresar clave maestra");
                ui.add_space(10.0);
                ui.label(if self.set_master_key {
                    "Definí tu nueva clave maestra. Recordá que no se puede recuperar."
                } else {
                    "Ingresá tu clave maestra para desbloquear las contraseñas."
                });

                ui.add(egui::TextEdit::singleline(&mut self.key_input).password(true));

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(if self.set_master_key {
                                "Guardar clave"
                            } else {
                                "Ingresar"
                            })
                            .color(egui::Color32::WHITE), // texto blanco
                        )
                        .fill(egui::Color32::from_rgb(0, 122, 255)), // azul (color estilo iOS)
                    )
                    .clicked()
                {
                    if self.key_input.len() < 4 {
                        self.master_key_error = Some("La clave es muy corta".into());
                    } else if self.set_master_key {
                        let hash = hash_master_key(&self.key_input);
                        save_master_key_hash(&hash);
                        self.set_master_key = false;
                        self.master_key = Some(self.key_input.clone());
                        self.authenticated = true;
                    } else if let Some(saved_hash) = load_master_key_hash() {
                        let hash = hash_master_key(&self.key_input);
                        if hash == saved_hash {
                            self.master_key = Some(self.key_input.clone());
                            self.authenticated = true;
                        } else {
                            self.master_key_error = Some("Clave incorrecta".into());
                        }
                    }
                    self.key_input.clear();
                }

                if let Some(err) = &self.master_key_error {
                    ui.colored_label(egui::Color32::RED, err);
                }

                return;
            }

            ui.heading("🔒 Password Generator");

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Number of characters:");
                ui.add(
                    egui::DragValue::new(&mut self.length)
                        .clamp_range(4..=64)
                        .speed(1),
                );
            });

            ui.checkbox(&mut self.include_uppercase, "Include uppercase letters");
            ui.checkbox(&mut self.include_numbers, "Include numbers");
            ui.checkbox(&mut self.include_symbols, "Include symbols");

            ui.add_space(10.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("Generate password").color(egui::Color32::WHITE), // texto blanco
                    )
                    .fill(egui::Color32::from_rgb(0, 122, 255)), // azul
                )
                .clicked()
            {
                self.generate_password();
            }

            if !self.password.is_empty() {
                ui.separator();
                ui.label("Password Generated:");
                ui.code(&self.password);

                ui.horizontal(|ui| {
                    ui.label("Save as:");
                    ui.text_edit_singleline(&mut self.save_title);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("📋 Save Password")
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(0, 122, 255)),
                        )
                        .clicked()
                    {
                        if let Some(key) = &self.master_key {
                            if !self.save_title.trim().is_empty() {
                                let title = self.save_title.trim().to_string();
                                let password = self.password.clone();
                                let key = key.clone();
                                let tx = self.save_tx.clone();

                                std::thread::spawn(move || {
                                    let encrypted = encrypt(&password, &key);
                                    let _ = tx.send((title, encrypted, "".to_string()));
                                });

                                self.save_title.clear();
                            }
                        }
                    }
                });
            }

            if !self.saved_passwords.is_empty() {
                ui.separator();
                ui.heading("Saved Passwords 🔑");
                ui.add_space(10.0);

                let mut to_delete = None;

                for (title, password) in &self.saved_passwords {
                    let show = self.show_passwords.get(title).copied().unwrap_or(false);
                    ui.horizontal(|ui| {
                        ui.label(format!("📌 {}:", title));

                        if show {
                            if let Some(cached) = self.decrypted_cache.get(title) {
                                ui.code(cached);
                            } else if self.pending_decryption.get(title).is_none() {
                                self.pending_decryption.insert(title.clone(), true);

                                let title_clone = title.clone();
                                let password_clone = password.clone();
                                let key_clone = self.master_key.clone();
                                let tx_clone = self.tx.clone();

                                std::thread::spawn(move || {
                                    if let Some(key) = key_clone {
                                        if let Some(decrypted) = decrypt(&password_clone, &key) {
                                            let _ = tx_clone.send((title_clone, decrypted));
                                        }
                                    }
                                });

                                ui.label("⏳ Deciphering...");
                            } else {
                                ui.label("⏳ Deciphering...");
                            }
                        } else {
                            ui.label("••••••••••");
                        }

                        let btn_label = if show { "Hide" } else { "Show" };
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(btn_label).color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(0, 0, 0)),
                            )
                            .clicked() {
                            self.show_passwords.insert(title.clone(), !show);
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("📋").color(egui::Color32::WHITE),
                                )
                                .fill(egui::Color32::from_rgb(152, 152, 152)),
                            )
                            .on_hover_text("Copy")
                            .clicked()
                        {
                            let key_clone = self.master_key.clone();
                            let password_clone = password.clone();
                            let copy_tx_clone = self.copy_tx.clone();

                            std::thread::spawn(move || {
                                if let Some(key) = key_clone {
                                    if let Some(decrypted) = decrypt(&password_clone, &key) {
                                        let _ = copy_tx_clone.send(decrypted);
                                    }
                                }
                            });
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("❌").color(egui::Color32::BLACK),
                                )
                                .fill(egui::Color32::from_rgb(253, 75, 75)),
                            )
                            .on_hover_text("Delete")
                            .clicked()
                        {
                            to_delete = Some(title.clone());
                        }
                    });
                }

                if let Some(key) = to_delete {
                    self.saved_passwords.remove(&key);
                    self.show_passwords.remove(&key);

                    let json = serde_json::to_string_pretty(&SavedPasswords {
                        entries: self.saved_passwords.clone(),
                    })
                    .unwrap();

                    std::fs::write("src/passwords.json", json).ok();
                }
            }
        });
    }
}
