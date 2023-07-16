use std::fs;
use egui::Context;
use rfd::AsyncFileDialog;
use crate::visualizer::{UiData, VisualizationParameterType};

pub fn main_window_ui(ui_data: &mut UiData, ctx: &Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        main_menu(ui, ui_data);
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
        if ui_data.file_path.is_some() {
            ui.add(egui::Slider::new(&mut ui_data.frame_index,
                                     0..=ui_data.last_frame_from_all)
                .text("Frame"));
            ui.checkbox(&mut ui_data.play, "Play");
            ui.add(egui::Slider::new(&mut ui_data.play_speed, 1..=100)
                .text("Play speed"));
        }
    });
}

fn main_menu(ui: &mut egui::Ui, ui_data: &mut UiData,) {
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
                            let in_file = df.path();
                            let file_dir = in_file.parent().expect("Can't get project folder");
                            let paths = fs::read_dir(file_dir).expect("Can't read directory");
                            let file_path_without_ext = in_file
                                .with_extension("")
                                .with_extension("");
                            let mut sizes: Vec<usize> = vec![];
                            for path in paths {
                                let path = path.expect("Can't get file");
                                let path = path.path();
                                if path.with_extension("").with_extension("") == file_path_without_ext {
                                    if let Some(extension) = path.with_extension("").extension() {
                                        let extension_string = extension.to_str()
                                            .expect(format!("Can't convert to str {:?}", extension).as_str());
                                        if let Ok(last) = extension_string.parse() {
                                            sizes.push(last);
                                        }
                                    }
                                }
                            }
                            let end = sizes.iter().max().unwrap();
                            ui_data.last_frame_from_all = end.clone() - 1;
                            let _ = ui_data.file_path.insert(in_file.to_path_buf());
                            ui_data.files = sizes;
                        }
                    };
                    pollster::block_on(future);
                }
            }
        });
    });
}