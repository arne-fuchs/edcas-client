use std::ops::{Add, Deref};
use eframe::{egui, Frame};
use eframe::egui::{Align, Direction, Label, Layout, TextStyle, TextureHandle, Ui};
use eframe::egui::collapsing_header::{CollapsingState, HeaderResponse};
use iota_wallet::iota_client::crypto::ciphers::traits::consts::Exp;
use crate::app::explorer::body::{get_body_class_from_body};
use crate::app::explorer::body::Icons as BodyIcons;
use crate::app::explorer::icons::{get_icon_from_string, Icons};
use crate::egui::{Context};

mod body;
mod icons;

pub struct Explorer {
    pub pages: Vec<Page>,
    pub index: usize,
    pub system: System,
    pub system_signal_list: Vec<SystemSignal>,
    pub body_list: Vec<Body>,
    pub body_signal_list: Vec<BodySignal>,
    pub body: Body,
    pub body_icons: body::Icons,
    pub body_signal_icons: Icons,
    pub ctx: Context,
}

impl Default for Explorer {
    fn default() -> Self {
        let page = Page{
            system:  System::default(),
            body_list: vec![],
            body_signal_list: vec![],
            system_signal_list: vec![],
            body: Default::default(),
        };
        Self {
            pages: vec![page],
            index : 0,
            system:  System::default(),
            system_signal_list: Vec::new(),
            body_list: Vec::new(),
            body_signal_list: Vec::new(),
            body: Body::default(),
            body_icons: body::Icons::default(),
            body_signal_icons: Icons::default(),
            ctx: Context::default(),
        }
    }
}

pub struct Page{
    pub system: System,
    pub body_list: Vec<Body>,
    pub body_signal_list: Vec<BodySignal>,
    pub system_signal_list: Vec<SystemSignal>,
    pub body: Body,
}

#[derive(Clone)]
pub struct System {
    pub name: String,
    pub allegiance: String,
    pub economy_localised: String,
    pub second_economy_localised: String,
    pub government_localised: String,
    pub security_localised: String,
    pub population: String,
    pub body_count: String,
    pub non_body_count: String,
}

impl Default for System {
    fn default() -> Self {
        Self {
            name: "N/A".to_string(),
            allegiance: "N/A".to_string(),
            economy_localised: "N/A".to_string(),
            second_economy_localised: "N/A".to_string(),
            government_localised: "N/A".to_string(),
            security_localised: "N/A".to_string(),
            population: "N/A".to_string(),
            body_count: "N/A".to_string(),
            non_body_count: "N/A".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct SystemSignal{
    pub timestamp: String,
    pub event: String,
    pub name: String,
    pub thread: String
}

#[derive(Clone, Debug)]
pub struct Body {
    pub name: String,
    pub body_id: String,
    pub parents: String,
    pub star_system: String,
    pub system_address: String,
    pub distance_from_arrival_ls: String,
    pub tidal_lock: String,
    pub terraform_state: String,
    pub planet_class: String,
    pub atmosphere: String,
    pub atmosphere_composition: String,
    pub volcanism: String,
    pub mass_em: String,
    pub radius: String,
    pub surface_gravity: String,
    pub surface_temperature: String,
    pub surface_pressure: String,
    pub landable: String,
    pub semi_major_axis: String,
    pub eccentricity: String,
    pub orbital_inclination: String,
    pub periapsis: String,
    pub orbital_period: String,
    pub ascending_node: String,
    pub mean_anomaly: String,
    pub rotation_period: String,
    pub axial_tilt: String,
    pub was_discovered: String,
    pub was_mapped: String,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            name: "N/A".to_string(),
            body_id: "N/A".to_string(),
            parents: "N/A".to_string(),
            star_system: "N/A".to_string(),
            system_address: "N/A".to_string(),
            distance_from_arrival_ls: "N/A".to_string(),
            tidal_lock: "N/A".to_string(),
            terraform_state: "N/A".to_string(),
            planet_class: "N/A".to_string(),
            atmosphere: "N/A".to_string(),
            atmosphere_composition: "N/A".to_string(),
            volcanism: "N/A".to_string(),
            mass_em: "N/A".to_string(),
            radius: "N/A".to_string(),
            surface_gravity: "N/A".to_string(),
            surface_temperature: "N/A".to_string(),
            surface_pressure: "N/A".to_string(),
            landable: "N/A".to_string(),
            semi_major_axis: "N/A".to_string(),
            eccentricity: "N/A".to_string(),
            orbital_inclination: "N/A".to_string(),
            periapsis: "N/A".to_string(),
            orbital_period: "N/A".to_string(),
            ascending_node: "N/A".to_string(),
            mean_anomaly: "N/A".to_string(),
            rotation_period: "N/A".to_string(),
            axial_tilt: "N/A".to_string(),
            was_discovered: "N/A".to_string(),
            was_mapped: "N/A".to_string(),
        }
    }
}
#[derive(Clone)]
pub struct Signal {
    pub r#type: String,
    pub type_localised: String,
    pub count: i64,
}
#[derive(Clone)]
pub struct BodySignal {
    pub timestamp: String,
    pub event: String,
    pub body_name: String,
    pub body_id: i64,
    pub system_address: i64,
    pub signals: Vec<Signal>,
}

impl Default for BodySignal {
    fn default() -> Self {
        Self{
            timestamp: "N/A".to_string(),
            event: "N/A".to_string(),
            body_name: "N/A".to_string(),
            body_id: -1,
            system_address: -1,
            signals: vec![]
        }
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self{
            r#type: "N/A".to_string(),
            type_localised: "N/A".to_string(),
            count: 0
        }
    }
}

pub fn update(explorer: &mut Explorer, ctx: &Context, _frame: &mut Frame) {
    explorer.ctx = ctx.clone();

    let result = explorer.pages.get(explorer.index);
    let mut system;
    let mut body_list;
    let mut body_signal_list;
    let mut signal_list;
    match result {
        None => {
            system = explorer.system.clone();
            body_list = explorer.body_list.clone();
            signal_list = explorer.system_signal_list.clone();
            body_signal_list = explorer.body_signal_list.clone();
        }
        Some(page) => {
            system = page.system.clone();
            body_list = page.body_list.clone();
            signal_list = page.system_signal_list.clone();
            body_signal_list = page.body_signal_list.clone();
        }
    }
    drop(result);

    egui::SidePanel::left("system_data").show(ctx, |ui| {
        draw_planet_count(explorer,ui);
        ui.separator();
        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                draw_system_signal_list(&signal_list, ui);
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::Grid::new("page_grid")
            .num_columns(4)
            .striped(true)
            .min_col_width(100.0)
            .max_col_width(300.0)
            .show(ui,|ui|{

                ui.with_layout(Layout::left_to_right(Align::RIGHT), |ui| {
                    if ui.button("<-").clicked() {
                        if explorer.index != 0 {
                            explorer.index = explorer.index - 1;
                        }
                    }
                });

                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.heading(&explorer.system.name)
                });

                ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                    if ui.button("->").clicked() {
                        if explorer.index + 1 < explorer.pages.len() {
                            explorer.index = explorer.index + 1;
                        }
                    }
                });

                ui.add_space(ui.available_size_before_wrap().x);
            });

        egui::ScrollArea::vertical()
            .stick_to_right(true)
            .show(ui, |ui| {
                //Fixme Without separator scrollbar is in the middle of field. With it, it doesnt show at all
                ui.separator();

                //If queue wasn't empty, headers will be rebuild
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    build_headers(ui, explorer);
                });

            });
    });

    egui::SidePanel::right("body_data").show(ctx, |ui| {
        egui::Grid::new("system_data_grid")
            .num_columns(2)
            .striped(true)
            .spacing([10.0, 5.0])
            .min_col_width(200.0)
            .show(ui, |ui| {
                body::draw_body_info(explorer, ui);
            });
        ui.heading("Body Signals");
        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                draw_body_signal_list(&explorer.system, &body_signal_list, ui, &explorer.body_signal_icons);
            });
    });
}

