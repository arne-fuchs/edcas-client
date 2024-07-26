use crate::edcas::backend::evm::journal_uploader;
use crate::edcas::settings::{Settings, JournalReadStatus};
use eframe::egui::scroll_area::ScrollBarVisibility::AlwaysVisible;
use eframe::egui::{global_dark_light_mode_switch, vec2, Color32, Context, Window};
use eframe::{egui, App, Frame};
use ethers::prelude::{LocalWallet, Signer};
use log::error;
use std::path::Path;
use std::{env, fs};

impl App for Settings {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let Self { .. } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Appearance");
                egui::Grid::new("appearance_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Font-style:");
                        egui::introspection::font_id_ui(ui, &mut self.appearance_settings.font_id);
                        ui.end_row();
                        if ui.button("Apply").clicked() {
                            self.appearance_settings.applied = false;
                        }
                    });
                ui.separator();

                ui.heading("Journal File Settings");
                egui::Grid::new("journal_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        if Path::new(&self.journal_reader_settings.journal_directory).exists() {
                            ui.label("Journal Directory:");
                        } else {
                            ui.label("Journal Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.journal_reader_settings.journal_directory);
                        if ui.button("Open").clicked() {
                            opener::open(&self.journal_reader_settings.journal_directory).unwrap();
                        }
                        ui.end_row();
                        ui.label("Log File:");
                        ui.label(&self.log_path);
                        if ui.button("Open").clicked() {
                            opener::open(&self.log_path).unwrap();
                        }
                        ui.end_row();
                        ui.label("Action after reaching shutdown:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.journal_reader_settings.action_at_shutdown_signal.to_string())
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, crate::edcas::settings::ActionAtShutdownSignal::Exit, "exit");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, crate::edcas::settings::ActionAtShutdownSignal::Continue, "continue");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, crate::edcas::settings::ActionAtShutdownSignal::Nothing, "nothing");
                            });
                    });
                ui.separator();

                egui::CollapsingHeader::new("Explorer").show(ui, |ui| {
                    ui.checkbox(&mut self.explorer_settings.include_system_name, "Include system in body name");
                    egui::CollapsingHeader::new("Icons").show(ui, |ui| {
                        egui::Grid::new("explorer_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for icon in &mut self.icons {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut icon.1.char);
                                        ui.label(icon.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut icon.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut icon.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();
                    egui::CollapsingHeader::new("Stars").show(ui, |ui| {
                        egui::Grid::new("explorer_star_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for star in &mut self.stars {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut star.1.char);
                                        ui.label(star.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut star.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut star.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();
                    egui::CollapsingHeader::new("Planets").show(ui, |ui| {
                        egui::Grid::new("explorer_planet_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for planet in &mut self.planets {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut planet.1.char);
                                        ui.label(planet.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut planet.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut planet.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();
                });
                ui.separator();
                ui.heading("Graphics Override");
                egui::Grid::new("graphics_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        if ui.button("Open Editor").clicked() {
                            self.graphic_editor_settings.show_editor = !self.graphic_editor_settings.show_editor;
                        }
                        ui.end_row();
                        if Path::new(&self.graphic_editor_settings.graphics_directory).exists() {
                            ui.label("Graphics Directory:");
                        } else {
                            ui.label("Graphics Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.graphic_editor_settings.graphics_directory);
                        if ui.button("Open").clicked() {
                            opener::open(&self.graphic_editor_settings.graphics_directory).unwrap();
                        }
                        ui.end_row();
                        if self.graphic_editor_settings.show_editor {
                            self.show_graphics_editor(ctx);
                        }
                    });

                ui.separator();
                let info_ui = |ui: &mut egui::Ui| {
                    ui.heading("What data is being shared?");
                    ui.label("Because of the early development, many things can change between releases.\n\
                    So currently you have to assume that potentially everything will be shared over the EDCAS network and therefore is being available in the internet \n\
                    what is being written in the journal log.\n\
                    If you do not want that, please leave this function disabled.\n\
                    Keep in mind, that your experience might decrease if you leave this disabled.");
                };

                ui.heading("EDCAS Network").on_hover_ui(info_ui);
                egui::Grid::new("network_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Allow to share journal log data:").on_hover_ui(info_ui);
                        ui.checkbox(&mut self.evm_settings.allow_share_data, "");
                        ui.end_row();
                        if ui
                            .button("Upload journal data ⬆")
                            .clicked()
                        {
                            self.evm_settings.show_upload_data_window = true;
                        }
                        if self.evm_settings.show_upload_data_window {
                            Window::new("Upload journal data ⬆").collapsible(false).show(ctx,|ui| {
                                ui.label("Are you sure to upload all journal data to the EDCAS network? Depending on the size of your journal logs, it may can take up to several hours.");
                                ui.vertical_centered(|ui|{
                                    if let Some(ref mut upload_status) = self.evm_settings.journal_read_status {
                                        ui.label("You are able to close this window and still use EDCAS. It will run in the background");
                                        if let Ok(index) = upload_status.log_index_updates.try_recv() {upload_status.current_log = index as u32;}
                                        let status = (upload_status.total_logs - upload_status.current_log) as f32 / upload_status.total_logs as f32;
                                        ui.add(egui::ProgressBar::new(status).text(format!("{} of {} logs read", upload_status.total_logs - upload_status.current_log, upload_status.total_logs)));
                                    }else if ui.button("Do it!").clicked(){
                                        let (progress_bus_reader, total) = journal_uploader::initialize(&self.evm_settings, self.journal_reader_settings.journal_directory.clone());
                                        self.evm_settings.journal_read_status = Some(JournalReadStatus {
                                            current_log: 1,
                                            total_logs: total as u32,
                                            log_index_updates: progress_bus_reader,
                                        });
                                    }
                                    if ui.button("Close Window").clicked() {
                                        self.evm_settings.show_upload_data_window = false;
                                    }
                                });

                            }).unwrap();
                        }

                        ui.end_row();
                        ui.label("EVM RPC:");
                        ui.text_edit_singleline(&mut self.evm_settings.url);
                        ui.end_row();
                        ui.label("Smart Contract Address:");
                        ui.text_edit_singleline(&mut self.evm_settings.smart_contract_address);
                        ui.end_row();
                        ui.label("Private Key:");
                        ui.add(egui::TextEdit::singleline(&mut self.evm_settings.private_key).password(true));
                        ui.end_row();
                        ui.label("Address:");
                        let address = format!("{:?}", self.evm_settings.private_key.parse::<LocalWallet>().unwrap().address());
                        if ui
                            .button(format!("{} 🗐", &address))
                            .clicked()
                        {
                            ui.output_mut(|o| {
                                o.copied_text =
                                    address
                            });
                        }
                        ui.end_row();
                        ui.label("EVM Adapter Timeout:");
                        ui.add(egui::Slider::new(&mut self.evm_settings.n_timeout, 0..=20).suffix(" Seconds"));
                        ui.end_row();
                        ui.label("EVM Adapter Attempts:");
                        ui.add(egui::Slider::new(&mut self.evm_settings.n_attempts, 0..=20).suffix(" Attempts"));
                    });
            });
            ui.separator();
            ui.end_row();
            ui.label("Some settings require a restart to be applied");
            ui.end_row();
            ui.separator();

            //Apply Button
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Save 💾").clicked() {
                    self.save_settings_to_file();
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                global_dark_light_mode_switch(ui);
            });
        });
    }
}

impl Settings {
    fn show_graphics_editor(&mut self, ctx: &Context) {
        Window::new("Editor")
            .fixed_size(vec2(800f32, 600f32))
            .show(ctx, |ui| {
                egui::Grid::new("preset_buttons")
                    .show(ui, |ui| {
                        ui.hyperlink_to(
                            "Fandom Article",
                            "https://elite-dangerous.fandom.com/wiki/Graphics_Mods",
                        );
                        egui::ComboBox::from_id_source("Presets_Combo_Box")
                            .selected_text("Presets")
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, crate::edcas::settings::presets::get_increase_texture_resolution_preset(), "Increased Textures");
                                ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, crate::edcas::settings::presets::get_increased_star_count_preset(), "Increased Star Count");
                                ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, crate::edcas::settings::presets::get_better_skybox_preset(), "Better Skybox");
                                ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, crate::edcas::settings::presets::get_8gb_plus_preset(), "8Gb+ VRAM");
                            });
                        if ui.button("Load custom preset").clicked() {
                            self.graphic_editor_settings.graphic_override_content = match env::var("HOME") {
                                Ok(home) => {
                                    match fs::read_to_string(format!("{}/.local/share/edcas-client/custom_graphics_override.xml", home)) {
                                        Ok(file) => {
                                            file
                                        }
                                        Err(_) => {
                                            fs::read_to_string("custom_graphics_override.xml").unwrap()
                                        }
                                    }
                                }
                                Err(_) => {
                                    fs::read_to_string("custom_graphics_override.xml").unwrap()
                                }
                            }
                        }
                        if ui.button("Save as custom preset").clicked() {
                            match env::var("HOME") {
                                Ok(home) => {
                                    match fs::write(format!("{}/.local/share/edcas-client/custom_graphics_override.xml", home), self.graphic_editor_settings.graphic_override_content.clone()) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            fs::write("custom_graphics_override.xml", self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                        }
                                    }
                                }
                                Err(_) => {
                                    fs::write("custom_graphics_override.xml", self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                }
                            };
                        }
                    });
                ui.end_row();
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(AlwaysVisible)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.graphic_editor_settings.graphic_override_content)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(10)
                                .text_color(Color32::from_rgb(255, 165, 0))
                                .font(egui::FontId::monospace(10.0))
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                        );
                    });
                egui::Grid::new("editor_buttons")
                    .show(ui, |ui| {
                        if ui.button("Save").clicked() {
                            match fs::write(format!("{}/GraphicsConfigurationOverride.xml", self.graphic_editor_settings.graphics_directory.clone()), self.graphic_editor_settings.graphic_override_content.clone()) {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Failed to save settings: {}",err);
                                    panic!("Failed to save settings: {}", err);
                                }
                            }
                        }
                        if ui.button("Exit").clicked() {
                            self.graphic_editor_settings.show_editor = false;
                        }
                        if ui.button("Reset").clicked() {
                            self.graphic_editor_settings.graphic_override_content = fs::read_to_string(format!("{}/GraphicsConfigurationOverride.xml", self.graphic_editor_settings.graphics_directory.clone())).unwrap();
                        }
                        if ui.button("Defaults").clicked() {
                            self.graphic_editor_settings.graphic_override_content = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig />\n".into();
                        }
                    });
            });
    }
}
