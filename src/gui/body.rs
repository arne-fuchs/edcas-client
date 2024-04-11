use crate::app::explorer::planet::{get_body_class_from_body, get_profit_from_body, Planet};
use crate::app::explorer::ring::Ring;
use crate::app::explorer::star::Star;
use crate::app::explorer::body::BodyType;
use eframe::egui::Ui;
use num_format::{Locale, ToFormattedString};
use crate::app::explorer::belt_cluster::BeltCluster;

impl BodyType {
    pub fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        match self {
            BodyType::Star(star) => star.print_header_content(ui, system_index, body_index),
            BodyType::Planet(planet) => planet.print_header_content(ui, system_index, body_index),
            BodyType::Ring(ring) => ring.print_header_content(ui, system_index, body_index),
            BodyType::BeltCluster(cluster) => cluster.print_header_content(ui, system_index, body_index),
        }
    }
    pub fn print_side_panel_information(&self, ui: &mut Ui) {
        match self {
            BodyType::Star(star) => star.print_side_panel_information(ui),
            BodyType::Planet(planet) => planet.print_side_panel_information(ui),
            BodyType::Ring(ring) => ring.print_side_panel_information(ui),
            BodyType::BeltCluster(cluster) => cluster.print_side_panel_information(ui),
        }
    }
}
impl Star {
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        if self
            .settings
            .stars
            .get(self.star_type.as_str())
            .unwrap()
            .enabled
        {
            ui.label(
                self.settings
                    .stars
                    .get(self.star_type.as_str())
                    .unwrap()
                    .get_richtext()
                    .size(self.radius.log(1.7) as f32),
            );
        }
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            body_name.replace_range(0..self.star_system.len(), "");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        for ring in &self.asteroid_rings {
            if self
                .settings
                .icons
                .get(ring.ring_class.as_str())
                .unwrap()
                .enabled
            {
                ui.label("|");
                ui.label(
                    self.settings
                        .icons
                        .get(ring.ring_class.as_str())
                        .unwrap()
                        .get_richtext(),
                );
            }
        }

        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled {
            ui.label("|");
            ui.label(
                self.settings
                    .icons
                    .get("discovered")
                    .unwrap()
                    .get_richtext(),
            );
        }
        if self.was_mapped && self.settings.icons.get("mapped").unwrap().enabled {
            ui.label("|");
            ui.label(self.settings.icons.get("mapped").unwrap().get_richtext());
        }
    }
    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Type");
        ui.label(&self.star_type);
        ui.end_row();
        ui.label("Subclass");
        ui.label(&self.subclass.to_string());
        ui.end_row();
        ui.label("Age in My");
        ui.label(&self.age_my.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("Stellar Mass");
        ui.label(&self.stellar_mass.to_string());
        ui.end_row();
        ui.label("Radius");
        ui.label(&self.radius.to_string());
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();
        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
        ui.end_row();
        ui.label("Temperature K");
        ui.label(&self.surface_temperature.to_string());
        ui.end_row();
        ui.heading("Rings");
        ui.end_row();
        for ring in &self.asteroid_rings {
            ui.label(&ring.ring_class);
            ui.vertical(|ui| {
                ui.label(format!("{}km", &ring.outer_rad));
                ui.label(format!("{}km", &ring.inner_rad));
            });
            ui.end_row();
        }
    }
}

impl Planet {
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        if self
            .settings
            .planets
            .get(self.planet_class.as_str())
            .unwrap()
            .enabled
        {
            ui.label(
                self.settings
                    .planets
                    .get(self.planet_class.as_str())
                    .unwrap()
                    .get_richtext()
                    .size(self.radius.log(1.5) as f32),
            );
        }

        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            body_name.replace_range(0..self.star_system.len(), "");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        for ring in &self.asteroid_rings {
            if self
                .settings
                .icons
                .get(ring.ring_class.as_str())
                .unwrap()
                .enabled
            {
                ui.label("|");
                ui.label(
                    self.settings
                        .icons
                        .get(ring.ring_class.as_str())
                        .unwrap()
                        .get_richtext(),
                );
            }
        }

        if self.landable && self.settings.icons.get("landable").unwrap().enabled {
            ui.label("|");
            ui.label(self.settings.icons.get("landable").unwrap().get_richtext());
        }