fn draw_system_info(system: &System, ui: &mut Ui) {
    ui.label("Allegiance");
    ui.label(&system.allegiance);
    ui.end_row();

    ui.label("Economy");
    ui.label(&system.economy_localised);
    ui.end_row();

    ui.label("sec. Economy");
    ui.label(&system.second_economy_localised);
    ui.end_row();

    ui.label("Government");
    ui.label(&system.government_localised);
    ui.end_row();

    ui.label("Security");
    ui.label(&system.security_localised);
    ui.end_row();

    ui.label("Population");
    ui.label(&system.population);
    ui.end_row();
}

fn draw_planet_count(explorer: &Explorer, ui: &mut Ui) {
    let result = explorer.pages.get(explorer.index);
    let mut system;
    let mut body_list;
    match result {
        None => {
            system = &explorer.system;
            body_list = &explorer.body_list;
        }
        Some(page) => {
            system = &page.system;
            body_list = &page.body_list;
        }
    }

    egui::Grid::new("system_data_grid")
        .num_columns(2)
        .striped(true)
        .min_col_width(200.0)
        .show(ui, |ui| {
            draw_system_info(system, ui);
        });

    ui.separator();
    egui::Grid::new("body_count_grid")
        .num_columns(2)
        .striped(true)
        .min_col_width(200.0)
        .show(ui, |ui| {
            ui.label("Bodies");
            ui.label(&system.body_count);
            ui.end_row();
            ui.label("Non-bodies");
            ui.label(&system.non_body_count);
            ui.end_row();
        });


    if !system.body_count.eq("N/A") {
        ui.add(egui::ProgressBar::new(body_list.len() as f32 / (&system.body_count.parse::<f32>().unwrap()+&system.non_body_count.parse::<f32>().unwrap()))
            .text(body_list.len().to_string().add("/").add((&system.body_count.parse::<f32>().unwrap()+&system.non_body_count.parse::<f32>().unwrap()).to_string().as_str()))
        );
    }
    ui.end_row();
}
fn draw_system_signal_list(system_signal_list: &Vec<SystemSignal>, ui: &mut Ui) {

    egui::Grid::new("system_signal_grid")
        .num_columns(2)
        .striped(true)
        .min_col_width(130.0)
        .show(ui, |ui| {
            ui.label("Name");
            ui.label("Thread");
            ui.end_row();
            for system_signal in system_signal_list{
                ui.label(&system_signal.name);
                ui.label(&system_signal.thread);

                ui.end_row();
            }
        });
}

