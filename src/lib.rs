use std::collections::HashMap;

use egui::{layers::PaintList, Color32, LayerId, Shape as EguiShape};
use svg::{
    node::element::{path::Data, Group, Path as SvgPath},
    Node,
};

pub fn shape_to_path(shape: &egui::Shape) -> Box<dyn svg::Node> {
    match shape {
        egui::Shape::Mesh(_mesh) => Box::new(SvgPath::default()),
        /*egui::Shape::Mesh(mesh) => {
            dbg!(&mesh);
            let mut group = Group::new();
            for tri in mesh.indices.chunks_exact(3) {
                let mut data = Data::new();

                let pt = mesh.vertices.first().unwrap().pos;
                data = data.move_to((pt.x, pt.y));

                for idx in &tri[1..] {
                    let pt = mesh.vertices[*idx as usize].pos;
                    data = data.line_to((pt.x, pt.y));
                }
                data = data.close();

                let color = mesh.vertices[tri[0] as usize].color;
                let path = svg::node::element::Path::new()
                    .set("fill", convert_color(color))
                    .set("d", data);

                group = group.add(path);

            }
            Box::new(group)
        }*/
        egui::Shape::Noop => Box::new(SvgPath::default()),
        egui::Shape::Vec(children) => {
            let mut group = Group::new();
            for child in children {
                group = group.add(shape_to_path(child));
            }
            Box::new(group)
        }
        egui::Shape::Path(path) => {
            let mut data = Data::new();
            if let Some(pt) = path.points.first() {
                data = data.move_to((pt.x, pt.y));
            }
            for pt in &path.points[1..] {
                data = data.line_to((pt.x, pt.y));
            }
            if path.closed {
                data = data.close();
            }

            Box::new(
                svg::node::element::Path::new()
                    .fill(path.fill)
                    .stroke(path.stroke)
                    .set("d", data),
            )
        }
        egui::Shape::Circle(circle) => Box::new(
            svg::node::element::Circle::new()
                .set("cx", circle.center.x)
                .set("cy", circle.center.y)
                .set("r", circle.radius)
                .fill(circle.fill)
                .stroke(circle.stroke),
        ),
        egui::Shape::LineSegment { points, stroke } => Box::new(
            svg::node::element::Line::new()
                .set("x1", points[0].x)
                .set("y1", points[0].y)
                .set("x2", points[1].x)
                .set("y2", points[1].y)
                .stroke(*stroke),
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
                    .fill(rectangle.fill)
                    .stroke(rectangle.stroke),
            )
        }
        EguiShape::Text(text) => {
            let mut group = Group::new();

            let s = text.galley.text();

            // TODO: Different sections have different positions?
            dbg!(text.galley.job.sections.len());
            let anchor = match text.galley.job.halign {
                egui::Align::Min => "start",
                egui::Align::Center => "middle",
                egui::Align::Max => "end",
            };

            for row in &text.galley.rows {
                let Some(last_section_idx_in_row) = row.glyphs.last().map(|s| s.section_index)
                else {
                    continue;
                };

                for sec_idx in row.section_index_at_start..=last_section_idx_in_row {
                    let sec = &text.galley.job.sections[sec_idx as usize];

                    let width: f32 = row
                        .glyphs
                        .iter()
                        .filter(|glyph| glyph.section_index == sec_idx)
                        .map(|glyph| glyph.size.x)
                        .sum();

                    let substring: String = row
                        .glyphs
                        .iter()
                        .filter(|glyph| glyph.section_index == sec_idx)
                        .map(|glyph| glyph.chr)
                        .collect();

                    dbg!(sec_idx);
                    let first_glyph = row
                        .glyphs
                        .iter()
                        .find(|glyph| glyph.section_index == sec_idx)
                        .unwrap();

                    let tl_pos = text.pos + first_glyph.pos.to_vec2();

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

                    // Account for the space between the bottom of the text and the baseline
                    let row_height = row.rect.height() / text.galley.rows.len() as f32;
                    let y_offset = row_height - font_size;

                    group = group.add(
                        svg::node::element::Text::new(substring)
                            .set("x", sec.leading_space + tl_pos.x)
                            .set("y", tl_pos.y)
                            .set("font-size", font_size)
                            .set("font-family", font_family)
                            // TODO: Match egui's anchoring behaviour for multiple lines(?)
                            .set("text-anchor", anchor)
                            .set("textLength", width)
                            .fill(color),
                    );
                }
            }

            Box::new(group)
        }
        other => {
            println!("{:?}", other);
            Box::new(SvgPath::default())
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

fn color32_rgba(color: Color32) -> String {
    format!(
        "rgba({}, {}, {}, {})",
        color.r(),
        color.g(),
        color.b(),
        color.a() as f32 / 255.0
    )
}

trait EguiColorable: svg::Node + Sized {
    fn fill(mut self, color: Color32) -> Self {
        self.assign("fill", color32_rgba(color));
        self
    }

    fn stroke(mut self, stroke: egui::Stroke) -> Self {
        self.assign("stroke-width", stroke.width);
        self.assign("stroke", color32_rgba(stroke.color));
        self
    }
}

impl<T: svg::Node + Sized> EguiColorable for T {}

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
