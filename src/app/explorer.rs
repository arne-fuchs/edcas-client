use std::sync::Arc;
use eframe::{App, egui, Frame};
use eframe::egui::{Align, Layout};
use eframe::egui::collapsing_header::{CollapsingState, HeaderResponse};
use crate::app::explorer::structs::{BodyImplementation, Parent};
use crate::app::explorer::system::System;
use crate::app::settings::Settings;
use crate::egui::Context;

pub mod planet;
pub mod star;
pub mod structs;
pub mod belt_cluster;
pub mod system;
mod ring;

pub struct Explorer {
    pub systems: Vec<System>,
    pub index: usize,
    pub body_list_index: Option<usize>,
    pub settings: Arc<Settings>,
}

impl App for Explorer {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.systems.len() > self.index {
            egui::SidePanel::left("system_data").show(ctx, |ui| {
                self.systems[self.index].draw_system_info(ui);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("page_grid")
                    .num_columns(4)
                    .striped(true)
                    .min_col_width(200.0)
                    .max_col_width(200.0)
                    .show(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::RIGHT), |ui| {
                            if ui.button("<-").clicked() && self.index != 0 {
                                self.index -= 1;
                            }
                        });

                        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                            ui.heading(&self.systems[self.index].name)
                        });

                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            if ui.button("->").clicked() && self.index + 1 < self.systems.len() {
                                self.index += 1;
                            }
                        });
                        ui.add_space(ui.available_size_before_wrap().x);
                    });

                egui::ScrollArea::vertical()
                    .stick_to_right(true)
                    .show(ui, |ui| {
                        //Fixme Without separator scrollbar is in the middle of field. With it, it doesnt show at all
                        ui.separator();

                        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                            let current_parent = Parent {
                                name: "Null".to_string(),
                                id: 0,
                            };
                            let system = &mut self.systems[self.index];
                            let finished_bodies_ids = build_tree(&mut system.body_list, &mut system.index,current_parent, ui,vec![]);
                            for i in 0..system.body_list.len(){
                                let body = &system.body_list[i];
                                let mut found = false;
                                for id in &finished_bodies_ids{
                                    if id == &body.get_id() {
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    let id = ui.make_persistent_id(body.get_name());
                                    let entry: CollapsingState =
                                        CollapsingState::
                                        load_with_default_open(ui.ctx(), id, true);
                                    let header: HeaderResponse<()> = entry.show_header(ui, |ui| {
                                        body.print_header_content(ui,&mut system.index,i);
                                    });
                                    header.body(|_ui| {});
                                }
                            }
                        });
                    });
            });

            egui::SidePanel::right("body_data").show(ctx, |ui| {
                egui::Grid::new("system_data_grid")
                    .num_columns(2)
                    .striped(true)
                    .min_col_width(200.0)
                    .max_col_width(200.0)
                    .show(ui, |ui| {
                        match self.systems.last() {
                            None => {}
                            Some(system) => {
                                match system.body_list.get(system.index) {
                                    None => {}
                                    Some(body) => {
                                        body.print_side_panel_information(ui);
                                    }
                                }
                            }
                        }
                    });
                ui.separator();
                ui.heading("Body Signals");
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        self.systems[self.index].draw_body_signal_list(ui);
                    });
            });
        }
    }
}

fn build_tree(body_list: &mut Vec<Box<dyn BodyImplementation>>,system_index: &mut usize, current_parent: Parent, ui: &mut egui::Ui,parent_ids: Vec<i64>) -> Vec<i64>{
    let mut finished_bodies = vec![];
    for i in 0..body_list.len() {
        //look what the largest body id in parents is. If is the current parent, print the header
        let mut largest_parent_id = 0;
        let parents = body_list[i].get_parents();
        for parent in parents {
            println!("{}: {}",parent.name, parent.id);
            if parent.id > largest_parent_id && (parent.id > 0 && parent.name.as_str() != "Null") {
                largest_parent_id = parent.id;
            }
        }

        //If it is in the parents list, this body has already been drawn
        if largest_parent_id == current_parent.id && !parent_ids.contains(&largest_parent_id){
            let id = ui.make_persistent_id(body_list[i].get_name());
            let entry: CollapsingState =
                CollapsingState::
                load_with_default_open(ui.ctx(), id, true);

            let header: HeaderResponse<()> = entry.show_header(ui, |ui| {
                body_list[i].print_header_content(ui,system_index,i);
            });

            header.body(|ui| {
                //Build header with this body as parent
                let this_as_parent = Parent{
                    name: "".to_string(),
                    id: body_list[i].get_id(),
                };
                let mut this_parents = parent_ids.clone();
                this_parents.push(largest_parent_id);
                let finished_ids = build_tree(body_list,system_index,this_as_parent,ui,this_parents);
                for finished_body_id in finished_ids {
                    if !finished_bodies.contains(&finished_body_id){
                        finished_bodies.push(finished_body_id);
                    }
                }
            });
            if !finished_bodies.contains(&body_list[i].get_id()){
                finished_bodies.push(body_list[i].get_id());
            }
        }
    }
    finished_bodies
}