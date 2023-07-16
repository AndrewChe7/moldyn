use egui::Context;
use crate::visualizer::{UiData, VisualizationParameterType};

pub fn main_window_ui(ui_data: &mut UiData, ctx: &Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        main_menu(ui);
    });
    egui::Window::new("Visualization parameters").show(ctx, |ui| {
        ui.color_edit_button_srgb(&mut ui_data.color_0);
        ui.color_edit_button_srgb(&mut ui_data.color_05);
        ui.color_edit_button_srgb(&mut ui_data.color_1);
        egui::ComboBox::from_label("Parameter")
            .selected_text(format!("{:?}", &ui_data.visualization_parameter_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ui_data.visualization_parameter_type,
                                    VisualizationParameterType::Type, "Particle type");
                ui.selectable_value(&mut ui_data.visualization_parameter_type,
                                    VisualizationParameterType::Velocity, "Velocity");
                ui.selectable_value(&mut ui_data.visualization_parameter_type,
                                    VisualizationParameterType::Pressure, "Pressure");
        });
    });
}

fn main_menu(ui: &mut egui::Ui) {
    use egui::menu;

    menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                // …
            }
        });
    });
}