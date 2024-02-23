use std::collections::HashMap;

use egui::{layers::PaintList, LayerId, Shape as EguiShape, Ui};
use svg::node::element::Path as SvgPath;

pub fn shape_to_path(shape: &egui::Shape) -> SvgPath {
    match dbg!(&shape) {
        egui::Shape::Noop => SvgPath::default(),
        other => {
            dbg!(other);
            SvgPath::default()
        }
    }
}

fn copy_paintlists(ctx: &egui::Context) -> HashMap<egui::LayerId, PaintList> {
    let layer_ids: Vec<LayerId> = ctx.memory(|mem| mem.layer_ids().collect());
    ctx.graphics(|gfx| {
        layer_ids
            .into_iter()
            .filter_map(|id| gfx.get(id).map(|paint| (id, paint.clone())))
            .collect()
    })
}

/*
pub fn wrap(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> egui::InnerResponse<bool>) -> egui::Response {

}
*/

pub fn snapshot(ctx: &egui::Context) -> svg::Document {
    // Steal graphics data from context
    let paintlists = copy_paintlists(ctx);

    // Set viewbox to screen rect.
    // TODO: Is this what we want?
    let screen_rect = ctx.screen_rect();
    let viewbox = (
        screen_rect.min.x,
        screen_rect.min.y,
        screen_rect.width(),
        screen_rect.height(),
    );
    let mut document = svg::Document::new().set("viewBox", viewbox);

    // Sort layers back to front
    let mut paintlists: Vec<(egui::LayerId, PaintList)> = paintlists.into_iter().collect();
    paintlists.sort_by_key(|(id, _)| id.order);

    // Convert
    for (_id, list) in paintlists {
        for clip_shape in list.all_entries() {
            // TODO: Clipping for SVG paths!
            let path = shape_to_path(&clip_shape.shape);
            document = document.add(path);
        }
    }

    document
}