        if !self.planet_signals.is_empty() {
            for signal in &self.planet_signals {
                //⛓ -> xeno
                //🌱 🌿 🌴-> bio
                //🌋 🗻 -> xeno
                //✋ -> human
                //？ -> unknown
                //⛅☁ -> Atmosphere
                match signal.r#type.as_str() {
                    "$SAA_SignalType_Biological;" => {
                        if self.settings.icons.get("bio_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(
                                self.settings
                                    .icons
                                    .get("bio_signal")
                                    .unwrap()
                                    .get_richtext(),
                            );
                        }
                    }
                    "$SAA_SignalType_Geological;" => {
                        if self.settings.icons.get("geo_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(
                                self.settings
                                    .icons
                                    .get("geo_signal")
                                    .unwrap()
                                    .get_richtext(),
                            );
                        }
                    }
                    "$SAA_SignalType_Xenological;" => {
                        if self.settings.icons.get("xeno_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(
                                self.settings
                                    .icons
                                    .get("xeno_signal")
                                    .unwrap()
                                    .get_richtext(),
                            );
                        }
                    }
                    "$SAA_SignalType_Human;" => {
                        if self.settings.icons.get("human_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(
                                self.settings
                                    .icons
                                    .get("human_signal")
                                    .unwrap()
                                    .get_richtext(),
                            );
                        }
                    }
                    _ => {
                        if self.settings.icons.get("unknown_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(
                                self.settings
                                    .icons
                                    .get("unknown_signal")
                                    .unwrap()
                                    .get_richtext(),
                            );
                        }
                    }
                }
            }
        }
        if self.settings.icons.get("gravity").unwrap().enabled {
            ui.label("|");
            ui.label(self.surface_gravity.to_string());
            ui.label("G");
            ui.label(self.settings.icons.get("gravity").unwrap().get_richtext());
        }
        if self.settings.icons.get("distance").unwrap().enabled {
            ui.label("|");
            ui.label((self.distance_from_arrival_ls as u64).to_formatted_string(&Locale::en));
            ui.label("LS");
            ui.label(self.settings.icons.get("distance").unwrap().get_richtext());
        }
        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled {
            ui.label("|");
            ui.label(
                self.settings
                    .icons
                    .get("discovered")
                    .unwrap()
                    .get_richtext(),
            );
        }
        if self.was_mapped && self.settings.icons.get("mapped").unwrap().enabled {
            ui.label("|");
            ui.label(self.settings.icons.get("mapped").unwrap().get_richtext());
        }
    }
    fn print_side_panel_information(&self, ui: &mut Ui) {
        let profit = get_profit_from_body(get_body_class_from_body(self), self.was_discovered);
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Class");
        ui.label(&self.planet_class);
        ui.end_row();
        ui.label("Terraform State");
        ui.label(&self.terraform_state);
        ui.end_row();
        ui.label("Profit");
        ui.end_row();
        ui.label("Discovery");
        ui.label(profit.0.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("Mapping");
        ui.label(profit.1.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();

        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Mapped");
        ui.label(&self.was_mapped.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
        ui.end_row();
        ui.label("Landable");
        ui.label(&self.landable.to_string());
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();
        ui.label("Gravity");
        ui.label(&self.surface_gravity.to_string());
        ui.end_row();
        ui.label("Radius");
        ui.label(&self.radius.to_string());
        ui.end_row();
        ui.label("Temperature K");
        ui.label(&self.surface_temperature.to_string());
        ui.end_row();
        ui.label("Atmosphere");
        ui.label(&self.atmosphere);
        ui.end_row();
        for atmosphere_composition in &self.atmosphere_composition {
            ui.label(&atmosphere_composition.name);
            ui.label(format!("{}%", &atmosphere_composition.percent));
            ui.end_row();
        }
        ui.label("Reserve level");
        ui.label(&self.reserve_level);
        ui.end_row();
        ui.heading("Material");
        ui.end_row();
        for material in &self.materials {
            ui.label(&material.name);
            ui.label(format!("{}%", &material.percentage.to_string()));
            ui.end_row();
        }
        ui.heading("Composition");
        ui.end_row();
        for composition in &self.composition {
            ui.label(&composition.name);
            ui.label(format!("{}%", &composition.percentage.to_string()));
            ui.end_row();
        }
        ui.heading("Rings");
        ui.end_row();
        for ring in &self.asteroid_rings {
            ui.label(&ring.ring_class);
            ui.vertical(|ui| {
                ui.label(format!("{}km", &ring.outer_rad));
                ui.label(format!("{}km", &ring.inner_rad));
            });
            ui.end_row();
        }
    }
}

impl Ring {
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            let system_name = self.star_system.clone();
            body_name.replace_range(0..system_name.len(), "");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        if !self.ring_signals.is_empty() {
            for signal in &self.ring_signals {
                body_name.push('|');
                body_name.push_str(&signal.count.to_string());
                body_name.push(' ');
                let mut signal_name = signal.type_localised.clone();
                if &signal.type_localised.to_lowercase() == "null" {
                    signal_name = signal.r#type.clone();
                }
                signal_name.replace_range(signal_name.len() / 2..signal_name.len(), "");
                signal_name.push('.');
                body_name.push_str(signal_name.as_str());
            }
        }
        if self.settings.icons.get("distance").unwrap().enabled {
            ui.label("|");
            ui.label((self.distance_from_arrival_ls as u64).to_formatted_string(&Locale::en));
            ui.label(" LS");
            ui.label(self.settings.icons.get("distance").unwrap().get_richtext());
        }
        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled {
            ui.label("|");
            ui.label(
                self.settings
                    .icons
                    .get("discovered")
                    .unwrap()
                    .get_richtext(),
            );
        }
        if self.was_mapped && self.settings.icons.get("mapped").unwrap().enabled {
            ui.label("|");
            ui.label(self.settings.icons.get("mapped").unwrap().get_richtext());
        }
    }
    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        for signal in &self.ring_signals {
            let mut signal_name = signal.type_localised.clone();
            if &signal.type_localised.to_lowercase() == "null" {
                signal_name = signal.r#type.clone();
            }
            ui.label(signal_name);
            ui.label(signal.count.to_string());
            ui.end_row();
        }
    }
}

impl BeltCluster {
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        if self.settings.icons.get("belt_cluster").unwrap().enabled {
            ui.label(
                self.settings
                    .icons
                    .get("belt_cluster")
                    .unwrap()
                    .get_richtext(),
            );
        }
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            let system_name = self.star_system.clone();
            body_name.replace_range(0..system_name.len(), "");
        }

        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled {
            ui.label("|");
            ui.label(
                self.settings
                    .icons
                    .get("discovered")
                    .unwrap()
                    .get_richtext(),
            );
        }
        if self.was_mapped && self.settings.icons.get("mapped").unwrap().enabled {
            ui.label("|");
            ui.label(self.settings.icons.get("mapped").unwrap().get_richtext());
        }
    }

    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
    }
}
