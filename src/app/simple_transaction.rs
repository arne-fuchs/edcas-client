use eframe::{egui, Frame};
use crate::egui::Context;

pub struct SimpleTransaction{
    value : u64,
    seed: String,
    receiver: String,
    result: String,
}

impl Default for SimpleTransaction{
    fn default() -> Self {
        Self {
            value: 10,
            seed: "52d23081a626b1eca34b63f1eaeeafcbd66bf545635befc12cd0f19926efefb031f176dadf38cdec0eadd1d571394be78f0bbee3ed594316678dffc162a095cb".to_owned(),
            receiver: "60200bad8137a704216e84f8f9acfe65b972d9f4155becb4815282b03cef99fe".to_owned(),
            result: "".to_owned(),
        }
    }
}

pub fn update(data: &mut SimpleTransaction, ctx: &Context, _frame: &mut Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Simple transaction");
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Amount in IOTA:");
            ui.add(egui::DragValue::new(&mut data.value).speed(1.0));
        });
        ui.end_row();

        ui.horizontal(|ui| {
            ui.label("Address:");
            ui.text_edit_singleline(&mut data.receiver);
        });
        ui.end_row();



        //Send transaction
        if ui.button("Send transaction").clicked(){

        };
        ui.end_row();
        ui.label(&data.result)
    });
}