use egui::{epaint::ClippedShape, Color32, LayerId, Shape as EguiShape, Ui};
use svg::node::element::{path::Data, Group, Path as SvgPath};

pub fn shape_to_path(shape: &egui::Shape) -> Box<dyn svg::Node> {
    match shape {
        egui::Shape::Mesh(_mesh) => {
            eprintln!("TODO: Mesh");
            Box::new(SvgPath::default())
        }
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

            // TODO: Different sections have different positions?
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

                    let first_glyph_pos = row
                        .glyphs
                        .iter()
                        .find(|glyph| glyph.section_index == sec_idx)
                        .map(|glyph| glyph.pos)
                        .unwrap_or(row.rect.min);

                    let tl_pos = text.pos + first_glyph_pos.to_vec2();

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

                    group = group.add(
                        svg::node::element::Text::new(substring)
                            .set("x", tl_pos.x)
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

fn sorted_layer_ids(ctx: &egui::Context) -> Vec<LayerId> {
    let mut layer_ids: Vec<LayerId> = ctx.memory(|mem| mem.layer_ids().collect());
    layer_ids.sort_by_key(|id| id.order);
    layer_ids
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

fn rect_to_viewbox(rect: egui::Rect) -> (f32, f32, f32, f32) {
    (rect.min.x, rect.min.y, rect.width(), rect.height())
}

/// Take a snapshot of the entire screen as SVG
pub fn snapshot(ctx: &egui::Context) -> svg::Document {
    // Steal graphics data from context
    let layer_ids = sorted_layer_ids(ctx);
    let clipped_shapes: Vec<ClippedShape> = ctx.graphics(|gfx| {
        layer_ids
            .into_iter()
            .filter_map(|id| {
                gfx.get(id)
                    .map(|paint| paint.all_entries().cloned().collect::<Vec<_>>())
            })
            .flatten()
            .collect()
    });

    // Set viewbox to screen rect.
    // TODO: Is this what we want?
    let viewbox = rect_to_viewbox(ctx.screen_rect());

    svg::Document::new()
        .set("viewBox", viewbox)
        .add(clipped_shapes_to_group(&clipped_shapes))
}

fn clipped_shapes_to_group(shapes: &[ClippedShape]) -> Group {
    let mut group = Group::new();

    let mut next_clip_id = 0;
    for clip_shape in shapes {
        // Clip rectangles must be each assigned an ID
        // TODO: Make this more efficient- re-use IDs!
        let clip_id = format!("clip_rect_{next_clip_id}");
        next_clip_id += 1;

        let clip_path = svg::node::element::ClipPath::new()
            .set("id", clip_id.clone())
            .add(
                svg::node::element::Rectangle::new()
                    .set("x", clip_shape.clip_rect.min.x)
                    .set("y", clip_shape.clip_rect.min.y)
                    .set("width", clip_shape.clip_rect.width())
                    .set("height", clip_shape.clip_rect.height()),
            );

        let clip_group = Group::new()
            .set("clip-path", format!("url(#{clip_id})"))
            .add(clip_path)
            .add(shape_to_path(&clip_shape.shape));

        group = group.add(clip_group);
    }

    group
}

/// Runs the given function and, if it returns `true`, returns an SVG document
pub fn capture_scope(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> bool) -> Option<svg::Document> {
    let layer_ids = sorted_layer_ids(ui.ctx());

    let lengths_before: Vec<usize> = ui.ctx().graphics(|gfx| {
        layer_ids
            .iter()
            .copied()
            .filter_map(|id| gfx.get(id).map(|paint| paint.all_entries().len()))
            .collect()
    });

    let do_capture = f(ui);

    do_capture.then(|| {
        // Find the difference between the old and new shape vectors
        let mut new_clipped_shapes: Vec<ClippedShape> = ui.ctx().graphics(|gfx| {
            layer_ids
                .into_iter()
                .zip(lengths_before)
                .filter_map(|(id, idx_before)| {
                    gfx.get(id).map(|paint| {
                        paint
                            .all_entries()
                            .skip(idx_before)
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                })
                .flatten()
                .collect()
        });

        let total_rect = new_clipped_shapes
            .iter()
            .fold(egui::Rect::NOTHING, |acc, x| {
                acc.union(x.shape.visual_bounding_rect())
            });

        // Translate everything to the top left corner
        let to_tl = -total_rect.min.to_vec2();
        new_clipped_shapes.iter_mut().for_each(|clip_shape| {
            clip_shape.clip_rect = clip_shape.clip_rect.translate(to_tl);
            clip_shape.shape.translate(to_tl);
        });

        let viewbox = rect_to_viewbox(total_rect.translate(to_tl));

        svg::Document::new()
            .set("viewBox", viewbox)
            .add(clipped_shapes_to_group(&new_clipped_shapes))
    })
}
