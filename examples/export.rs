use egui_export_svg::snapshot;

fn main() -> eframe::Result<()> {
    // Our application state:
    let mut name = "Arthur".to_owned();
    let mut age = 42;

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Egui export SVG", options, move |ctx, _frame| {
        let mut take_snapshot = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
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

            take_snapshot |= ui.button("SVG SNAPSHOT").clicked();
        });

        if take_snapshot {
            let doc = snapshot(ctx);
            let file = std::fs::File::create("snap.svg").unwrap();
            svg::write(file, &doc).unwrap();
        }
    })
}
