use crate::edcas::EliteRustClient;
use crate::edcas::State::{
    About, CarrierPage, Explorer, MaterialInventory, Mining, News, Settings, StationPage,
};
use eframe::egui::{Context, TextStyle};
use eframe::{egui, App};

mod about;
mod body;
mod carrier;
mod explorer;
mod materials;
mod mining;
mod news;
mod settings;
mod station;

impl App for EliteRustClient {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if !self.settings.appearance_settings.applied {
            let mut style: egui::Style = (*ctx.style()).clone();
            for (text_style, font_id) in style.text_styles.iter_mut() {
                match text_style {
                    TextStyle::Small => {
                        if self.settings.appearance_settings.font_id.size > 4.0 {
                            font_id.size = self.settings.appearance_settings.font_id.size - 4.0;
                        } else {
                            font_id.size = self.settings.appearance_settings.font_id.size;
                        }
                    }
                    TextStyle::Heading => {
                        font_id.size = self.settings.appearance_settings.font_id.size + 4.0;
                    }
                    _ => {
                        font_id.size = self.settings.appearance_settings.font_id.size;
                        font_id.family = self.settings.appearance_settings.font_id.family.clone();
                    }
                }
            }
            ctx.set_style(style);
            self.settings.appearance_settings.font_size =
                self.settings.appearance_settings.font_id.size;
            self.settings.appearance_settings.font_style =
                self.settings.appearance_settings.font_id.family.to_string();
            self.settings.appearance_settings.applied = true;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Top panel as menu bar
            egui::menu::bar(ui, |menu_bar| {
                let news_button = menu_bar.button("News");
                if news_button.clicked() {
                    self.state = News;
                }
                let explorer_button = menu_bar.button("Explorer");
                if explorer_button.clicked() {
                    self.state = Explorer;
                }
                let mining_button = menu_bar.button("Mining");
                if mining_button.clicked() {
                    self.state = Mining;
                }
                let materials_button = menu_bar.button("Materials");
                if materials_button.clicked() {
                    self.state = MaterialInventory;
                }
                let station_button = menu_bar.button("Stations");
                if station_button.clicked() {
                    self.state = StationPage;
                }
                let carrier_button = menu_bar.button("Carriers");
                if carrier_button.clicked() {
                    self.state = CarrierPage;
                }
                let settings_button = menu_bar.button("Settings");
                if settings_button.clicked() {
                    self.state = Settings;
                }
                let about_button = menu_bar.button("About");
                if about_button.clicked() {
                    self.state = About;
                }

                menu_bar.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label(self.timestamp.as_str());
                });
                match self.state {
                    News => {
                        news_button.highlight();
                    }
                    About => {
                        about_button.highlight();
                    }
                    StationPage => {
                        station_button.highlight();
                    }
                    CarrierPage => {
                        carrier_button.highlight();
                    }
                    Settings => {
                        settings_button.highlight();
                    }
                    Explorer => {
                        explorer_button.highlight();
                    }
                    MaterialInventory => {
                        materials_button.highlight();
                    }
                    Mining => {
                        mining_button.highlight();
                    }
                }
            });
        });

        self.update_values();

        egui::CentralPanel::default().show(ctx, |_ui| match self.state {
            News => self.news.update(ctx, frame),
            About => self.about.update(ctx, frame),
            StationPage => self.station.update(ctx, frame),
            CarrierPage => self.carrier.update(ctx, frame),
            Settings => self.settings.update(ctx, frame),
            Explorer => self.explorer.update(ctx, frame),
            MaterialInventory => self.materials.update(ctx, frame),
            Mining => self.mining.update(ctx, frame),
        });
        //TODO more efficient way to send updates -> render only if new data comes in?
        //Low prio because performance is okay
        ctx.request_repaint();
    }
}
