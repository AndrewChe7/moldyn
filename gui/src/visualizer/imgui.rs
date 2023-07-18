use std::fs;
use std::ops::RangeInclusive;
use egui::Context;
use egui::plot::{Corner, Legend, Line, Plot, PlotPoints};
use itertools::Itertools;
use rfd::AsyncFileDialog;
use moldyn_core::DataFileMacro;
use crate::visualizer::{ChoosePlot, MacroPlots, UiData, VisualizationParameterType};

pub fn main_window_ui(ui_data: &mut UiData, ctx: &Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        main_menu(ui, ui_data);
    });
    if ui_data.show_plot && ui_data.macro_plots.is_some() {
        plot(ctx, ui_data);
    }
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
                            ui_data.macro_plots = None;
                            let mut macro_data = None;
                            for path in paths {
                                let path = path.expect("Can't get file");
                                let path = path.path();
                                let path_no_json = path.with_extension("");
                                if path_no_json.with_extension("") == file_path_without_ext {
                                    if let Some(extension) = path_no_json.extension() {
                                        if let Some(extension_string) = extension.to_str() {
                                            if let Ok(last) = extension_string.parse() {
                                                sizes.push(last);
                                            }
                                        }
                                    }
                                } else if path_no_json.with_extension("").with_extension("") == file_path_without_ext &&
                                    path_no_json.extension().unwrap() == "macro" {
                                    let data = DataFileMacro::load_from_file(&path);
                                    if macro_data.is_none() {
                                        let _ = macro_data.insert(data);
                                    } else {
                                        let macro_data = macro_data.as_mut().unwrap();
                                        macro_data.append_data(&data);
                                    }
                                }
                            }
                            if macro_data.is_some() {
                                let _ = ui_data.macro_plots.insert(MacroPlots::new());
                                let macro_plots = ui_data.macro_plots.as_mut().unwrap();
                                macro_data.as_ref().unwrap()
                                    .macro_parameters.iter()
                                    .sorted_by_key(|x| x.0)
                                    .for_each(|(x, y)| {
                                        macro_plots.kinetic_energy.push([x.clone() as f64, y.kinetic_energy]);
                                        macro_plots.potential_energy.push([x.clone() as f64, y.potential_energy]);
                                        macro_plots.thermal_energy.push([x.clone() as f64, y.thermal_energy]);
                                        macro_plots.full_energy.push([x.clone() as f64, y.kinetic_energy + y.potential_energy]);
                                        macro_plots.internal_energy.push([x.clone() as f64, y.thermal_energy + y.potential_energy]);
                                        macro_plots.unit_kinetic_energy.push([x.clone() as f64, y.unit_kinetic_energy]);
                                        macro_plots.unit_potential_energy.push([x.clone() as f64, y.unit_potential_energy]);
                                        macro_plots.unit_thermal_energy.push([x.clone() as f64, y.unit_thermal_energy]);
                                        macro_plots.unit_full_energy.push([x.clone() as f64, y.unit_kinetic_energy + y.unit_potential_energy]);
                                        macro_plots.unit_internal_energy.push([x.clone() as f64, y.unit_thermal_energy + y.unit_potential_energy]);
                                        macro_plots.pressure.push([x.clone() as f64, y.pressure]);
                                        macro_plots.temperature.push([x.clone() as f64, y.temperature]);
                                    });
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
        ui.menu_button("Macro parameters", |ui| {
            if ui.add(egui::Button::new("Recalculate macro params")).clicked() {
                // TODO:
            }
            if ui.add_enabled(ui_data.macro_plots.is_some(), egui::Button::new("Plot")).clicked() {
                ui_data.show_plot = true;
            }
        });
    });
}

