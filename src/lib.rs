use std::collections::HashMap;

use egui::{layers::PaintList, Color32, LayerId, Shape as EguiShape, Ui};
use svg::{
    node::element::{Group, Path as SvgPath},
    Node,
};

pub fn shape_to_path(shape: &egui::Shape) -> Box<dyn svg::Node> {
    match shape {
        egui::Shape::Noop => Box::new(SvgPath::default()),
        egui::Shape::Circle(circle) => Box::new(
            svg::node::element::Circle::new()
                .set("cx", circle.center.x)
                .set("cy", circle.center.y)
                .set("r", circle.radius)
                .set("fill", convert_color(circle.fill))
                .set("stroke-width", circle.stroke.width)
                .set("stroke", convert_color(circle.stroke.color)),
        ),
        EguiShape::Rect(rectangle) => {
            if !rectangle.rounding.is_same() {
                eprintln!("TODO: Implement per-edge rounding ...")
            }

            let rounding = 0_f32
                .max(rectangle.rounding.nw)
                .max(rectangle.rounding.ne)
                .max(rectangle.rounding.sw)
                .max(rectangle.rounding.se);

            Box::new(
                svg::node::element::Rectangle::new()
                    .set("x", rectangle.rect.min.x)
                    .set("y", rectangle.rect.min.y)
                    .set("rx", rounding)
                    .set("ry", rounding)
                    .set("width", rectangle.rect.width())
                    .set("height", rectangle.rect.height())
                    .set("fill", convert_color(rectangle.fill))
                    .set("stroke-width", rectangle.stroke.width)
                    .set("stroke", convert_color(rectangle.stroke.color)),
            )
        }
        EguiShape::Text(text) => {
            let mut group = Group::new();

            let s = text.galley.text();

            for sec in &text.galley.job.sections {
                let anchor = match text.galley.job.halign {
                    egui::Align::Min => "start",
                    egui::Align::Center => "middle",
                    egui::Align::Max => "end",
                };

                let font_family = match &sec.format.font_id.family {
                    egui::FontFamily::Proportional => "sans-serif",
                    egui::FontFamily::Monospace => "monospace",
                    egui::FontFamily::Name(fam) => {
                        eprintln!("Font family {} unsupported!", fam);
                        "sans-serif"
                    }
                };

                let font_size = sec.format.font_id.size;
                let mut color = text.override_text_color.unwrap_or(sec.format.color);
                if color == Color32::PLACEHOLDER {
                    color = text.fallback_color;
                }

                let length = text.galley.rect.width();
                dbg!(&sec.format.font_id.family);

                group = group.add(
                    svg::node::element::Text::new(&s[sec.byte_range.clone()])
                        .set("x", sec.leading_space + text.pos.x)
                        .set("y", text.pos.y + font_size)
                        .set("font-size", font_size)
                        .set("font-family", font_family)
                        .set("text-anchor", anchor)
                        .set("textLength", length)
                        .set("fill", convert_color(color)),
                );
            }

            Box::new(group)
        }
        other => {
            println!("{:?}", other);
            Box::new(SvgPath::default())
        }
    }
}

fn convert_color(color: Color32) -> String {
    format!(
        "rgba({}, {}, {}, {})",
        color.r(),
        color.g(),
        color.b(),
        color.a()
    )
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
