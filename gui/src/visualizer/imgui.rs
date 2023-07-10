use egui::Context;

pub fn main_window_ui(ctx: &Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        main_menu(ui);
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