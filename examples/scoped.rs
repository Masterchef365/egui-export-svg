use egui_export_svg::capture_scope;

fn main() -> eframe::Result<()> {
    // Our application state:
    let mut name = "Arthur".to_owned();
    let mut age = 42;

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Egui export SVG", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            let maybe_svg = capture_scope(ui, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Your name: ");
                    ui.text_edit_singleline(&mut name)
                        .labelled_by(name_label.id);
                });

                ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
                if ui.button("Increment").clicked() {
                    age += 1;
                }
                ui.label(format!("Hello '{name}', age {age}"));

                ui.button("SVG SNAPSHOT").clicked()
            });

            if let Some(doc) = maybe_svg {
                let file = std::fs::File::create("snap.svg").unwrap();
                svg::write(file, &doc).unwrap();
            }
        });
    })
}