fn plot (ctx: &Context, ui_data: &mut UiData) {
    egui::Window::new("Plot").open(&mut ui_data.show_plot).show(ctx, |ui| {
        egui::ComboBox::from_label("Macro parameter")
            .selected_text(format!("{:?}", &ui_data.plot_to_draw))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ui_data.plot_to_draw,
                                    ChoosePlot::Energy, "Energy");
                ui.selectable_value(&mut ui_data.plot_to_draw,
                                    ChoosePlot::UnitEnergy, "Unit energy");
                ui.selectable_value(&mut ui_data.plot_to_draw,
                                    ChoosePlot::Pressure, "Pressure");
                ui.selectable_value(&mut ui_data.plot_to_draw,
                                                ChoosePlot::Temperature, "Temperature");
            });
        let macro_plots = ui_data.macro_plots.as_ref().unwrap();
        let x_fmt = |x, _range: &RangeInclusive<f64>| {
            format!("iteration: {x}")
        };

        match ui_data.plot_to_draw {
            ChoosePlot::Energy => {
                let y_fmt = |y, _range: &RangeInclusive<f64>| {
                    format!("{y} ...")
                };
                let plot = Plot::new("Macro parameters")
                    .legend(Legend::default().position(Corner::RightTop))
                    .x_axis_formatter(x_fmt)
                    .y_axis_formatter(y_fmt);
                plot.show(ui, |plot_ui| {
                    let kinetic = Line::new(PlotPoints::new(macro_plots.kinetic_energy.clone()))
                        .name("Kinetic energy");
                    let potential =  Line::new(PlotPoints::new(macro_plots.potential_energy.clone()))
                        .name("Potential energy");
                    let thermal =  Line::new(PlotPoints::new(macro_plots.thermal_energy.clone()))
                        .name("Thermal energy");
                    let full =  Line::new(PlotPoints::new(macro_plots.full_energy.clone()))
                        .name("Full energy");
                    let internal =  Line::new(PlotPoints::new(macro_plots.internal_energy.clone()))
                        .name("Internal energy");
                    plot_ui.line(kinetic);
                    plot_ui.line(potential);
                    plot_ui.line(thermal);
                    plot_ui.line(full);
                    plot_ui.line(internal);
                });
            }
            ChoosePlot::UnitEnergy => {
                let y_fmt = |y, _range: &RangeInclusive<f64>| {
                    format!("{y} ...")
                };
                let plot = Plot::new("Macro parameters")
                    .legend(Legend::default().position(Corner::RightTop))
                    .x_axis_formatter(x_fmt)
                    .y_axis_formatter(y_fmt);
                plot.show(ui, |plot_ui| {
                    let kinetic = Line::new(PlotPoints::new(macro_plots.unit_kinetic_energy.clone()))
                        .name("Unit kinetic energy");
                    let potential =  Line::new(PlotPoints::new(macro_plots.unit_potential_energy.clone()))
                        .name("Unit potential energy");
                    let thermal =  Line::new(PlotPoints::new(macro_plots.unit_thermal_energy.clone()))
                        .name("Unit thermal energy");
                    let full =  Line::new(PlotPoints::new(macro_plots.unit_full_energy.clone()))
                        .name("Unit full energy");
                    let internal =  Line::new(PlotPoints::new(macro_plots.unit_internal_energy.clone()))
                        .name("Unit internal energy");
                    plot_ui.line(kinetic);
                    plot_ui.line(potential);
                    plot_ui.line(thermal);
                    plot_ui.line(full);
                    plot_ui.line(internal);
                });
            }
            ChoosePlot::Temperature => {
                let y_fmt = |y, _range: &RangeInclusive<f64>| {
                    format!("{y} K")
                };
                let plot = Plot::new("Macro parameters")
                    .legend(Legend::default().position(Corner::RightTop))
                    .x_axis_formatter(x_fmt)
                    .y_axis_formatter(y_fmt);
                plot.show(ui, |plot_ui| {
                    let line = Line::new(PlotPoints::new(macro_plots.temperature.clone()));
                    plot_ui.line(line);
                });
            }
            ChoosePlot::Pressure => {
                let y_fmt = |y, _range: &RangeInclusive<f64>| {
                    format!("{y} pressure units")
                };
                let plot = Plot::new("Macro parameters")
                    .legend(Legend::default().position(Corner::RightTop))
                    .x_axis_formatter(x_fmt)
                    .y_axis_formatter(y_fmt);
                plot.show(ui, |plot_ui| {
                    let line = Line::new(PlotPoints::new(macro_plots.pressure.clone()));
                    plot_ui.line(line);
                });
            }
        }
    });
}