use std::collections::HashMap;

use egui::{layers::PaintList, LayerId, Shape as EguiShape, Ui, Color32};
use svg::node::element::Path as SvgPath;

pub fn shape_to_path(shape: &egui::Shape) -> Box<dyn svg::Node> {
    match dbg!(&shape) {
        egui::Shape::Noop => Box::new(SvgPath::default()),
        egui::Shape::Circle(circle) => {
            Box::new(svg::node::element::Circle::new()
                .set("cx", circle.center.x)
                .set("cy", circle.center.y)
                .set("r", circle.radius)
                .set("fill", convert_color(circle.fill))
                .set("stroke-width", circle.stroke.width)
                .set("stroke", convert_color(circle.stroke.color)))
        },
        other => {
            dbg!(other);
            Box::new(SvgPath::default())
        }
    }
}

fn convert_color(color: Color32) -> String {
    format!("rgba({}, {}, {}, {})", color.r(), color.g(), color.b(), color.a())
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
