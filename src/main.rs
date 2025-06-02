use eframe::egui;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("assets/icon.png"))
        .expect("The icon data must be valid");
    let mut options = eframe::NativeOptions::default();

    options.viewport = egui::ViewportBuilder::default().with_inner_size([600.0, 320.0]);
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
}

impl Default for App {
    fn default() -> Self {
        let mut saved_passwords = HashMap::new();
        let mut show_passwords = HashMap::new();

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
            ui.heading("Password Generator 🔒");
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
            if ui.button("Generate password").clicked() {
                self.generate_password();
            }
            ui.add_space(10.0);
            if !self.password.is_empty() {
                ui.separator();
                ui.label("Password Generated:");
                ui.code(&self.password);

                ui.horizontal(|ui| {
                    ui.label("Save as:");
                    ui.text_edit_singleline(&mut self.save_title);
                    if ui.button("💾 Save Password").clicked() {
                        if !self.save_title.trim().is_empty() {
                            let key = self.save_title.trim().to_string();
                            self.saved_passwords
                                .insert(key.clone(), self.password.clone());
                            self.show_passwords.insert(key.clone(), false);
                            self.save_title.clear();

                            let json = serde_json::to_string_pretty(&SavedPasswords {
                                entries: self.saved_passwords.clone(),
                            })
                            .unwrap();

                            std::fs::write("src/passwords.json", json).ok();
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
                            ui.code(password);
                        } else {
                            ui.label("••••••••••");
                        }

                        let btn_label = if show { "Hide" } else { "Show" };
                        if ui.button(btn_label).clicked() {
                            self.show_passwords.insert(title.clone(), !show);
                        }

                        if ui
                            .button("📋")
                            .on_hover_text("Copiar al portapapeles")
                            .clicked()
                        {
                            ctx.output_mut(|o| o.copied_text = password.clone());
                        }

                        if ui.button("❌").clicked() {
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
