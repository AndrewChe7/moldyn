use egui::Context;
use rfd::AsyncFileDialog;
use moldyn_core::DataFile;
use crate::visualizer::{UiData, VisualizationParameterType};

pub fn main_window_ui(ui_data: &mut UiData, ctx: &Context, data_file: &mut Option<DataFile>) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        main_menu(ui, data_file);
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
        if let Some(data_file) = data_file {
            ui.add(egui::Slider::new(&mut ui_data.frame_index,
                 data_file.start_frame..=(data_file.start_frame + data_file.frame_count - 1))
                .text("Frame"));
            ui.checkbox(&mut ui_data.play, "Play");
            ui.add(egui::Slider::new(&mut ui_data.play_speed, 1..=100)
                .text("Play speed"));
        }
    });
}

fn main_menu(ui: &mut egui::Ui, data_file: &mut Option<DataFile>) {
    use egui::menu;

    menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let future = async {
                        let df = AsyncFileDialog::new()
                            .add_filter("simulation", &["json"])
                            .pick_file()
                            .await;
                        if let Some(df) = df {
                            let _ = data_file.insert(DataFile::load_from_file(df.path()));
                        }
                    };
                    pollster::block_on(future);
                }
            }
        });
    });
}