fn draw_body_signal_list(system: &System, body_signal_list: &Vec<BodySignal>, ui: &mut Ui, icons: &Icons) {
    egui::Grid::new("body_signal_grid")
        .num_columns(3)
        .striped(true)
        .min_col_width(130.0)
        .show(ui, |ui| {
            ui.label("Body");
            ui.label("Type");
            ui.label("Count");
            ui.end_row();
            for body_signal in body_signal_list{
                for signal in &body_signal.signals{
                    ui.label(body_signal.body_name.trim_start_matches(&system.name));
                    ui.label(&signal.type_localised);

                    let id = body_signal.body_name.clone().add(&signal.r#type.to_string().clone());

                    egui::Grid::new(id)
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(signal.count.to_string());
                            let texture: TextureHandle = ui.ctx().load_texture(
                                "body-signal-icon",
                                get_icon_from_string(signal.r#type.clone(),&icons).clone(),
                                egui::TextureOptions::LINEAR,
                            );

                            let img_size = 32.0 * texture.size_vec2() / texture.size_vec2().y;
                            ui.image(&texture, img_size);
                        });
                    ui.end_row();
                }
            }
        });
}

fn build_headers(ui: &mut Ui, explorer: &mut Explorer) {
    let result = explorer.pages.get(explorer.index);
    let mut body_list;
    match result {
        None => {
            body_list = &explorer.body_list;
        }
        Some(page) => {
            body_list = &page.body_list;
        }
    }

    if !body_list.is_empty() {
        let cloned_list = &body_list.clone();

        //Get all stars first -> then attach childs
        let mainstar_list = cloned_list.iter().filter(|body| {
            let parents_array = json::parse(&*body.parents).unwrap();

            let mut i = 0;
            let mut parent_object = &parents_array[i];
            while !parent_object.is_null() {
                i = i + 1;
                for entry in parent_object.entries() {
                    if !entry.0.eq("Null") {
                        return false;
                    }
                }
                parent_object = &parents_array[i];
            }
            true
        });

        let body_list_copy = body_list.clone();
        drop(result);

        for parentless_body in mainstar_list {
            let id = ui.make_persistent_id(&parentless_body.name);
            let entry: CollapsingState =
                CollapsingState::
                load_with_default_open(ui.ctx(), id, true);

            let header: HeaderResponse<()> = entry.show_header(ui, |ui| {

                let texture: TextureHandle = ui.ctx().load_texture(
                    "parentless-body-icon",
                    body::get_color_image_from_planet_class(get_body_class_from_body(parentless_body), &explorer.body_icons).clone(),
                    egui::TextureOptions::LINEAR,
                );

                let img_size = 32.0 * texture.size_vec2() / texture.size_vec2().y;
                ui.image(&texture, img_size);
                if ui.selectable_label(false, &parentless_body.name).clicked() {
                    explorer.body = parentless_body.clone();
                };
            });

            header.body(|ui| {
                attach_all_childs(body_list_copy.clone(), parentless_body.body_id.parse().unwrap(), ui, explorer);
            });
        }
    }
}

fn attach_all_childs(body_list: Vec<Body>, parent_id: i32, ui: &mut Ui, explorer: &mut Explorer) {
    let cloned_list = body_list.clone();
    let filtered_list = cloned_list.iter().filter(|body| {
        let parents_array = json::parse(&*body.parents).unwrap();
        if parents_array.is_empty() { return false; }
        //Filters all planets out, which are not a child by the given parent_id.
        parents_array[0].entries().any(|entry| {
            //If first is null the second parent has to be looked at
            //Second parent has to be there because otherwise it has to be a main star -> main stars are handled in build_headers
            if entry.0.eq("Null") {
                parents_array[1].entries().any(|entry| {
                    entry.1.as_i32().unwrap() == parent_id
                })
            } else {
                entry.1.as_i32().unwrap() == parent_id
            }
        })
    });
    for body in filtered_list {
        let id = ui.make_persistent_id(&body.name);

        let entry = CollapsingState::
        load_with_default_open(ui.ctx(), id, true);

        let header_response: HeaderResponse<()> = entry.show_header(ui, |ui| {

            let texture: TextureHandle = ui.ctx().load_texture(
                "body-icon",
                body::get_color_image_from_planet_class(get_body_class_from_body(&body), &explorer.body_icons).clone(),
                egui::TextureOptions::LINEAR,
            );

            let img_size = 32.0 * texture.size_vec2() / texture.size_vec2().y;
            ui.image(&texture, img_size);
            let button = ui.selectable_label(false, &body.name);
            if button.clicked() {
                explorer.body = body.clone();
            };
        });

        header_response.body(|ui| {
            attach_all_childs(body_list.to_owned(), body.body_id.parse().unwrap(), ui, explorer);
        });
    }
}