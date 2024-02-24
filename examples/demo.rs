use egui_demo_lib::DemoWindows;
use egui_export_svg::snapshot;

fn main() -> eframe::Result<()> {
    // Our application state:
    let _name = "Arthur".to_owned();
    let _age = 42;

    let mut demo = DemoWindows::default();

    let options = eframe::NativeOptions::default();
    eframe::run_simple_native("Egui export SVG", options, move |ctx, _frame| {
        demo.ui(ctx);

        let mut take_snapshot = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            take_snapshot |= ui.button("SVG SNAPSHOT").clicked();
        });

        if take_snapshot {
            let doc = snapshot(ctx);
            let file = std::fs::File::create("snap.svg").unwrap();
            svg::write(file, &doc).unwrap();
        }
    })
}
