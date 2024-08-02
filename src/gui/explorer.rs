use crate::edcas::explorer::body::{BodyType, Parent};
use crate::edcas::explorer::Explorer;
use eframe::egui::collapsing_header::{CollapsingState, HeaderResponse};
use eframe::egui::{Align, Context, Direction, Layout, Widget};
use eframe::{egui, App, Frame};

impl App for Explorer {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        if self.systems.len() > self.index {
            egui::SidePanel::left("system_data").show(ctx, |ui| {
                self.systems[self.index].draw_system_info(ui);
                ui.separator();
                ui.heading("Body Signals");
                egui::ScrollArea::vertical()
                    .id_source("body_signal_scroll_area")
                    .show(ui, |ui| {
                        self.systems[self.index].draw_body_signal_list(ui);
                    });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("page_grid")
                    .num_columns(4)
                    .striped(true)
                    .min_col_width(200.0)
                    .max_col_width(200.0)
                    .show(ui, |ui| {
                        ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| {
                            if ui.button("<-").clicked() && self.index != 0 {
                                self.index -= 1;
                            }
                        });

                        ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                            ui.heading(&self.systems[self.index].name)
                        });

                        ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
                            if ui.button("->").clicked() && self.index + 1 < self.systems.len() {
                                self.index += 1;
                            }
                        });
                        //ui.add_space(ui.available_size_before_wrap().x);
                        ui.end_row();
                    });

                egui::ScrollArea::vertical()
                    .id_source("body_scroll_area")
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
                            let finished_bodies_ids = build_tree(
                                &mut system.body_list,
                                &mut system.index,
                                current_parent,
                                ui,
                                vec![],
                            );
                            for i in 0..system.body_list.len() {
                                let body = &system.body_list[i];
                                let mut found = false;
                                for id in &finished_bodies_ids {
                                    if id == &body.get_id() {
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    let id = ui.make_persistent_id(body.get_name());
                                    let entry: CollapsingState =
                                        CollapsingState::load_with_default_open(ui.ctx(), id, true);
                                    let header: HeaderResponse<()> = entry.show_header(ui, |ui| {
                                        body.print_header_content(ui, &mut system.index, i);
                                    });
                                    header.body(|_ui| {});
                                }
                            }
                        });
                    });
            });

            egui::SidePanel::right("body_data").show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_source("body_data_scroll_area")
                    .show(ui, |ui| {
                        egui::Grid::new("system_data_grid")
                            .num_columns(2)
                            .striped(true)
                            .min_col_width(200.0)
                            .max_col_width(200.0)
                            .show(ui, |ui| match self.systems.last() {
                                None => {}
                                Some(system) => match system.body_list.get(system.index) {
                                    None => {}
                                    Some(body) => {
                                        body.print_side_panel_information(ui);
                                    }
                                },
                            });
                    });
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::LeftToRight), |ui|{
                  ui.heading("No Data");  
                });
            });
        }
    }
}

fn build_tree(
    body_list: &mut Vec<BodyType>,
    system_index: &mut usize,
    current_parent: Parent,
    ui: &mut egui::Ui,
    parent_ids: Vec<u64>,
) -> Vec<u64> {
    let mut finished_bodies: Vec<u64> = vec![];
    for i in 0..body_list.len() {
        //look what the largest body id in parents is. If is the current parent, print the header
        let mut largest_parent_id = 0;
        let parents = body_list[i].get_parents();
        for parent in parents {
            //println!("{}: {}",parent.name, parent.id);
            if parent.id > largest_parent_id && (parent.id > 0 && parent.name.as_str() != "Null") {
                largest_parent_id = parent.id;
            }
        }

        //If it is in the parents list, this body has already been drawn
        if largest_parent_id == current_parent.id && !parent_ids.contains(&largest_parent_id) {
            let id = ui.make_persistent_id(body_list[i].get_name());
            let entry: CollapsingState =
                CollapsingState::load_with_default_open(ui.ctx(), id, true);

            let header: HeaderResponse<()> = entry.show_header(ui, |ui| {
                body_list[i].print_header_content(ui, system_index, i);
            });

            header.body(|ui| {
                //Build header with this body as parent
                let this_as_parent = Parent {
                    name: "".to_string(),
                    id: body_list[i].get_id(),
                };
                let mut this_parents = parent_ids.clone();
                this_parents.push(largest_parent_id);
                let finished_ids: Vec<u64> =
                    build_tree(body_list, system_index, this_as_parent, ui, this_parents);
                for finished_body_id in finished_ids {
                    if !finished_bodies.contains(&finished_body_id) {
                        finished_bodies.push(finished_body_id);
                    }
                }
            });
            if !finished_bodies.contains(&body_list[i].get_id()) {
                finished_bodies.push(body_list[i].get_id());
            }
        }
    }
    finished_bodies
}
